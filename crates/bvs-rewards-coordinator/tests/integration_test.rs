use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_registry::api::RegistryError;
use bvs_registry::testing::RegistryContract;
use bvs_rewards_coordinator::merkle::{
    calculate_earner_leaf_hash, calculate_rewards_submission_hash, calculate_token_leaf_hash,
    merkleize_sha256, EarnerTreeMerkleLeaf, RewardsMerkleClaim, RewardsSubmission,
    StrategyAndMultiplier, TokenTreeMerkleLeaf,
};
use bvs_rewards_coordinator::msg::ExecuteMsg;
use bvs_rewards_coordinator::testing::{RewardsCoordinatorContract, ONE_DAY};
use bvs_rewards_coordinator::ContractError;
use bvs_strategy_base::testing::StrategyBaseContract;
use bvs_strategy_manager::msg::ExecuteMsg as StrategyManagerExecuteMsg;
use bvs_strategy_manager::testing::StrategyManagerContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, HexBinary, OverflowError, OverflowOperation, StdError, Uint128};
use cw20::{BalanceResponse, Cw20Coin, MinterResponse};
use cw20_base::msg::ExecuteMsg::{IncreaseAllowance, Mint};
use cw_multi_test::App;
use rs_merkle::{algorithms::Sha256 as MerkleSha256, MerkleTree};

struct TestContracts {
    rewards_coordinator: RewardsCoordinatorContract,
    registry: RegistryContract,
    cw20token: Cw20TokenContract,
    strategy_manager: StrategyManagerContract,
    strategy1: StrategyBaseContract,
}

fn instantiate() -> (App, TestContracts) {
    let mut app = App::default();
    let env = mock_env();
    let owner = app.api().addr_make("owner");
    let rewards_updater = app.api().addr_make("rewards_updater");

    let registry = RegistryContract::new(&mut app, &env, None);
    // init rewards_coordinator
    let rewards_coordinator = RewardsCoordinatorContract::new(&mut app, &env, None);

    // set rewards_updater role
    let msg = ExecuteMsg::SetRewardsUpdater {
        addr: rewards_updater.to_string(),
    };
    rewards_coordinator.execute(&mut app, &owner, &msg).unwrap();

    // create strategy_manager and set it to rewards_coordinator
    let strategy_manager = StrategyManagerContract::new(&mut app, &env, None);
    let msg = ExecuteMsg::SetRouting {
        strategy_manager: strategy_manager.addr.to_string(),
    };
    rewards_coordinator.execute(&mut app, &owner, &msg).unwrap();

    // init cw20 token dummy
    let cw20token = Cw20TokenContract::new(&mut app, &env, None);

    // init strategy1
    let strategy1 = StrategyBaseContract::new(&mut app, &env, None);

    // add strategy1 to strategy_manager
    let msg = StrategyManagerExecuteMsg::AddStrategy {
        strategy: strategy1.addr.to_string(),
        whitelisted: true,
    };
    strategy_manager.execute(&mut app, &owner, &msg).unwrap();

    (
        app,
        TestContracts {
            rewards_coordinator,
            registry,
            cw20token,
            strategy_manager,
            strategy1,
        },
    )
}

