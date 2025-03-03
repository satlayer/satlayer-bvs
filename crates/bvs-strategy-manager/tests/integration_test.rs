use bvs_library::testing::TestingContract;
use bvs_registry::testing::RegistryContract;
use bvs_strategy_base::msg::InstantiateMsg as StrategyBaseInstantiateMsg;
use bvs_strategy_manager::msg::InstantiateMsg as StrategyManagerInstantiateMsg;
use bvs_strategy_manager::query as StrategyManagerQuery;
use bvs_testing::integration::mock_contracts::{mock_app, mock_bvs_delegation_manager};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, Uint128};
use cw_multi_test::Executor;

/// pre approve spending for x address so that strategy subsystem can spend it
/// function mostly intended for testing
/// Not suitable for production
fn approve_allowance(
    app: &mut cw_multi_test::App,
    token: &bvs_strategy_base::testing::Cw20TokenContract,
    owner: Addr,
    spender: &str,
    amount: u128,
    expires: Option<cw20::Expiration>,
) {
    app.execute_contract(
        owner.clone(),
        token.addr().clone(),
        &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
            spender: spender.to_string(),
            amount: cosmwasm_std::Uint128::new(amount),
            expires,
        },
        &[],
    )
    .unwrap();
}

#[test]
fn test_add_new_strategy() {
    let mut app = mock_app();
    let env = mock_env();

    let whitelister = app.api().addr_make("manager_whitelister");
    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
            initial_strategy_whitelister: whitelister.to_string(),
        }),
    );
    let token = bvs_strategy_base::testing::Cw20TokenContract::new(&mut app, &env, None);

    let strategy = bvs_strategy_base::testing::StrategyBaseContract::new(
        &mut app,
        &env,
        Some(StrategyBaseInstantiateMsg {
            registry: registry.addr().to_string(),
            owner: strategy_owner.to_string(),
            strategy_manager: manager.addr().to_string(),
            underlying_token: token.addr().to_string(),
        }),
    );

    let res = manager
        .execute(
            &mut app,
            &owner,
            &bvs_strategy_manager::msg::ExecuteMsg::AddNewStrategy {
                new_strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    assert_eq!(res.events[1].ty, "wasm-NewStrategyAdded");

    let query_res = manager
        .query::<StrategyManagerQuery::TokenStrategyResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::TokenStrategy {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.strategy, strategy.addr);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);
}

#[test]
fn test_blacklist_whitelist() {
    let mut app = mock_app();
    let env = mock_env();

    let whitelister = app.api().addr_make("manager_whitelister");
    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
            initial_strategy_whitelister: whitelister.to_string(),
        }),
    );
    let token = bvs_strategy_base::testing::Cw20TokenContract::new(&mut app, &env, None);

    let strategy = bvs_strategy_base::testing::StrategyBaseContract::new(
        &mut app,
        &env,
        Some(StrategyBaseInstantiateMsg {
            registry: registry.addr().to_string(),
            owner: strategy_owner.to_string(),
            strategy_manager: manager.addr().to_string(),
            underlying_token: token.addr().to_string(),
        }),
    );

    let res = manager
        .execute(
            &mut app,
            &owner,
            &bvs_strategy_manager::msg::ExecuteMsg::AddNewStrategy {
                new_strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    assert_eq!(res.events[1].ty, "wasm-NewStrategyAdded");

    let query_res = manager
        .query::<StrategyManagerQuery::TokenStrategyResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::TokenStrategy {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.strategy, strategy.addr);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);

    let res = manager
        .execute(
            &mut app,
            &whitelister,
            &bvs_strategy_manager::msg::ExecuteMsg::BlacklistTokens {
                tokens: vec![token.addr().to_string()],
            },
        )
        .unwrap();

    let query_res = manager
        .query::<StrategyManagerQuery::IsTokenBlacklistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsTokenBlacklisted {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_blacklisted, true);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, false);
}

#[test]
fn test_unauthorized_blacklist_whitelist() {
    let mut app = mock_app();
    let env = mock_env();

    let whitelister = app.api().addr_make("manager_whitelister");
    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
            initial_strategy_whitelister: whitelister.to_string(),
        }),
    );
    let token = bvs_strategy_base::testing::Cw20TokenContract::new(&mut app, &env, None);

    let strategy = bvs_strategy_base::testing::StrategyBaseContract::new(
        &mut app,
        &env,
        Some(StrategyBaseInstantiateMsg {
            registry: registry.addr().to_string(),
            owner: strategy_owner.to_string(),
            strategy_manager: manager.addr().to_string(),
            underlying_token: token.addr().to_string(),
        }),
    );

    let res = manager
        .execute(
            &mut app,
            &owner,
            &bvs_strategy_manager::msg::ExecuteMsg::AddNewStrategy {
                new_strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    assert_eq!(res.events[1].ty, "wasm-NewStrategyAdded");

    let query_res = manager
        .query::<StrategyManagerQuery::TokenStrategyResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::TokenStrategy {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.strategy, strategy.addr);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);

    let res = manager.execute(
        &mut app,
        &owner,
        &bvs_strategy_manager::msg::ExecuteMsg::BlacklistTokens {
            tokens: vec![token.addr().to_string()],
        },
    );

    assert!(res.is_err());

    let query_res = manager
        .query::<StrategyManagerQuery::IsTokenBlacklistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsTokenBlacklisted {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_blacklisted, false);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);
}

