use bvs_delegation_manager::testing::DelegationManagerContract;
use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_strategy_base::msg::{InstantiateMsg as StrategyBaseInstantiateMsg, TotalSharesResponse};
use bvs_strategy_base::testing::{Cw20TokenContract, StrategyBaseContract};
use bvs_strategy_manager::msg::{StakerDepositListResponse, StrategyShare};
use bvs_strategy_manager::{
    msg::{ExecuteMsg, IsStrategyWhitelistedResponse, QueryMsg},
    testing::StrategyManagerContract,
    ContractError,
};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Event, Uint128};
use cw_multi_test::App;

struct TestContracts {
    strategy_manager: StrategyManagerContract,
    delegation_manager: DelegationManagerContract,
    pauser: PauserContract,
}

impl TestContracts {
    fn init(app: &mut App) -> TestContracts {
        let env = mock_env();

        let pauser = PauserContract::new(app, &env, None);
        let strategy_manager = StrategyManagerContract::new(app, &env, None);
        let delegation_manager = DelegationManagerContract::new(app, &env, None);

        let owner = app.api().addr_make("owner");
        let not_routed = app.api().addr_make("not_routed");

        strategy_manager
            .execute(
                app,
                &owner,
                &ExecuteMsg::SetRouting {
                    delegation_manager: delegation_manager.addr.to_string(),
                    slash_manager: not_routed.to_string(),
                },
            )
            .unwrap();

        delegation_manager
            .execute(
                app,
                &owner,
                &bvs_delegation_manager::msg::ExecuteMsg::SetRouting {
                    strategy_manager: strategy_manager.addr.to_string(),
                    slash_manager: not_routed.to_string(),
                },
            )
            .unwrap();

        Self {
            strategy_manager,
            delegation_manager,
            pauser,
        }
    }

    fn instantiate_base(&self, app: &mut App) -> (Cw20TokenContract, StrategyBaseContract) {
        let env = mock_env();
        let owner = app.api().addr_make("owner");
        let token = bvs_strategy_base::testing::Cw20TokenContract::new(app, &env, None);
        let strategy_base = bvs_strategy_base::testing::StrategyBaseContract::new(
            app,
            &env,
            Some(StrategyBaseInstantiateMsg {
                pauser: self.pauser.addr().to_string(),
                owner: owner.to_string(),
                strategy_manager: self.strategy_manager.addr().to_string(),
                underlying_token: token.addr().to_string(),
            }),
        );

        (token, strategy_base)
    }
}

#[test]
fn test_add_strategy() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);
    let (_, strategy_base) = tc.instantiate_base(app);

    let owner = app.api().addr_make("owner");
    let TestContracts {
        strategy_manager, ..
    } = &tc;

    let res = strategy_manager
        .execute(
            app,
            &owner,
            &ExecuteMsg::AddStrategy {
                strategy: strategy_base.addr().to_string(),
                whitelisted: true,
            },
        )
        .unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.as_str()),
            Event::new("wasm-StrategyUpdated")
                .add_attribute("_contract_address", strategy_manager.addr.as_str())
                .add_attribute("strategy", strategy_base.addr().as_str())
                .add_attribute("whitelisted", "true"),
        ]
    );

    let IsStrategyWhitelistedResponse(whitelisted) = strategy_manager
        .query(
            &app,
            &QueryMsg::IsStrategyWhitelisted(strategy_base.addr.to_string()),
        )
        .unwrap();
    assert_eq!(whitelisted, true);
}