// TODO: move to unit?
#[test]
fn create_rewards_submission_validation_failed() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");

    // Service creates RewardsSubmission
    let strategy1 = app.api().addr_make("strategy1");
    let strategy2 = app.api().addr_make("strategy2");

    {
        // amount validation failed
        let msg = ExecuteMsg::CreateRewardsSubmission {
            rewards_submissions: vec![RewardsSubmission {
                strategies_and_multipliers: vec![
                    StrategyAndMultiplier {
                        strategy: strategy1.clone(),
                        multiplier: 1,
                    },
                    StrategyAndMultiplier {
                        strategy: strategy2.clone(),
                        multiplier: 9,
                    },
                ],
                token: tc.cw20token.addr.clone(),
                amount: Uint128::new(0), // invalid amount
                start_timestamp: app
                    .block_info()
                    .time
                    .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
                duration: 7 * ONE_DAY,
            }],
        };
        let err = tc
            .rewards_coordinator
            .execute(&mut app, &owner, &msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::AmountCannotBeZero {}.to_string()
        );
    }

    {
        // start_timestamp validation failed
        let msg = ExecuteMsg::CreateRewardsSubmission {
            rewards_submissions: vec![RewardsSubmission {
                strategies_and_multipliers: vec![
                    StrategyAndMultiplier {
                        strategy: strategy1.clone(),
                        multiplier: 1,
                    },
                    StrategyAndMultiplier {
                        strategy: strategy2.clone(),
                        multiplier: 9,
                    },
                ],
                token: tc.cw20token.addr.clone(),
                amount: Uint128::new(1_000_000),
                start_timestamp: app.block_info().time, // not multiple of ONE_DAY
                duration: 7 * ONE_DAY,
            }],
        };
        let err = tc
            .rewards_coordinator
            .execute(&mut app, &owner, &msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::TimeMustBeMultipleOfCalcIntervalSec {}.to_string()
        );
    }

    {
        // duration validation failed
        let msg = ExecuteMsg::CreateRewardsSubmission {
            rewards_submissions: vec![RewardsSubmission {
                strategies_and_multipliers: vec![
                    StrategyAndMultiplier {
                        strategy: strategy1.clone(),
                        multiplier: 1,
                    },
                    StrategyAndMultiplier {
                        strategy: strategy2.clone(),
                        multiplier: 9,
                    },
                ],
                token: tc.cw20token.addr.clone(),
                amount: Uint128::new(1_000_000),
                start_timestamp: app.block_info().time,
                duration: 7, // not multiple of ONE_DAY
            }],
        };
        let err = tc
            .rewards_coordinator
            .execute(&mut app, &owner, &msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::DurationMustBeMultipleOfCalcIntervalSec {}.to_string()
        );
    }
}

#[test]
fn create_rewards_submission() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");
    let service = app.api().addr_make("service");

    const REWARDS_AMOUNT: Uint128 = Uint128::new(1_000_000);
    const MINT_AMOUNT: Uint128 = Uint128::new(1_000_001); // REWARDS_AMOUNT + 1

    // Service mints tokens
    let msg = Mint {
        recipient: service.to_string(),
        amount: MINT_AMOUNT,
    };
    tc.cw20token.execute(&mut app, &owner, &msg).unwrap();

    // Service allocate allowance to rewards_coordinator for token transfer
    let msg = IncreaseAllowance {
        spender: tc.rewards_coordinator.addr.to_string(),
        amount: REWARDS_AMOUNT,
        expires: None,
    };
    tc.cw20token.execute(&mut app, &service, &msg).unwrap();

    // Service creates RewardsSubmission
    let rewards_submission = RewardsSubmission {
        strategies_and_multipliers: vec![StrategyAndMultiplier {
            strategy: tc.strategy1.addr,
            multiplier: 1,
        }],
        token: tc.cw20token.addr.clone(),
        amount: REWARDS_AMOUNT,
        start_timestamp: app
            .block_info()
            .time
            .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
        duration: 7 * ONE_DAY,
    };

    // Service executes CreateRewardsSubmission
    let msg = ExecuteMsg::CreateRewardsSubmission {
        rewards_submissions: vec![rewards_submission.clone()],
    };
    let res = tc
        .rewards_coordinator
        .execute(&mut app, &service, &msg)
        .unwrap();

    assert_eq!(res.events.len(), 4);

    // assert reward submission event
    assert_eq!(
        res.events[1],
        Event::new("wasm-RewardsSubmissionCreated")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("sender", service.to_string())
            .add_attribute("nonce", 0.to_string())
            .add_attribute(
                "rewards_submission_hash",
                calculate_rewards_submission_hash(&service, 0, &rewards_submission).to_string(),
            )
            .add_attribute("token", tc.cw20token.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT.to_string())
    );

    // assert cw20 TransferFrom event
    assert_eq!(
        res.events[3],
        Event::new("wasm")
            .add_attribute("_contract_address", tc.cw20token.addr.to_string())
            .add_attribute("action", "transfer_from")
            .add_attribute("from", service.to_string())
            .add_attribute("to", tc.rewards_coordinator.addr.to_string())
            .add_attribute("by", tc.rewards_coordinator.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT.to_string())
    );

    // assert Service balance is deducted
    let service_balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: service.to_string(),
            },
        )
        .unwrap();
    assert_eq!(service_balance_res.balance, MINT_AMOUNT - REWARDS_AMOUNT);

    // assert RewardsCoordinator balance is increased
    let balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: tc.rewards_coordinator.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_res.balance, REWARDS_AMOUNT);
}

