use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_rewards::merkle::{Leaf, Sha3_256Algorithm};
use bvs_rewards::msg::ExecuteMsg::{ClaimRewards, DistributeRewards};
use bvs_rewards::msg::{
    ClaimRewardsProof, DistributionRootResponse, QueryMsg, RewardDistribution, RewardsType,
};
use bvs_rewards::testing::{generate_merkle_proof, generate_merkle_tree, RewardsContract};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, coins, Event, HexBinary, Uint128};
use cw_multi_test::{App, Executor};
use rs_merkle::MerkleTree;

fn instantiate() -> (App, RewardsContract, Cw20TokenContract) {
    let mut app = App::new(|router, api, storage| {
        let owner = api.addr_make("owner");
        router
            .bank
            .init_balance(
                storage,
                &owner,
                vec![
                    coin(Uint128::MAX.u128(), "rock"),
                    coin(Uint128::MAX.u128(), "stone"),
                ],
            )
            .unwrap();
    });
    let env = mock_env();

    let rewards_contract = RewardsContract::new(&mut app, &env, None);
    let cw20 = Cw20TokenContract::new(&mut app, &env, None);

    (app, rewards_contract, cw20)
}

// prep_merkle_tree_equalised generates n leaves for n different earners with equalised amount (1:1 ratio).
fn prep_merkle_tree_equalised(app: &App, n: u8, amount: Uint128) -> MerkleTree<Sha3_256Algorithm> {
    // iterate and generate n leaves with equalised amount
    prep_merkle_tree_rationed(app, vec![1; n as usize], amount)
}

// prep_merkle_tree_ratioed generates n leaves for n different earners with ratio amount.
// n is implicit by the length of the ratio vector.
// The amount is distributed to each earner according to the ratio
fn prep_merkle_tree_rationed(
    app: &App,
    ratio: Vec<u8>,
    amount: Uint128,
) -> MerkleTree<Sha3_256Algorithm> {
    let total_ratio = ratio.iter().sum::<u8>() as u128;
    let n = ratio.len() as u8;

    // iterate and generate n leaves with equalised amount
    let leaves = (0..n)
        .map(|i| Leaf {
            earner: app
                .api()
                .addr_make(format!("earner{}", i + 1).as_str())
                .to_string(),
            amount: Uint128::new(ratio[i as usize] as u128 * amount.u128() / total_ratio),
        })
        .collect::<Vec<_>>();
    generate_merkle_tree(&leaves)
}

#[test]
fn test_distribute_rewards_bank_mismatch_denom() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "rock" denom to service
    app.send_tokens(owner, service.clone(), coins(10_000, "rock").as_ref())
        .expect("Failed to mint tokens");

    // send "rock" denom in info instead of "stone"
    let err = rewards_contract
        .execute_with_funds(
            &mut app,
            &service,
            &DistributeRewards {
                merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                reward_distribution,
                reward_type: RewardsType::Bank,
            },
            coins(10_000, "rock"),
        )
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Payment error: Received unsupported denom 'rock'"
    );
}

#[test]
fn test_distribute_rewards_bank_mismatch_amount() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "stone" denom to service
    app.send_tokens(owner, service.clone(), coins(10_001, bank_token).as_ref())
        .expect("Failed to mint tokens");

    {
        // send 9_999 denom in info instead of 10_000
        let err = rewards_contract
            .execute_with_funds(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Bank,
                },
                coins(9_999, bank_token), // send a lower amount than reward_distribution.amount
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Funds sent do not match the funds received"
        );
    }
    {
        // send 10_001 denom in info instead of 10_000
        let err = rewards_contract
            .execute_with_funds(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                    reward_distribution,
                    reward_type: RewardsType::Bank,
                },
                coins(10_001, bank_token), // send a higher amount than reward_distribution.amount
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Funds sent do not match the funds received"
        );
    }
}