#[test]
fn test_update_strategy() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);
    let (_, strategy_base) = tc.instantiate_base(app);
    let TestContracts {
        strategy_manager, ..
    } = &tc;
    let owner = app.api().addr_make("owner");

    strategy_manager
        .execute(
            app,
            &owner,
            &ExecuteMsg::AddStrategy {
                strategy: strategy_base.addr().to_string(),
                whitelisted: true,
            },
        )
        .unwrap();

    let query_msg = QueryMsg::IsStrategyWhitelisted(strategy_base.addr.to_string());
    let IsStrategyWhitelistedResponse(whitelisted) =
        strategy_manager.query(&app, &query_msg).unwrap();
    assert_eq!(whitelisted, true);

    let res = strategy_manager
        .execute(
            app,
            &owner,
            &ExecuteMsg::UpdateStrategy {
                strategy: strategy_base.addr().to_string(),
                whitelisted: false,
            },
        )
        .unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.as_str()),
            Event::new("wasm-StrategyUpdated")
                .add_attribute("_contract_address", strategy_manager.addr.as_str())
                .add_attribute("strategy", strategy_base.addr().as_str())
                .add_attribute("whitelisted", "false"),
        ]
    );
    let IsStrategyWhitelistedResponse(whitelisted) =
        strategy_manager.query(&app, &query_msg).unwrap();
    assert_eq!(whitelisted, false);

    let res = strategy_manager
        .execute(
            app,
            &owner,
            &ExecuteMsg::UpdateStrategy {
                strategy: strategy_base.addr().to_string(),
                whitelisted: true,
            },
        )
        .unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.as_str()),
            Event::new("wasm-StrategyUpdated")
                .add_attribute("_contract_address", strategy_manager.addr.as_str())
                .add_attribute("strategy", strategy_base.addr().as_str())
                .add_attribute("whitelisted", "true"),
        ]
    );

    let IsStrategyWhitelistedResponse(whitelisted) =
        strategy_manager.query(&app, &query_msg).unwrap();
    assert_eq!(whitelisted, true);
}

#[test]
fn test_update_strategy_unauthorized() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);
    let TestContracts {
        strategy_manager, ..
    } = &tc;
    let owner = app.api().addr_make("owner");

    {
        let (_, strategy_base) = tc.instantiate_base(app);
        strategy_manager
            .execute(
                app,
                &owner,
                &ExecuteMsg::AddStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: true,
                },
            )
            .unwrap();

        let not_owner = app.api().addr_make("not_owner");
        let error = strategy_manager
            .execute(
                app,
                &not_owner,
                &ExecuteMsg::UpdateStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: false,
                },
            )
            .unwrap_err();

        assert_eq!(
            error.root_cause().to_string(),
            ContractError::Unauthorized {}.to_string()
        );

        let IsStrategyWhitelistedResponse(whitelisted) = strategy_manager
            .query(
                &app,
                &QueryMsg::IsStrategyWhitelisted(strategy_base.addr.to_string()),
            )
            .unwrap();
        assert_eq!(whitelisted, true);
    }

    {
        let (_, strategy_base) = tc.instantiate_base(app);
        strategy_manager
            .execute(
                app,
                &owner,
                &ExecuteMsg::AddStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: false,
                },
            )
            .unwrap();

        let not_owner = app.api().addr_make("not_owner");
        let error = strategy_manager
            .execute(
                app,
                &not_owner,
                &ExecuteMsg::UpdateStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: true,
                },
            )
            .unwrap_err();

        assert_eq!(
            error.root_cause().to_string(),
            ContractError::Unauthorized {}.to_string()
        );

        let IsStrategyWhitelistedResponse(whitelisted) = strategy_manager
            .query(
                &app,
                &QueryMsg::IsStrategyWhitelisted(strategy_base.addr.to_string()),
            )
            .unwrap();
        assert_eq!(whitelisted, false);
    }
}