#[test]
fn create_rewards_submission_not_enough_balance() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");
    let service = app.api().addr_make("service");

    const REWARDS_AMOUNT: Uint128 = Uint128::new(1_000_000);
    const MINT_AMOUNT: Uint128 = Uint128::new(999_999); // REWARDS_AMOUNT - 1

    // Service mints tokens
    let msg = Mint {
        recipient: service.to_string(),
        amount: MINT_AMOUNT,
    };
    tc.cw20token.execute(&mut app, &owner, &msg).unwrap();

    // Service allocate allowance to rewards_coordinator for token transfer
    let msg = IncreaseAllowance {
        spender: tc.rewards_coordinator.addr.to_string(),
        amount: REWARDS_AMOUNT,
        expires: None,
    };
    tc.cw20token.execute(&mut app, &service, &msg).unwrap();

    // Service creates RewardsSubmission
    let rewards_submission = RewardsSubmission {
        strategies_and_multipliers: vec![StrategyAndMultiplier {
            strategy: tc.strategy1.addr,
            multiplier: 1,
        }],
        token: tc.cw20token.addr.clone(),
        amount: REWARDS_AMOUNT,
        start_timestamp: app
            .block_info()
            .time
            .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
        duration: 7 * ONE_DAY,
    };

    // Service executes CreateRewardsSubmission
    let msg = ExecuteMsg::CreateRewardsSubmission {
        rewards_submissions: vec![rewards_submission.clone()],
    };
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &service, &msg)
        .unwrap_err();

    // assert Overflow error from cw20 transferFrom error
    assert_eq!(
        err.root_cause().to_string(),
        StdError::overflow(OverflowError {
            operation: OverflowOperation::Sub
        })
        .to_string()
    );

    // assert Service balance is not deducted
    let service_balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: service.to_string(),
            },
        )
        .unwrap();
    assert_eq!(service_balance_res.balance, MINT_AMOUNT);

    // assert RewardsCoordinator balance is not increased
    let balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: tc.rewards_coordinator.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_res.balance, Uint128::zero());
}

#[test]
fn create_rewards_submission_not_enough_allowance() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");
    let service = app.api().addr_make("service");

    const REWARDS_AMOUNT: Uint128 = Uint128::new(1_000_000);
    const MINT_AMOUNT: Uint128 = Uint128::new(1_000_001); // REWARDS_AMOUNT + 1

    // Service mints tokens
    let msg = Mint {
        recipient: service.to_string(),
        amount: MINT_AMOUNT,
    };
    tc.cw20token.execute(&mut app, &owner, &msg).unwrap();

    // Service allocate allowance to rewards_coordinator for token transfer
    let msg = IncreaseAllowance {
        spender: tc.rewards_coordinator.addr.to_string(),
        amount: REWARDS_AMOUNT.checked_sub(Uint128::new(1)).unwrap(), // REWARDS_AMOUNT - 1
        expires: None,
    };
    tc.cw20token.execute(&mut app, &service, &msg).unwrap();

    // Service creates RewardsSubmission
    let rewards_submission = RewardsSubmission {
        strategies_and_multipliers: vec![StrategyAndMultiplier {
            strategy: tc.strategy1.addr,
            multiplier: 1,
        }],
        token: tc.cw20token.addr.clone(),
        amount: REWARDS_AMOUNT,
        start_timestamp: app
            .block_info()
            .time
            .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
        duration: 7 * ONE_DAY,
    };

    // Service executes CreateRewardsSubmission
    let msg = ExecuteMsg::CreateRewardsSubmission {
        rewards_submissions: vec![rewards_submission.clone()],
    };
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &service, &msg)
        .unwrap_err();

    // assert Overflow error from cw20 transferFrom error
    assert_eq!(
        err.root_cause().to_string(),
        StdError::overflow(OverflowError {
            operation: OverflowOperation::Sub
        })
        .to_string()
    );

    // assert Service balance is not deducted
    let service_balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: service.to_string(),
            },
        )
        .unwrap();
    assert_eq!(service_balance_res.balance, MINT_AMOUNT);

    // assert RewardsCoordinator balance is not increased
    let balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: tc.rewards_coordinator.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_res.balance, Uint128::zero());
}