#[test]
fn test_distribute_and_claim_rewards_bank() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(10_000));
    let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "stone" denom to service
    app.send_tokens(owner, service.clone(), coins(10_001, bank_token).as_ref())
        .expect("Failed to mint tokens");

    // DISTRIBUTE flow
    {
        // send 10_000 denom in info
        let res = rewards_contract
            .execute_with_funds(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: merkle_root.clone(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Bank,
                },
                coins(10_000, bank_token),
            )
            .unwrap();

        // assert events are correct
        assert_eq!(res.events.len(), 2);
        assert_eq!(
            res.events[1],
            Event::new("wasm-DistributeRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.clone())
                .add_attribute("token", bank_token)
                .add_attribute("amount", "10000")
                .add_attribute("root", merkle_root.to_hex())
        );

        // assert contract balance is increased
        let contract_balance: u128 = app
            .wrap()
            .query_balance(&rewards_contract.addr, bank_token)
            .unwrap()
            .amount
            .u128();
        assert_eq!(contract_balance, 10_000u128); // 10_000

        // assert service balance is reduced
        let balance: u128 = app
            .wrap()
            .query_balance(&service, bank_token)
            .unwrap()
            .amount
            .u128();
        assert_eq!(balance, 1u128); // 10_001 - 10_000

        // assert DISTRIBUTION_ROOTS state is updated
        let DistributionRootResponse(root) = rewards_contract
            .query(
                &app,
                &QueryMsg::DistributionRoot {
                    service: service.to_string(),
                    token: bank_token.to_string(),
                },
            )
            .unwrap();
        assert_eq!(root, merkle_root.to_string());
    }

    // CLAIM flow
    {
        let earner = app.api().addr_make("earner2");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_000),
        };
        let leaf_index = 1u32;
        let total_leaves_count = 10u32;
        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();
        let recipient = earner.to_string();
        let claim = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Bank,
                    service: service.to_string(),
                    token: bank_token.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.clone(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim.events.len(), 3);

        // assert ClaimRewards event
        assert_eq!(
            claim.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", leaf.amount.to_string())
                .add_attribute("token", bank_token)
        );

        // assert transfer event
        assert_eq!(
            claim.events[2],
            Event::new("transfer")
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("sender", rewards_contract.addr.clone())
                .add_attribute("amount", format!("{}{}", leaf.amount, bank_token))
        );

        // assert contract balance is reduced
        let balance: u128 = app
            .wrap()
            .query_balance(&rewards_contract.addr, bank_token)
            .unwrap()
            .amount
            .u128();
        assert_eq!(balance, 9_000u128); // 10_000 - 1_000

        // assert recipient balance is increased
        let recipient_balance: u128 = app
            .wrap()
            .query_balance(&recipient, bank_token)
            .unwrap()
            .amount
            .u128();
        assert_eq!(recipient_balance, 1_000u128); // 1_000
    }
}

#[test]
fn test_distribute_and_claim_rewards_cw20() {
    let (mut app, rewards_contract, cw20) = instantiate();

    let service = app.api().addr_make("service");

    let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(10_000));
    let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

    // DISTRIBUTE flow
    {
        let reward_distribution = RewardDistribution {
            token: cw20.addr.to_string(),
            amount: Uint128::new(10_000),
        };

        // send 10_001 cw20 token to service
        cw20.fund(&mut app, &service, reward_distribution.amount.u128() + 1);

        // allow rewards contract to transfer token
        cw20.increase_allowance(
            &mut app,
            &service,
            &rewards_contract.addr,
            reward_distribution.amount.u128(),
        );

        // send 10_000 cw20 token
        let res = rewards_contract
            .execute(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: merkle_root.clone(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Cw20,
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(res.events.len(), 4);
        assert_eq!(
            res.events[1],
            Event::new("wasm-DistributeRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.clone())
                .add_attribute("token", cw20.addr.clone())
                .add_attribute("amount", "10000")
                .add_attribute("root", merkle_root.to_hex())
        );

        // assert DISTRIBUTION_ROOTS state is updated
        let DistributionRootResponse(root) = rewards_contract
            .query(
                &app,
                &QueryMsg::DistributionRoot {
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                },
            )
            .unwrap();
        assert_eq!(root, merkle_root.to_string());

        // assert service balance is reduced
        let balance: u128 = cw20.balance(&app, &service);
        assert_eq!(balance, 1u128); // 10_001 - 10_000

        // assert contract balance is increased
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 10_000u128); // 10_000
    }

    // CLAIM flow
    {
        let earner = app.api().addr_make("earner9");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_000),
        };
        let recipient = earner.clone();

        let leaf_index = 8u32;
        let total_leaves_count = 10u32;

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let claim_res = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim_res.events.len(), 4);

        // assert ClaimRewards event
        assert_eq!(
            claim_res.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", leaf.amount.to_string())
                .add_attribute("token", cw20.addr.to_string())
        );

        // assert transfer event
        assert_eq!(
            claim_res.events[3],
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.clone())
                .add_attribute("action", "transfer")
                .add_attribute("from", rewards_contract.addr.clone())
                .add_attribute("to", recipient.to_string())
                .add_attribute("amount", leaf.amount.to_string())
        );

        // assert contract balance is reduced
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 9_000u128); // 10_000 - 1_000

        // assert recipient balance is increased
        let recipient_balance: u128 = cw20.balance(&app, &recipient);
        assert_eq!(recipient_balance, 1_000u128); // 1_000
    }
}