#[test]
fn test_deposit_withdraw() {
    let mut app = mock_app();
    let env = mock_env();

    let whitelister = app.api().addr_make("manager_whitelister");
    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
            initial_strategy_whitelister: whitelister.to_string(),
        }),
    );
    let token = bvs_strategy_base::testing::Cw20TokenContract::new(&mut app, &env, None);

    let strategy = bvs_strategy_base::testing::StrategyBaseContract::new(
        &mut app,
        &env,
        Some(StrategyBaseInstantiateMsg {
            registry: registry.addr().to_string(),
            owner: strategy_owner.to_string(),
            strategy_manager: manager.addr().to_string(),
            underlying_token: token.addr().to_string(),
        }),
    );

    let res = manager
        .execute(
            &mut app,
            &owner,
            &bvs_strategy_manager::msg::ExecuteMsg::AddNewStrategy {
                new_strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    assert_eq!(res.events[1].ty, "wasm-NewStrategyAdded");

    let query_res = manager
        .query::<StrategyManagerQuery::TokenStrategyResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::TokenStrategy {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.strategy, strategy.addr);

    let query_res = manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);

    // let delegation_manager = app.api().addr_make("delegation_manager");
    let slash_manager = app.api().addr_make("slash_manager");

    // some dude has a lot of token as defined in default_init of the
    // Cw20TokenContract
    let staker1 = app.api().addr_make("some_dude");

    // ideally I'd do it with trait implemented TestingContract like the other
    // But it's in parallel ticket in-progress by another team member
    // I'll fix it later but now I don't wanna be blocked by that ticket.
    // This code block is preping delegation_manager so that we can test deposit for strategy_manager
    let delegation_manager = {
        let contract = mock_bvs_delegation_manager();
        let id = app.store_code(contract);
        let addr = app
            .instantiate_contract(
                id,
                owner.clone(),
                &bvs_delegation_manager::msg::InstantiateMsg {
                    registry: registry.addr().to_string(),
                    owner: owner.to_string(),
                    min_withdrawal_delay_blocks: 0 as u64,
                    withdrawal_delay_blocks: vec![0 as u64],
                    strategies: vec![strategy.addr().to_string()],
                },
                &[],
                "bvs_delegation_manager",
                None,
            )
            .unwrap();

        app.execute_contract(
            owner.clone(),
            addr.clone(),
            &bvs_delegation_manager::msg::ExecuteMsg::SetRouting {
                slash_manager: slash_manager.to_string(),
                strategy_manager: manager.addr().to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            staker1.clone(),
            addr.clone(),
            &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
                operator_details: bvs_delegation_manager::msg::OperatorDetails {
                    staker_opt_out_window_blocks: 0 as u64,
                },
                metadata_uri: "https://example.com/whitepaper.pdf".to_string(),
            },
            &[],
        )
        .unwrap();

        addr
    };

    let res = manager
        .execute(
            &mut app,
            &owner,
            &bvs_strategy_manager::msg::ExecuteMsg::SetRouting {
                delegation_manager: delegation_manager.to_string(),
                slash_manager: slash_manager.to_string(),
            },
        )
        .unwrap();

    let balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            token.addr().clone(),
            &cw20::Cw20QueryMsg::Balance {
                address: staker1.to_string(),
            },
        )
        .unwrap();

    assert_eq!(balance.balance, cosmwasm_std::Uint128::new(1000000));

    approve_allowance(
        &mut app,
        &token,
        staker1.clone(),
        &manager.addr().to_string(),
        100,
        None,
    );

    let allowance = app
        .wrap()
        .query_wasm_smart::<cw20::AllowanceResponse>(
            token.addr().clone(),
            &cw20::Cw20QueryMsg::Allowance {
                owner: staker1.to_string(),
                spender: manager.addr().to_string(),
            },
        )
        .unwrap();

    assert_eq!(allowance.allowance, cosmwasm_std::Uint128::new(100));

    let res = manager
        .execute(
            &mut app,
            &staker1,
            &bvs_strategy_manager::msg::ExecuteMsg::DepositIntoStrategy {
                amount: 10u128.into(),
                strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    let event = Event::new("wasm-OperatorSharesIncreased");

    assert_eq!(res.has_event(&event), true);

    let query_res = manager
        .query::<StrategyManagerQuery::DepositsResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::GetDeposits {
                staker: staker1.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.shares, vec![Uint128::new(10)]);

    let res = manager
        .execute(
            &mut app,
            &delegation_manager,
            &bvs_strategy_manager::msg::ExecuteMsg::WithdrawSharesAsTokens {
                recipient: staker1.to_string(),
                shares: 5u128.into(),
                strategy: strategy.addr().to_string(),
                token: token.addr().to_string(),
            },
        )
        .unwrap();

    let query_res = manager
        .query::<StrategyManagerQuery::DepositsResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::GetDeposits {
                staker: staker1.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.shares, vec![Uint128::new(5)]);

    let query_res = strategy
        .query::<bvs_strategy_base::query::TotalSharesResponse>(
            &app,
            &bvs_strategy_base::msg::QueryMsg::GetTotalShares {},
        )
        .unwrap();

    assert_eq!(query_res.total_shares, Uint128::new(5));
}