#[test]
fn create_2_rewards_submission_different_tokens() {
    let (mut app, tc) = instantiate();
    let mock_env = mock_env();
    let owner = app.api().addr_make("owner");
    let service = app.api().addr_make("service");

    // Create second cw20 token
    let cw20token2 = Cw20TokenContract::new(
        &mut app,
        &mock_env,
        Some(cw20_base::msg::InstantiateMsg {
            symbol: "SATLL".to_string(),
            name: "Satlayer Test Token 2".to_string(),
            decimals: 18,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount: Uint128::new(1_000_001),
            }],
            mint: Some(MinterResponse {
                minter: owner.to_string(),
                cap: Some(Uint128::new(1_000_000_000_000_000_000_000)), // 1000e18 = 1e21
            }),
            marketing: None,
        }),
    );

    // create strategy2
    let strategy2 = StrategyBaseContract::new(&mut app, &mock_env, None);

    // add strategy2 to strategy_manager
    let msg = StrategyManagerExecuteMsg::AddStrategy {
        strategy: strategy2.addr.to_string(),
        whitelisted: true,
    };
    tc.strategy_manager.execute(&mut app, &owner, &msg).unwrap();

    // for token1
    const REWARDS_AMOUNT: Uint128 = Uint128::new(1_000_000);
    const MINT_AMOUNT: Uint128 = Uint128::new(1_000_001); // REWARDS_AMOUNT + 1

    // for token2
    const REWARDS_AMOUNT2: Uint128 = Uint128::new(999_990);
    const MINT_AMOUNT2: Uint128 = Uint128::new(999_992); // REWARDS_AMOUNT2 + 2

    // Service mints tokens - token1
    let msg = Mint {
        recipient: service.to_string(),
        amount: MINT_AMOUNT,
    };
    tc.cw20token.execute(&mut app, &owner, &msg).unwrap();

    // Service mints tokens - token2
    let msg = Mint {
        recipient: service.to_string(),
        amount: MINT_AMOUNT2,
    };
    cw20token2.execute(&mut app, &owner, &msg).unwrap();

    // Service allocate allowance to rewards_coordinator for token transfer - token1
    let msg = IncreaseAllowance {
        spender: tc.rewards_coordinator.addr.to_string(),
        amount: REWARDS_AMOUNT,
        expires: None,
    };
    tc.cw20token.execute(&mut app, &service, &msg).unwrap();

    // Service allocate allowance to rewards_coordinator for token transfer - token2
    let msg = IncreaseAllowance {
        spender: tc.rewards_coordinator.addr.to_string(),
        amount: REWARDS_AMOUNT2,
        expires: None,
    };
    cw20token2.execute(&mut app, &service, &msg).unwrap();

    // Service creates list of RewardsSubmission
    let rewards_submissions = vec![
        // token1 - strategy1
        RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: tc.strategy1.addr,
                multiplier: 1,
            }],
            token: tc.cw20token.addr.clone(),
            amount: REWARDS_AMOUNT,
            start_timestamp: app
                .block_info()
                .time
                .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
            duration: 7 * ONE_DAY,
        },
        // token2 - strategy2
        RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: strategy2.addr,
                multiplier: 5,
            }],
            token: cw20token2.addr.clone(),
            amount: REWARDS_AMOUNT2,
            start_timestamp: app
                .block_info()
                .time
                .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
            duration: 10 * ONE_DAY,
        },
    ];

    // Service executes CreateRewardsSubmission
    let msg = ExecuteMsg::CreateRewardsSubmission {
        rewards_submissions: rewards_submissions.clone(),
    };
    let res = tc
        .rewards_coordinator
        .execute(&mut app, &service, &msg)
        .unwrap();

    assert_eq!(res.events.len(), 7);

    // assert reward submission event - token 1
    assert_eq!(
        res.events[1],
        Event::new("wasm-RewardsSubmissionCreated")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("sender", service.to_string())
            .add_attribute("nonce", 0.to_string())
            .add_attribute(
                "rewards_submission_hash",
                calculate_rewards_submission_hash(&service, 0, &rewards_submissions[0]).to_string(),
            )
            .add_attribute("token", tc.cw20token.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT.to_string())
    );

    // assert cw20 TransferFrom event - token 1
    assert_eq!(
        res.events[4],
        Event::new("wasm")
            .add_attribute("_contract_address", tc.cw20token.addr.to_string())
            .add_attribute("action", "transfer_from")
            .add_attribute("from", service.to_string())
            .add_attribute("to", tc.rewards_coordinator.addr.to_string())
            .add_attribute("by", tc.rewards_coordinator.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT.to_string())
    );

    // assert reward submission event - token 2
    assert_eq!(
        res.events[2],
        Event::new("wasm-RewardsSubmissionCreated")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("sender", service.to_string())
            .add_attribute("nonce", 1.to_string())
            .add_attribute(
                "rewards_submission_hash",
                calculate_rewards_submission_hash(&service, 1, &rewards_submissions[1]).to_string(),
            )
            .add_attribute("token", cw20token2.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT2.to_string())
    );

    // assert cw20 TransferFrom event - token 2
    assert_eq!(
        res.events[6],
        Event::new("wasm")
            .add_attribute("_contract_address", cw20token2.addr.to_string())
            .add_attribute("action", "transfer_from")
            .add_attribute("from", service.to_string())
            .add_attribute("to", tc.rewards_coordinator.addr.to_string())
            .add_attribute("by", tc.rewards_coordinator.addr.to_string())
            .add_attribute("amount", REWARDS_AMOUNT2.to_string())
    );

    // assert Service balance is deducted - token 1
    let service_balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: service.to_string(),
            },
        )
        .unwrap();
    assert_eq!(service_balance_res.balance, MINT_AMOUNT - REWARDS_AMOUNT);

    // assert Service balance is deducted - token 2
    let service_balance_res: BalanceResponse = cw20token2
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: service.to_string(),
            },
        )
        .unwrap();
    assert_eq!(service_balance_res.balance, MINT_AMOUNT2 - REWARDS_AMOUNT2);

    // assert RewardsCoordinator balance is increased - token 1
    let balance_res: BalanceResponse = tc
        .cw20token
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: tc.rewards_coordinator.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_res.balance, REWARDS_AMOUNT);

    // assert RewardsCoordinator balance is increased - token 2
    let balance_res: BalanceResponse = cw20token2
        .query(
            &app,
            &cw20_base::msg::QueryMsg::Balance {
                address: tc.rewards_coordinator.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance_res.balance, REWARDS_AMOUNT2);
}