#[test]
fn test_claim_rewards_after_multiple_distribution() {
    let (mut app, rewards_contract, cw20) = instantiate();

    let service = app.api().addr_make("service");

    // DISTRIBUTE flow - 1
    {
        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(10_000));
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let reward_distribution = RewardDistribution {
            token: cw20.addr.to_string(),
            amount: Uint128::new(10_000),
        };

        // send 10_001 cw20 token to service
        cw20.fund(&mut app, &service, reward_distribution.amount.u128() + 1);

        // allow rewards contract to transfer token
        cw20.increase_allowance(
            &mut app,
            &service,
            &rewards_contract.addr,
            reward_distribution.amount.u128(),
        );

        // send 10_000 cw20 token
        let res = rewards_contract
            .execute(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: merkle_root.clone(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Cw20,
                },
            )
            .unwrap();

        // assert service balance is reduced
        let balance: u128 = cw20.balance(&app, &service);
        assert_eq!(balance, 1u128); // 10_001 - 10_000

        // assert contract balance is increased
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 10_000u128); // 10_000
    }

    // DISTRIBUTE flow - 2
    {
        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(15_000)); // 10_000 + 5_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let reward_distribution = RewardDistribution {
            token: cw20.addr.to_string(),
            amount: Uint128::new(5_000),
        };

        // send 5_001 cw20 token to service
        cw20.fund(&mut app, &service, reward_distribution.amount.u128() + 1);

        // allow rewards contract to transfer token
        cw20.increase_allowance(
            &mut app,
            &service,
            &rewards_contract.addr,
            reward_distribution.amount.u128(),
        );

        // send 5_000 cw20 token
        rewards_contract
            .execute(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: merkle_root.clone(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Cw20,
                },
            )
            .unwrap();

        // assert service balance is reduced
        let balance: u128 = cw20.balance(&app, &service);
        assert_eq!(balance, 2u128); // 5_002 - 5_000

        // assert contract balance is increased
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 15_000u128); // 10_000 + 5_000
    }

    // CLAIM flow - 1 (fail due to using wrong amount)
    {
        let earner = app.api().addr_make("earner9");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_000),
        };
        let recipient = earner.clone();

        let leaf_index = 8u32;
        let total_leaves_count = 10u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(15_000)); // 10_000 + 5_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let err = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap_err();

        // assert events are correct
        assert_eq!(
            err.root_cause().to_string(),
            "Merkle proof verification failed: Invalid Merkle proof"
        );
    }

    // CLAIM flow - 2 (success)
    {
        let earner = app.api().addr_make("earner9");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_500), // 15_000 / 10 = 1_500
        };
        let recipient = earner.clone();

        let leaf_index = 8u32;
        let total_leaves_count = 10u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(15_000)); // 10_000 + 5_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let claim_res = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim_res.events.len(), 4);

        // assert ClaimRewards event
        assert_eq!(
            claim_res.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", leaf.amount.to_string())
                .add_attribute("token", cw20.addr.to_string())
        );

        // assert transfer event
        assert_eq!(
            claim_res.events[3],
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.clone())
                .add_attribute("action", "transfer")
                .add_attribute("from", rewards_contract.addr.clone())
                .add_attribute("to", recipient.to_string())
                .add_attribute("amount", leaf.amount.to_string())
        );

        // assert contract balance is reduced
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 13_500u128); // 15_000 - 1_500

        // assert recipient balance is increased
        let recipient_balance: u128 = cw20.balance(&app, &recipient);
        assert_eq!(recipient_balance, 1_500u128); // 1_500
    }

    // CLAIM flow - 3 (error due to previous claim)
    {
        let earner = app.api().addr_make("earner9");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_500), // 15_000 / 10 = 1_500
        };
        let recipient = earner.clone();

        let leaf_index = 8u32;
        let total_leaves_count = 10u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(15_000)); // 10_000 + 5_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let err = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap_err();

        assert_eq!(err.root_cause().to_string(), "Rewards already claimed");
    }

    // DISTRIBUTE flow - 3 (add new earners)
    {
        let merkle_tree = prep_merkle_tree_equalised(&app, 15, Uint128::new(30_000)); // 10_000 + 5_000 + 15_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let reward_distribution = RewardDistribution {
            token: cw20.addr.to_string(),
            amount: Uint128::new(15_000),
        };

        // send 15_001 cw20 token to service
        cw20.fund(&mut app, &service, reward_distribution.amount.u128() + 1);

        // allow rewards contract to transfer token
        cw20.increase_allowance(
            &mut app,
            &service,
            &rewards_contract.addr,
            reward_distribution.amount.u128(),
        );

        // send 15_000 cw20 token
        rewards_contract
            .execute(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: merkle_root.clone(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Cw20,
                },
            )
            .unwrap();

        // assert service balance is reduced
        let balance: u128 = cw20.balance(&app, &service);
        assert_eq!(balance, 3u128); // 15_003 - 15_000

        // assert contract balance is increased
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 28_500u128); // 10_000 + 5_000 - 1_500 + 15_000
    }

    // CLAIM flow - 4 (success, transfer new rewards)
    {
        let earner = app.api().addr_make("earner9");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(2_000), // 30_000 / 15 = 2_000
        };
        let recipient = earner.clone();

        let leaf_index = 8u32;
        let total_leaves_count = 15u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 15, Uint128::new(30_000)); // 10_000 + 5_000 + 15_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let claim_res = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim_res.events.len(), 4);

        let amount_to_claim = leaf.amount - Uint128::new(1_500); // 2_000 - 1_500

        // assert ClaimRewards event
        assert_eq!(
            claim_res.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", amount_to_claim.to_string())
                .add_attribute("token", cw20.addr.to_string())
        );

        // assert transfer event
        assert_eq!(
            claim_res.events[3],
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.clone())
                .add_attribute("action", "transfer")
                .add_attribute("from", rewards_contract.addr.clone())
                .add_attribute("to", recipient.to_string())
                .add_attribute("amount", amount_to_claim.to_string())
        );

        // assert contract balance is reduced
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 28_000u128); // 15_000 - 1_500 + 15_000 - 500

        // assert recipient balance is increased
        let recipient_balance: u128 = cw20.balance(&app, &recipient);
        assert_eq!(recipient_balance, 2_000u128); // 1_500 + 500
    }

    // CLAIM flow - 5 (success, earner8 using previous root - distribute flow 2)
    {
        let earner = app.api().addr_make("earner8");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(1_500), // 15_000 / 10 = 1_500
        };
        let recipient = earner.clone();

        let leaf_index = 7u32;
        let total_leaves_count = 10u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 10, Uint128::new(15_000)); // 10_000 + 5_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let claim_res = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim_res.events.len(), 4);

        // assert ClaimRewards event
        assert_eq!(
            claim_res.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", leaf.amount.to_string())
                .add_attribute("token", cw20.addr.to_string())
        );

        // assert transfer event
        assert_eq!(
            claim_res.events[3],
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.clone())
                .add_attribute("action", "transfer")
                .add_attribute("from", rewards_contract.addr.clone())
                .add_attribute("to", recipient.to_string())
                .add_attribute("amount", leaf.amount.to_string())
        );

        // assert contract balance is reduced
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 26_500u128); // 28_000 - 1_500

        // assert recipient balance is increased
        let recipient_balance: u128 = cw20.balance(&app, &recipient);
        assert_eq!(recipient_balance, 1_500u128); // 1_500
    }

    // CLAIM flow - 6 (success, earner8 using recent root)
    {
        let earner = app.api().addr_make("earner8");
        let leaf = Leaf {
            earner: earner.to_string(),
            amount: Uint128::new(2_000), // 30_000 / 15 = 2_000
        };
        let recipient = earner.clone();

        let leaf_index = 7u32;
        let total_leaves_count = 15u32;

        let merkle_tree = prep_merkle_tree_equalised(&app, 15, Uint128::new(30_000)); // 10_000 + 5_000 + 15_000
        let merkle_root = HexBinary::from(merkle_tree.root().unwrap());

        let proof = generate_merkle_proof(&merkle_tree, leaf_index).unwrap();

        let claim_res = rewards_contract
            .execute(
                &mut app,
                &earner,
                &ClaimRewards {
                    claim_rewards_proof: ClaimRewardsProof {
                        root: merkle_root.clone(),
                        proof,
                        leaf_index,
                        total_leaves_count,
                    },
                    reward_type: RewardsType::Cw20,
                    service: service.to_string(),
                    token: cw20.addr.to_string(),
                    amount: leaf.amount,
                    recipient: recipient.to_string(),
                },
            )
            .unwrap();

        // assert events are correct
        assert_eq!(claim_res.events.len(), 4);

        let amount_to_claim = leaf.amount - Uint128::new(1_500); // 2_000 - 1_500

        // assert ClaimRewards event
        assert_eq!(
            claim_res.events[1],
            Event::new("wasm-ClaimRewards")
                .add_attribute("_contract_address", rewards_contract.addr.clone())
                .add_attribute("service", service.to_string())
                .add_attribute("earner", earner.to_string())
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("total_claimed_amount", leaf.amount.to_string())
                .add_attribute("amount", amount_to_claim.to_string())
                .add_attribute("token", cw20.addr.to_string())
        );

        // assert transfer event
        assert_eq!(
            claim_res.events[3],
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.clone())
                .add_attribute("action", "transfer")
                .add_attribute("from", rewards_contract.addr.clone())
                .add_attribute("to", recipient.to_string())
                .add_attribute("amount", amount_to_claim.to_string())
        );

        // assert contract balance is reduced
        let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
        assert_eq!(contract_balance, 26_000u128); // 26_500 - 500

        // assert recipient balance is increased
        let recipient_balance: u128 = cw20.balance(&app, &recipient);
        assert_eq!(recipient_balance, 2_000u128); // 1_500 + 500
    }
}