#[test]
fn test_deposit_withdraw() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);
    let (token, strategy_base) = tc.instantiate_base(app);
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let TestContracts {
        strategy_manager,
        delegation_manager,
        ..
    } = &tc;

    {
        strategy_manager
            .execute(
                app,
                &owner,
                &ExecuteMsg::AddStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: true,
                },
            )
            .unwrap();

        delegation_manager
            .execute(
                app,
                &staker,
                &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
                    operator_details: bvs_delegation_manager::msg::OperatorDetails {
                        staker_opt_out_window_blocks: 0 as u64,
                    },
                    metadata_uri: "https://example.com/whitepaper.pdf".to_string(),
                },
            )
            .unwrap();
    }

    token.fund(app, &staker, 1_000_000);
    token.increase_allowance(app, &staker, &strategy_base.addr(), 1000);
    let deposit_msg = ExecuteMsg::DepositIntoStrategy {
        amount: 10u128.into(),
        strategy: strategy_base.addr().to_string(),
        token: token.addr().to_string(),
    };

    let res = strategy_manager
        .execute(app, &staker, &deposit_msg)
        .unwrap();

    assert_eq!(
        res.has_event(&Event::new("wasm-OperatorSharesIncreased")),
        true
    );

    let StakerDepositListResponse(shares) = strategy_manager
        .query(
            &app,
            &QueryMsg::StakerDepositList {
                staker: staker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        shares,
        vec![StrategyShare {
            strategy: strategy_base.addr().clone(),
            shares: Uint128::new(10),
        }]
    );

    let _res = strategy_manager
        .execute(
            app,
            &delegation_manager.addr,
            &ExecuteMsg::WithdrawSharesAsTokens {
                recipient: staker.to_string(),
                shares: 5u128.into(),
                strategy: strategy_base.addr().to_string(),
            },
        )
        .unwrap();

    let StakerDepositListResponse(shares) = strategy_manager
        .query::<StakerDepositListResponse>(
            &app,
            &QueryMsg::StakerDepositList {
                staker: staker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        shares,
        vec![StrategyShare {
            strategy: strategy_base.addr().clone(),
            shares: Uint128::new(5),
        }]
    );

    let TotalSharesResponse(shares) = strategy_base
        .query(&app, &bvs_strategy_base::msg::QueryMsg::TotalShares {})
        .unwrap();

    assert_eq!(shares, Uint128::new(5));

    let balance = token.balance(app, &staker);
    assert_eq!(balance, 999_995);
}

#[test]
fn test_add_remove_shares() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);
    let (token, strategy_base) = tc.instantiate_base(app);
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/353");
    let TestContracts {
        strategy_manager,
        delegation_manager,
        ..
    } = &tc;

    {
        strategy_manager
            .execute(
                app,
                &owner,
                &ExecuteMsg::AddStrategy {
                    strategy: strategy_base.addr().to_string(),
                    whitelisted: true,
                },
            )
            .unwrap();

        delegation_manager
            .execute(
                app,
                &staker,
                &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
                    operator_details: bvs_delegation_manager::msg::OperatorDetails {
                        staker_opt_out_window_blocks: 0 as u64,
                    },
                    metadata_uri: "https://example.com/whitepaper.pdf".to_string(),
                },
            )
            .unwrap();
    }

    token.fund(app, &staker, 1000);
    token.increase_allowance(app, &staker, &strategy_base.addr(), 1000);

    let res = strategy_manager
        .execute(
            app,
            &staker,
            &ExecuteMsg::DepositIntoStrategy {
                amount: 1000u128.into(),
                strategy: strategy_base.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    let event = Event::new("wasm-OperatorSharesIncreased");
    assert_eq!(res.has_event(&event), true);

    let StakerDepositListResponse(strategies) = strategy_manager
        .query(
            &app,
            &QueryMsg::StakerDepositList {
                staker: staker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        strategies,
        vec![StrategyShare {
            strategy: strategy_base.addr().clone(),
            shares: Uint128::new(1000),
        }]
    );

    // Called by Delegation Manager: to manage and add back shares post-withdrawal.
    // This will cause shares imbalance
    let _res = strategy_manager
        .execute(
            app,
            &delegation_manager.addr,
            &ExecuteMsg::AddShares {
                staker: staker.to_string(),
                strategy: strategy_base.addr().to_string(),
                shares: 25u128.into(),
            },
        )
        .unwrap();

    let StakerDepositListResponse(strategies) = strategy_manager
        .query(
            &app,
            &QueryMsg::StakerDepositList {
                staker: staker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        strategies,
        vec![StrategyShare {
            strategy: strategy_base.addr().clone(),
            shares: Uint128::new(1025),
        }]
    );

    let TotalSharesResponse(total_shares) = strategy_base
        .query(&app, &bvs_strategy_base::msg::QueryMsg::TotalShares {})
        .unwrap();

    assert_eq!(total_shares, Uint128::new(1000));

    // Remove some shares. Only delegation_manager can remove shares
    strategy_manager
        .execute(
            app,
            &delegation_manager.addr,
            &ExecuteMsg::RemoveShares {
                staker: staker.to_string(),
                strategy: strategy_base.addr().to_string(),
                shares: 500u128.into(),
            },
        )
        .unwrap();

    // confirm that the shares have been removed
    let StakerDepositListResponse(strategies) = strategy_manager
        .query(
            &app,
            &QueryMsg::StakerDepositList {
                staker: staker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        strategies,
        vec![StrategyShare {
            strategy: strategy_base.addr().clone(),
            shares: Uint128::new(525),
        }]
    );
}