/// TEMPORARY FUNCTION - only used in tests
fn generate_proof(leaves: Vec<Vec<u8>>, leaf_index: usize) -> Vec<Vec<u8>> {
    // generate merkle tree
    let tree = MerkleTree::<MerkleSha256>::from_leaves(
        &leaves
            .iter()
            .map(|v| v.as_slice().try_into().unwrap())
            .collect::<Vec<[u8; 32]>>(),
    );

    tree.proof(&[leaf_index])
        .proof_hashes()
        .iter()
        .map(|h| h.to_vec())
        .collect()
}

#[test]
fn process_claim_not_yet_activated() {
    let (mut app, tc) = instantiate();
    let earner = app.api().addr_make("earner");
    let earner2 = app.api().addr_make("earner2");
    let dummy_token = app.api().addr_make("dummy_token");
    let rewards_updater = app.api().addr_make("rewards_updater");

    // submit_root
    // create token leaf - earner
    let token_leaves = vec![
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: tc.cw20token.addr.clone(),
            cumulative_earnings: Uint128::new(99),
        }),
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: dummy_token.clone(),
            cumulative_earnings: Uint128::new(90),
        }),
    ];

    // create token root - earner
    let token_root = merkleize_sha256(token_leaves.clone());

    // create token leaf - earner2
    let token_leaves2 = vec![
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: tc.cw20token.addr.clone(),
            cumulative_earnings: Uint128::new(70),
        }),
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: dummy_token.clone(),
            cumulative_earnings: Uint128::new(71),
        }),
    ];

    // create token root - earner2
    let token_root2 = merkleize_sha256(token_leaves2.clone());

    // create earner leaf
    let earner_leaves = vec![
        calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner.clone(),
            earner_token_root: HexBinary::from(token_root.clone()),
        }),
        calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner2.clone(),
            earner_token_root: HexBinary::from(token_root2.clone()),
        }),
    ];

    // create distribution root
    let distribution_root = merkleize_sha256(earner_leaves.clone());

    let msg = ExecuteMsg::SubmitRoot {
        root: HexBinary::from(distribution_root.clone()),
        rewards_calculation_end_timestamp: app.block_info().time.minus_hours(1).seconds(),
    };
    tc.rewards_coordinator
        .execute(&mut app, &rewards_updater, &msg)
        .expect("submit_root failed");

    // create earner token tree proof for tc.cw20token
    let token_tree_proof = generate_proof(token_leaves.clone(), 0);

    // create proof for earner
    let earner_tree_proof = generate_proof(earner_leaves.clone(), 0);

    // process_claim starts
    let msg = ExecuteMsg::ProcessClaim {
        claim: RewardsMerkleClaim {
            root_index: 0,
            earner_index: 0,
            earner_tree_proof: earner_tree_proof.concat(),
            earner_leaf: EarnerTreeMerkleLeaf {
                earner: earner.clone(),
                earner_token_root: HexBinary::from(token_root.clone()),
            },
            token_indices: vec![],
            token_tree_proofs: token_tree_proof,
            token_leaves: vec![],
        },
        recipient: earner.to_string(),
    };
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &earner, &msg)
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::RootNotActivatedYet {}.to_string()
    );
}

