use bvs_library::testing::TestingContract;
use bvs_registry::testing::RegistryContract;
use bvs_strategy_base::msg::InstantiateMsg as StrategyBaseInstantiateMsg;
use bvs_strategy_manager::msg::InstantiateMsg as StrategyManagerInstantiateMsg;
use bvs_strategy_manager::query as StrategyManagerQuery;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, Uint128};
use cw_multi_test::{App, Executor};

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
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
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
fn test_whitelist() {
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
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
fn test_unauthorized_whitelist() {
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
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
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
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

    let slash_manager = app.api().addr_make("slash_manager");

    // we need to fund this account
    let staker1 = app.api().addr_make("retail_staker");

    {
        // transfer Cw20 token
        // we mint alot of tokens to owner in cw20 testing contract
        let owner = app.api().addr_make("owner");
        let _ = app
            .execute_contract(
                owner,
                token.addr().clone(),
                &cw20_base::msg::ExecuteMsg::Transfer {
                    recipient: staker1.to_string(),
                    amount: Uint128::new(1000),
                },
                &[],
            )
            .unwrap();
    }

    let delegation_manager = {
        let owner = app.api().addr_make("owner");
        let contract =
            bvs_delegation_manager::testing::DelegationManagerContract::new(&mut app, &env, None);

        let addr = contract.addr().clone();

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

    let _res = manager
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

    assert_eq!(balance.balance, cosmwasm_std::Uint128::new(1000));

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

    let _res = manager
        .execute(
            &mut app,
            &delegation_manager,
            &bvs_strategy_manager::msg::ExecuteMsg::WithdrawSharesAsTokens {
                recipient: staker1.to_string(),
                shares: 5u128.into(),
                strategy: strategy.addr().to_string(),
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
        .query::<bvs_strategy_base::msg::TotalSharesResponse>(
            &app,
            &bvs_strategy_base::msg::QueryMsg::TotalShares {},
        )
        .unwrap();

    assert_eq!(query_res.0, Uint128::new(4));
}

#[test]
fn test_add_remove_shares() {
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("mangaer_owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
        &mut app,
        &env,
        Some(StrategyManagerInstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
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

    // we need to fund this account
    let staker1 = app.api().addr_make("retail_staker");

    {
        // transfer Cw20 token
        // we mint alot of tokens to owner in cw20 testing contract
        let owner = app.api().addr_make("owner");
        let _ = app
            .execute_contract(
                owner,
                token.addr().clone(),
                &cw20_base::msg::ExecuteMsg::Transfer {
                    recipient: staker1.to_string(),
                    amount: Uint128::new(1000),
                },
                &[],
            )
            .unwrap();
    }

    let delegation_manager = {
        let owner = app.api().addr_make("owner");
        let contract =
            bvs_delegation_manager::testing::DelegationManagerContract::new(&mut app, &env, None);

        let addr = contract.addr().clone();

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

    let _res = manager
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

    assert_eq!(balance.balance, cosmwasm_std::Uint128::new(1000));

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

    // according to the current implementation, the addshres function will add the shares to the
    // existing shares but does not add to the total shares of the strategy
    // I'm not particularly sure whether this is the intended behaviour
    // Same goes for the remove shares function
    let _res = manager
        .execute(
            &mut app,
            &delegation_manager,
            &bvs_strategy_manager::msg::ExecuteMsg::AddShares {
                staker: staker1.to_string(),
                strategy: strategy.addr().to_string(),
                shares: 25u128.into(),
            },
        )
        .unwrap();

    // let's check the shares
    let query_res = manager
        .query::<StrategyManagerQuery::DepositsResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::GetDeposits {
                staker: staker1.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.shares, vec![Uint128::new(35)]);

    let query_res = strategy
        .query::<bvs_strategy_base::msg::TotalSharesResponse>(
            &app,
            &bvs_strategy_base::msg::QueryMsg::TotalShares {},
        )
        .unwrap();

    assert_eq!(query_res.0, Uint128::new(9));

    // remove shares
    let _res = manager
        .execute(
            &mut app,
            &delegation_manager,
            &bvs_strategy_manager::msg::ExecuteMsg::RemoveShares {
                staker: staker1.to_string(),
                strategy: strategy.addr().to_string(),
                shares: 5u128.into(),
            },
        )
        .unwrap();

    // confirm that the shares have been removed
    let query_res = manager
        .query::<StrategyManagerQuery::DepositsResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::GetDeposits {
                staker: staker1.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.shares, vec![Uint128::new(30)]);
}