#[test]
fn process_claim() {
    let (mut app, tc) = instantiate();
    let earner = app.api().addr_make("earner");
    let earner2 = app.api().addr_make("earner2");
    let dummy_token = app.api().addr_make("dummy_token");
    let rewards_updater = app.api().addr_make("rewards_updater");

    {
        // Service calls CreateRewardsSubmission to send reward tokens to contract
        let owner = app.api().addr_make("owner");
        let service = app.api().addr_make("service");

        const REWARDS_AMOUNT: Uint128 = Uint128::new(1_000_000);
        const MINT_AMOUNT: Uint128 = Uint128::new(1_000_001); // REWARDS_AMOUNT + 1

        // Service mints tokens
        let msg = Mint {
            recipient: service.to_string(),
            amount: MINT_AMOUNT,
        };
        tc.cw20token.execute(&mut app, &owner, &msg).unwrap();

        // Service allocate allowance to rewards_coordinator for token transfer
        let msg = IncreaseAllowance {
            spender: tc.rewards_coordinator.addr.to_string(),
            amount: REWARDS_AMOUNT,
            expires: None,
        };
        tc.cw20token.execute(&mut app, &service, &msg).unwrap();

        // Service creates RewardsSubmission
        let rewards_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: tc.strategy1.addr.clone(),
                multiplier: 1,
            }],
            token: tc.cw20token.addr.clone(),
            amount: REWARDS_AMOUNT,
            start_timestamp: app
                .block_info()
                .time
                .minus_seconds(app.block_info().time.seconds() % ONE_DAY),
            duration: 7 * ONE_DAY,
        };

        // Service executes CreateRewardsSubmission
        let msg = ExecuteMsg::CreateRewardsSubmission {
            rewards_submissions: vec![rewards_submission.clone()],
        };
        tc.rewards_coordinator
            .execute(&mut app, &service, &msg)
            .expect("create rewards submission failed");
    }

    // submit_root
    // create token leaf - earner
    let token_leaves = vec![
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: tc.cw20token.addr.clone(),
            cumulative_earnings: Uint128::new(99),
        }),
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: dummy_token.clone(),
            cumulative_earnings: Uint128::new(90),
        }),
    ];

    // create token root - earner
    let token_root = merkleize_sha256(token_leaves.clone());

    // create token leaf - earner2
    let token_leaves2 = vec![
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: tc.cw20token.addr.clone(),
            cumulative_earnings: Uint128::new(70),
        }),
        calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
            token: dummy_token.clone(),
            cumulative_earnings: Uint128::new(71),
        }),
    ];

    // create token root - earner2
    let token_root2 = merkleize_sha256(token_leaves2.clone());

    // create earner leaf
    let earner_leaves = vec![
        calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner.clone(),
            earner_token_root: HexBinary::from(token_root.clone()),
        }),
        calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner2.clone(),
            earner_token_root: HexBinary::from(token_root2.clone()),
        }),
    ];

    // create distribution root
    let distribution_root = merkleize_sha256(earner_leaves.clone());

    let msg = ExecuteMsg::SubmitRoot {
        root: HexBinary::from(distribution_root.clone()),
        rewards_calculation_end_timestamp: app.block_info().time.minus_hours(1).seconds(),
    };
    tc.rewards_coordinator
        .execute(&mut app, &rewards_updater, &msg)
        .expect("submit_root failed");

    // create earner token tree proof for tc.cw20token
    let token_tree_proof = generate_proof(token_leaves.clone(), 0);

    // create proof for earner
    let earner_tree_proof = generate_proof(earner_leaves.clone(), 0);

    // FAST FORWARD TO AFTER ACTIVATION DELAY
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(60);
    });

    // process_claim starts
    let msg = ExecuteMsg::ProcessClaim {
        claim: RewardsMerkleClaim {
            root_index: 0,
            earner_index: 0,
            earner_tree_proof: earner_tree_proof.concat(),
            earner_leaf: EarnerTreeMerkleLeaf {
                earner: earner.clone(),
                earner_token_root: HexBinary::from(token_root.clone()),
            },
            token_indices: vec![0],
            token_tree_proofs: token_tree_proof,
            token_leaves: vec![TokenTreeMerkleLeaf {
                token: tc.cw20token.addr.clone(),
                cumulative_earnings: Uint128::new(99),
            }],
        },
        recipient: earner.to_string(),
    };
    let res = tc
        .rewards_coordinator
        .execute(&mut app, &earner, &msg)
        .unwrap();

    assert_eq!(res.events.len(), 4);

    // assert RewardsClaimed event
    assert_eq!(
        res.events[1],
        Event::new("wasm-RewardsClaimed")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("root", HexBinary::from(distribution_root.clone()).to_hex())
            .add_attribute("earner", earner.to_string())
            .add_attribute("claimer", earner.to_string())
            .add_attribute("recipient", earner.to_string())
            .add_attribute("token", tc.cw20token.addr.to_string())
            .add_attribute("amount", Uint128::new(99).to_string())
    );

    // assert cw20 Transfer event
    assert_eq!(
        res.events[3],
        Event::new("wasm")
            .add_attribute("_contract_address", tc.cw20token.addr.to_string())
            .add_attribute("action", "transfer")
            .add_attribute("from", tc.rewards_coordinator.addr.to_string())
            .add_attribute("to", earner.to_string())
            .add_attribute("amount", Uint128::new(99).to_string())
    );
}

#[test]
fn submit_root() {
    let (mut app, tc) = instantiate();

    let earner = app.api().addr_make("earner");
    let rewards_updater = app.api().addr_make("rewards_updater");

    // create token leaf
    let token_leaves = vec![calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
        token: tc.cw20token.addr.clone(),
        cumulative_earnings: Uint128::new(99),
    })];

    // create token root
    let token_root = merkleize_sha256(token_leaves.clone());

    // create earner leaf
    let earner_leaves = vec![calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
        earner: earner.clone(),
        earner_token_root: HexBinary::from(token_root.clone()),
    })];

    // create earner root
    let earner_root = merkleize_sha256(earner_leaves.clone());

    {
        // 1st submit_root
        let msg = ExecuteMsg::SubmitRoot {
            root: HexBinary::from(earner_root.clone()),
            rewards_calculation_end_timestamp: app.block_info().time.minus_hours(1).seconds(),
        };
        let res = tc
            .rewards_coordinator
            .execute(&mut app, &rewards_updater, &msg)
            .unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(
            res.events[1],
            Event::new("wasm-DistributionRootSubmitted")
                .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
                .add_attribute("root_index", 0.to_string())
                .add_attribute("root", HexBinary::from(earner_root.clone()).to_hex())
                .add_attribute(
                    "rewards_calculation_end_timestamp",
                    app.block_info().time.minus_hours(1).seconds().to_string()
                )
                .add_attribute(
                    "activated_at",
                    app.block_info().time.plus_seconds(60).seconds().to_string()
                )
        );
    }
    {
        // 2nd submit_root
        // emulate time passing
        app.update_block(|block| {
            block.height += 100;
            block.time = block.time.plus_seconds(100 * 6);
        });
        let dummy_cw20_token = app.api().addr_make("dummy_cw20_token");
        let earner2 = app.api().addr_make("earner2");

        // create token leaf
        let token_leaves = vec![
            calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
                token: tc.cw20token.addr.clone(),
                cumulative_earnings: Uint128::new(199),
            }),
            calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
                token: dummy_cw20_token.clone(),
                cumulative_earnings: Uint128::new(70),
            }),
        ];

        // create token root
        let token_root = merkleize_sha256(token_leaves.clone());

        // create earner leaf
        let earner_leaves = vec![calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner.clone(),
            earner_token_root: HexBinary::from(token_root.clone()),
        })];

        // create earner root
        let earner_root = merkleize_sha256(earner_leaves.clone());

        // create 2nd earner root
        let token_leaves = vec![
            calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
                token: tc.cw20token.addr.clone(),
                cumulative_earnings: Uint128::new(5),
            }),
            calculate_token_leaf_hash(&TokenTreeMerkleLeaf {
                token: dummy_cw20_token.clone(),
                cumulative_earnings: Uint128::new(1),
            }),
        ];
        let token_root = merkleize_sha256(token_leaves.clone());
        let earner2_leaves = vec![calculate_earner_leaf_hash(&EarnerTreeMerkleLeaf {
            earner: earner2.clone(),
            earner_token_root: HexBinary::from(token_root.clone()),
        })];
        let earner2_root = merkleize_sha256(earner2_leaves.clone());

        // create distribution root
        let distribution_root = merkleize_sha256(vec![earner_root.clone(), earner2_root.clone()]);

        // submit root
        let msg = ExecuteMsg::SubmitRoot {
            root: HexBinary::from(distribution_root.clone()),
            rewards_calculation_end_timestamp: app.block_info().time.minus_hours(1).seconds(),
        };
        let res = tc
            .rewards_coordinator
            .execute(&mut app, &rewards_updater, &msg)
            .unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(
            res.events[1],
            Event::new("wasm-DistributionRootSubmitted")
                .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
                .add_attribute("root_index", 1.to_string())
                .add_attribute("root", HexBinary::from(distribution_root.clone()).to_hex())
                .add_attribute(
                    "rewards_calculation_end_timestamp",
                    app.block_info().time.minus_hours(1).seconds().to_string()
                )
                .add_attribute(
                    "activated_at",
                    app.block_info().time.plus_seconds(60).seconds().to_string()
                )
        );
    }
}

#[test]
fn set_activation_delay() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");

    let msg = ExecuteMsg::SetActivationDelay {
        new_activation_delay: 100,
    };
    let res = tc
        .rewards_coordinator
        .execute(&mut app, &owner, &msg)
        .unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-SetActivationDelay")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("old_activation_delay", "60")
            .add_attribute("new_activation_delay", "100")
    );
}

#[test]
fn set_activation_delay_but_paused() {
    let (mut app, tc) = instantiate();
    let owner = Addr::unchecked(tc.registry.init.owner.clone());

    tc.registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let msg = ExecuteMsg::SetActivationDelay {
        new_activation_delay: 100,
    };
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &owner, &msg)
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Registry(RegistryError::IsPaused).to_string()
    );
}

#[test]
fn set_routing() {
    let (mut app, tc) = instantiate();

    let strategy_manager = app.api().addr_make("strategy_manager");
    let msg = ExecuteMsg::SetRouting {
        strategy_manager: strategy_manager.to_string(),
    };

    let owner = Addr::unchecked(tc.registry.init.owner.clone());
    tc.rewards_coordinator
        .execute(&mut app, &owner, &msg)
        .unwrap();
}

#[test]
fn set_routing_not_owner() {
    let (mut app, tc) = instantiate();

    let strategy_manager = app.api().addr_make("strategy_manager");
    let msg = ExecuteMsg::SetRouting {
        strategy_manager: strategy_manager.to_string(),
    };

    let not_owner = app.api().addr_make("not_owner");
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &not_owner, &msg)
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Ownership(bvs_library::ownership::OwnershipError::Unauthorized).to_string()
    );
}

#[test]
fn set_rewards_updater() {
    let (mut app, tc) = instantiate();
    let owner = app.api().addr_make("owner");

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        addr: new_updater.to_string(),
    };
    let res = tc
        .rewards_coordinator
        .execute(&mut app, &owner, &msg)
        .unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-SetRewardsUpdater")
            .add_attribute("_contract_address", tc.rewards_coordinator.addr.to_string())
            .add_attribute("addr", new_updater.to_string())
    );
}

#[test]
fn set_rewards_updater_but_paused() {
    let (mut app, tc) = instantiate();
    let owner = Addr::unchecked(tc.rewards_coordinator.init.owner.clone());

    tc.registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        addr: new_updater.to_string(),
    };
    let err = tc
        .rewards_coordinator
        .execute(&mut app, &owner, &msg)
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Registry(RegistryError::IsPaused).to_string()
    );
}
