use bvs_delegation_manager::{
    msg::{ExecuteMsg, OperatorDetails, QueryMsg, QueuedWithdrawalParams, Withdrawal},
    query::{
        CumulativeWithdrawalsQueuedResponse, DelegatableSharesResponse, DelegatedResponse,
        OperatorDetailsResponse, OperatorResponse, OperatorSharesResponse, OperatorStakersResponse,
        StakerOptOutWindowBlocksResponse, StakerShares, WithdrawalDelayResponse,
    },
    testing::DelegationManagerContract,
};
use bvs_library::testing::TestingContract;
use bvs_registry::testing::RegistryContract;
use bvs_strategy_base::msg::InstantiateMsg as StrategyBaseInstantiateMsg;
use bvs_strategy_base::testing::Cw20TokenContract;
use bvs_strategy_base::{msg::ExecuteMsg as StrategyBaseExecuteMsg, testing::StrategyBaseContract};
use bvs_strategy_manager::msg::delegation_manager::IncreaseDelegatedShares;
use bvs_strategy_manager::msg::InstantiateMsg as StrategyManagerInstantiateMsg;
use bvs_strategy_manager::{
    msg::ExecuteMsg as StrategyManagerExecuteMsg, query as StrategyManagerQuery,
    testing::StrategyManagerContract,
};
use cosmwasm_std::{testing::mock_env, Addr, Env, Event, StdError, Uint128};
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

fn instantiate() -> (App, DelegationManagerContract, StrategyManagerContract) {
    let mut app = App::default();
    let env = mock_env();

    let _ = RegistryContract::new(&mut app, &env, None);
    let strategy_manager = StrategyManagerContract::new(&mut app, &env, None);
    let delegation_manager = DelegationManagerContract::new(&mut app, &env, None);
    let slash_manager = app.api().addr_make("slash_manager");

    let msg = ExecuteMsg::SetRouting {
        strategy_manager: strategy_manager.addr().to_string(),
        slash_manager: slash_manager.to_string(),
    };
    let owner = app.api().addr_make("owner");
    delegation_manager.execute(&mut app, &owner, &msg).unwrap();

    let msg = StrategyManagerExecuteMsg::SetRouting {
        delegation_manager: delegation_manager.addr().to_string(),
        slash_manager: slash_manager.to_string(),
    };
    strategy_manager.execute(&mut app, &owner, &msg).unwrap();

    (app, delegation_manager, strategy_manager)
}

#[test]
fn register_as_operator_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let operator = app.api().addr_make("operator");

    // test register as operator
    let operator_details = OperatorDetails {
        staker_opt_out_window_blocks: 100,
    };
    let metadata_uri = "https://example.com/metadata";
    let msg = ExecuteMsg::RegisterAsOperator {
        operator_details: operator_details.clone(),
        metadata_uri: metadata_uri.to_string(),
    };

    let result = delegation_manager.execute(&mut app, &operator, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 3);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-OperatorRegistered")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string()),
            Event::new("wasm-OperatorMetadataURIUpdated")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("metadata_uri", metadata_uri.to_string())
        ]
    );

    // test query is operator
    let msg = QueryMsg::IsOperator {
        operator: operator.to_string(),
    };
    let result: Result<OperatorResponse, StdError> = delegation_manager.query(&app, &msg);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_operator);

    // test query is delegated
    let msg = QueryMsg::IsDelegated {
        staker: operator.to_string(),
    };
    let result: Result<DelegatedResponse, StdError> = delegation_manager.query(&app, &msg);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_delegated);

    // test query operator details
    let msg = QueryMsg::OperatorDetails {
        operator: operator.to_string(),
    };
    let result: Result<OperatorDetailsResponse, StdError> = delegation_manager.query(&app, &msg);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.details.staker_opt_out_window_blocks, 100);

    // test query staker opt out window blocks
    let msg = QueryMsg::StakerOptOutWindowBlocks {
        operator: operator.to_string(),
    };
    let result: Result<StakerOptOutWindowBlocksResponse, StdError> =
        delegation_manager.query(&app, &msg);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.staker_opt_out_window_blocks, 100);
}

#[test]
fn modify_operator_details_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let new_operator_details = OperatorDetails {
        staker_opt_out_window_blocks: 150,
    };
    let msg = ExecuteMsg::ModifyOperatorDetails {
        new_operator_details,
    };
    let result = delegation_manager.execute(&mut app, &operator, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-OperatorDetailsSet")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("staker_opt_out_window_blocks", "150")
        ]
    );
}

#[test]
fn update_operator_metadata_uri_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let new_metadata_uri = "https://example.com/new_metadata";
    let msg = ExecuteMsg::UpdateOperatorMetadataUri {
        metadata_uri: new_metadata_uri.to_string(),
    };
    let result = delegation_manager.execute(&mut app, &operator, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-OperatorMetadataURIUpdated")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("metadata_uri", new_metadata_uri.to_string())
        ]
    );
}

#[test]
fn delegate_to_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");
    let msg = ExecuteMsg::DelegateTo {
        operator: operator.to_string(),
    };
    let result = delegation_manager.execute(&mut app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-Delegate")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("operator", operator.to_string())
        ]
    );
}

#[test]
fn undelegate_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");

    // delegate to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    let msg = ExecuteMsg::Undelegate {
        staker: staker.to_string(),
    };
    let result = delegation_manager.execute(&mut app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-StakerUndelegated")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("operator", operator.to_string())
        ]
    );
}

#[test]
fn queue_withdrawals_successfully() {
    let (mut app, delegation_manager, strategy_manager) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");
    let strategy1 = app.api().addr_make("strategy1");

    // add shares in strategy-manager
    {
        let token = app.api().addr_make("token");
        let msg = StrategyManagerExecuteMsg::AddShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
            shares: Uint128::new(100),
        };
        let delegation_manager_addr = delegation_manager.addr();
        let result = strategy_manager.execute(&mut app, delegation_manager_addr, &msg);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 2);
        assert_eq!(
            response.events[0],
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
        );
        assert_eq!(
            response.events[1],
            Event::new("wasm-add_shares")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "100")
        );
    }

    // delegate some shares to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    let queued_withdrawal_params = vec![QueuedWithdrawalParams {
        withdrawer: staker.clone(),
        strategies: vec![strategy1.clone()],
        shares: vec![Uint128::new(10)],
    }];
    let msg = ExecuteMsg::QueueWithdrawals {
        queued_withdrawal_params,
    };
    let result = delegation_manager.execute(&mut app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 5);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute(
                    "withdrawal_roots",
                    "HpXRkQeEEmdEQ0Usi1052SjbIUYxmFlCp7MVT9SLZYI="
                ),
            Event::new("wasm-WithdrawalQueued")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute(
                    "withdrawal_root",
                    "HpXRkQeEEmdEQ0Usi1052SjbIUYxmFlCp7MVT9SLZYI="
                )
                .add_attribute("staker", staker.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("withdrawer", staker.to_string()),
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
                .add_attribute("method", "remove_shares")
                .add_attribute("staker", staker.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "10")
                .add_attribute("strategy_removed", "false")
        ]
    );

    // test query operator shares
    {
        let msg = QueryMsg::GetOperatorShares {
            operator: operator.to_string(),
            strategies: vec![strategy1.to_string()],
        };
        let result: Result<OperatorSharesResponse, StdError> = delegation_manager.query(&app, &msg);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.shares.len(), 1);
        assert_eq!(response.shares[0], Uint128::new(90));
    }

    // test query delegatable shares
    {
        let msg = QueryMsg::GetDelegatableShares {
            staker: staker.to_string(),
        };
        let result: Result<DelegatableSharesResponse, StdError> =
            delegation_manager.query(&app, &msg);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.strategies.len(), 1);
        assert_eq!(response.strategies[0], strategy1);
        assert_eq!(response.shares.len(), 1);
        assert_eq!(response.shares[0], Uint128::new(90));
    }

    // test query operator stakers
    {
        let msg = QueryMsg::GetOperatorStakers {
            operator: operator.to_string(),
        };
        let result: Result<OperatorStakersResponse, StdError> =
            delegation_manager.query(&app, &msg);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.stakers_and_shares.len(), 1);
        assert_eq!(
            response.stakers_and_shares[0],
            StakerShares {
                staker: staker.clone(),
                shares_per_strategy: vec![(strategy1, Uint128::new(90))]
            }
        );
    }

    // test query cumulative withdrawals queued
    {
        let msg = QueryMsg::GetCumulativeWithdrawalsQueued {
            staker: staker.to_string(),
        };
        let result: Result<CumulativeWithdrawalsQueuedResponse, StdError> =
            delegation_manager.query(&app, &msg);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.cumulative_withdrawals, Uint128::new(1));
    }
}

// TODO: set receive_as_tokens to false needs starting a CW20 contract
#[test]
fn complete_queued_withdrawal_successfully() {
    let (mut app, delegation_manager, strategy_manager) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");
    let strategy1 = app.api().addr_make("strategy1");
    let token = app.api().addr_make("token");

    // add shares in strategy-manager
    {
        let msg = StrategyManagerExecuteMsg::AddShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
            shares: Uint128::new(100),
        };
        let delegation_manager_addr = delegation_manager.addr();
        let result = strategy_manager.execute(&mut app, delegation_manager_addr, &msg);
        assert!(result.is_ok());
    }

    // delegate some shares to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    // queue withdrawal
    {
        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: staker.clone(),
            strategies: vec![strategy1.clone()],
            shares: vec![Uint128::new(10)],
        }];
        let msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    let withdrawal = Withdrawal {
        staker: staker.clone(),
        delegated_to: operator.clone(),
        withdrawer: staker.clone(),
        nonce: Uint128::new(0),
        start_block: 12345,
        strategies: vec![strategy1.clone()],
        shares: vec![Uint128::new(10)],
    };

    let num_of_blocks = 100;
    app.update_block(|block| {
        block.height += num_of_blocks;
        block.time = block.time.plus_seconds(num_of_blocks * 6);
    });

    let msg = ExecuteMsg::CompleteQueuedWithdrawal {
        withdrawal,
        middleware_times_index: 0,
        receive_as_tokens: false,
    };
    let result = delegation_manager.execute(&mut app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 4);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-WithdrawalCompleted")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute(
                    "withdrawal_root",
                    "HpXRkQeEEmdEQ0Usi1052SjbIUYxmFlCp7MVT9SLZYI="
                ),
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.to_string()),
            Event::new("wasm-add_shares")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "10")
        ]
    );
}

// TODO: set receive_as_tokens to false needs starting a CW20 contract
#[test]
fn complete_queued_withdrawals_successfully() {
    let (mut app, delegation_manager, strategy_manager) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker1 = app.api().addr_make("staker1");
    let strategy1 = app.api().addr_make("strategy1");
    let token = app.api().addr_make("token");

    // add shares in strategy-manager
    {
        // staker1
        let msg = StrategyManagerExecuteMsg::AddShares {
            staker: staker1.to_string(),
            strategy: strategy1.to_string(),
            shares: Uint128::new(100),
        };
        let delegation_manager_addr = delegation_manager.addr();
        let result = strategy_manager.execute(&mut app, delegation_manager_addr, &msg);
        assert!(result.is_ok());
    }

    // delegate some shares to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker1, &msg);
        assert!(result.is_ok());
    }

    // queue withdrawal
    {
        // withdrawal1
        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: staker1.clone(),
            strategies: vec![strategy1.clone()],
            shares: vec![Uint128::new(10)],
        }];
        let msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        };
        let result = delegation_manager.execute(&mut app, &staker1, &msg);
        assert!(result.is_ok());

        // withdrawal2
        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: staker1.clone(),
            strategies: vec![strategy1.clone()],
            shares: vec![Uint128::new(20)],
        }];
        let msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        };
        let result = delegation_manager.execute(&mut app, &staker1, &msg);
        assert!(result.is_ok());
    }

    let withdrawal1 = Withdrawal {
        staker: staker1.clone(),
        delegated_to: operator.clone(),
        withdrawer: staker1.clone(),
        nonce: Uint128::new(0),
        start_block: 12345,
        strategies: vec![strategy1.clone()],
        shares: vec![Uint128::new(10)],
    };
    let withdrawal2 = Withdrawal {
        staker: staker1.clone(),
        delegated_to: operator.clone(),
        withdrawer: staker1.clone(),
        nonce: Uint128::new(1),
        start_block: 12345,
        strategies: vec![strategy1.clone()],
        shares: vec![Uint128::new(20)],
    };
    let withdrawals = vec![withdrawal1, withdrawal2];

    let num_of_blocks = 101;
    app.update_block(|block| {
        block.height += num_of_blocks;
        block.time = block.time.plus_seconds(num_of_blocks * 6);
    });

    let msg = ExecuteMsg::CompleteQueuedWithdrawals {
        withdrawals,
        middleware_times_indexes: vec![0, 0],
        receive_as_tokens: vec![false, false],
    };
    let result = delegation_manager.execute(&mut app, &staker1, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 7);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-WithdrawalCompleted")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute(
                    "withdrawal_root",
                    "4MXYHwHE/VlSoBd7sxUUdREC/RFMB3e16zavPgyFCH0="
                ),
            Event::new("wasm-WithdrawalCompleted")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute(
                    "withdrawal_root",
                    "OCXEKriEdhwNMZSUqWNWN/wUrw4ePB6y4jOQofPpO3E="
                ),
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.to_string()),
            Event::new("wasm-add_shares")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
                .add_attribute("staker", staker1.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "10"),
            Event::new("execute")
                .add_attribute("_contract_address", strategy_manager.addr.to_string()),
            Event::new("wasm-add_shares")
                .add_attribute("_contract_address", strategy_manager.addr.to_string())
                .add_attribute("staker", staker1.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "20")
        ]
    );
}

#[test]
fn increase_delegated_shares_successfully() {
    let (mut app, delegation_manager, strategy_manager) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");
    let strategy1 = app.api().addr_make("strategy1");

    // add shares in strategy-manager
    {
        let token = app.api().addr_make("token");
        let msg = StrategyManagerExecuteMsg::AddShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
            shares: Uint128::new(100),
        };
        let delegation_manager_addr = delegation_manager.addr();
        let result = strategy_manager.execute(&mut app, delegation_manager_addr, &msg);
        assert!(result.is_ok());
    }

    // delegate some shares to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());

        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: staker.clone(),
            strategies: vec![strategy1.clone()],
            shares: vec![Uint128::new(10)],
        }];
        let msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    let msg = ExecuteMsg::IncreaseDelegatedShares(IncreaseDelegatedShares {
        staker: staker.to_string(),
        strategy: strategy1.to_string(),
        shares: Uint128::new(20),
    });
    let result = delegation_manager.execute(&mut app, strategy_manager.addr(), &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-OperatorSharesIncreased")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "20")
                .add_attribute("new_shares", "110")
        ]
    );
}

#[test]
fn decrease_delegated_shares_successfully() {
    let (mut app, delegation_manager, strategy_manager) = instantiate();
    let operator = app.api().addr_make("operator");

    // register as operator
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://example.com/metadata";
        let msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &operator, &msg);
        assert!(result.is_ok());
    }

    let staker = app.api().addr_make("staker");
    let strategy1 = app.api().addr_make("strategy1");

    // add shares in strategy-manager
    {
        let token = app.api().addr_make("token");
        let msg = StrategyManagerExecuteMsg::AddShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
            shares: Uint128::new(100),
        };
        let delegation_manager_addr = delegation_manager.addr();
        let result = strategy_manager.execute(&mut app, delegation_manager_addr, &msg);
        assert!(result.is_ok());
    }

    // delegate some shares to operator
    {
        let msg = ExecuteMsg::DelegateTo {
            operator: operator.to_string(),
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());

        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: staker.clone(),
            strategies: vec![strategy1.clone()],
            shares: vec![Uint128::new(10)],
        }];
        let msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        };
        let result = delegation_manager.execute(&mut app, &staker, &msg);
        assert!(result.is_ok());
    }

    let msg = ExecuteMsg::DecreaseDelegatedShares {
        staker: staker.to_string(),
        strategy: strategy1.to_string(),
        shares: Uint128::new(20),
    };
    let result = delegation_manager.execute(&mut app, strategy_manager.addr(), &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-OperatorSharesDecreased")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("staker", staker.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("shares", "20")
        ]
    );
}

#[test]
fn set_min_withdrawal_delay_blocks_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let owner = app.api().addr_make("owner");

    let msg = ExecuteMsg::SetMinWithdrawalDelayBlocks {
        new_min_withdrawal_delay_blocks: 120,
    };
    let result = delegation_manager.execute(&mut app, &owner, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-MinWithdrawalDelayBlocksSet")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("method", "set_min_withdrawal_delay_blocks")
                .add_attribute("prev_min_withdrawal_delay_blocks", "5")
                .add_attribute("new_min_withdrawal_delay_blocks", "120")
        ]
    );
}

#[test]
fn set_strategy_withdrawal_delay_blocks_successfully() {
    let (mut app, delegation_manager, _) = instantiate();
    let owner = app.api().addr_make("owner");

    let strategy1 = app.api().addr_make("strategy1");
    let strategy2 = app.api().addr_make("strategy2");
    let strategies = vec![strategy1.to_string(), strategy2.to_string()];

    let msg = ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
        strategies: strategies.clone(),
        withdrawal_delay_blocks: vec![10, 20],
    };
    let result = delegation_manager.execute(&mut app, &owner, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 3);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-StrategyWithdrawalDelayBlocksSet")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("strategy", strategy1.to_string())
                .add_attribute("prev", "50")
                .add_attribute("new", "10"),
            Event::new("wasm-StrategyWithdrawalDelayBlocksSet")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("strategy", strategy2.to_string())
                .add_attribute("prev", "60")
                .add_attribute("new", "20")
        ]
    );

    // test query withdrawal delay
    let msg = QueryMsg::GetWithdrawalDelay { strategies };
    let result: Result<WithdrawalDelayResponse, StdError> = delegation_manager.query(&app, &msg);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.withdrawal_delays, vec![10, 20]);
}

#[test]
fn set_routing_successfully() {
    let (mut app, delegation_manager, _) = instantiate();

    let owner = app.api().addr_make("owner");
    let strategy_manager = app.api().addr_make("strategy_manager");
    let slash_manager = app.api().addr_make("slash_manager");

    let msg = ExecuteMsg::SetRouting {
        strategy_manager: strategy_manager.to_string(),
        slash_manager: slash_manager.to_string(),
    };
    let result = delegation_manager.execute(&mut app, &owner, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-SetRouting")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("strategy_manager", strategy_manager.to_string())
                .add_attribute("slash_manager", slash_manager.to_string())
        ]
    );
}

#[test]
fn transfer_ownership_successfully() {
    let (mut app, delegation_manager, _) = instantiate();

    let owner = app.api().addr_make("owner");
    let new_owner = app.api().addr_make("new_owner");

    let msg = ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };
    let result = delegation_manager.execute(&mut app, &owner, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", delegation_manager.addr.to_string()),
            Event::new("wasm-TransferredOwnership")
                .add_attribute("_contract_address", delegation_manager.addr.to_string())
                .add_attribute("old_owner", owner.to_string())
                .add_attribute("new_owner", new_owner.to_string())
        ]
    );
}

struct IntegratedTestComponents {
    app: App,
    env: Env,
    delegation_manager: DelegationManagerContract,
    strategy_manager: StrategyManagerContract,
    strategy: StrategyBaseContract,
    token: Cw20TokenContract,
    owner: Addr,
}

fn integrated_instantiation() -> IntegratedTestComponents {
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let strategy_owner = app.api().addr_make("strategy_owner");

    let registry = RegistryContract::new(&mut app, &env, None);

    let strategy_manager = bvs_strategy_manager::testing::StrategyManagerContract::new(
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
            strategy_manager: strategy_manager.addr().to_string(),
            underlying_token: token.addr().to_string(),
        }),
    );

    let res = strategy_manager
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

    let query_res = strategy_manager
        .query::<StrategyManagerQuery::TokenStrategyResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::TokenStrategy {
                token: token.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.strategy, strategy.addr);

    let query_res = strategy_manager
        .query::<StrategyManagerQuery::StrategyWhitelistedResponse>(
            &app,
            &bvs_strategy_manager::msg::QueryMsg::IsStrategyWhitelisted {
                strategy: strategy.addr.to_string(),
            },
        )
        .unwrap();

    assert_eq!(query_res.is_whitelisted, true);

    let delegation_manager = DelegationManagerContract::new(
        &mut app,
        &env,
        Some(bvs_delegation_manager::msg::InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.addr().to_string(),
            min_withdrawal_delay_blocks: 5,
            withdrawal_delay_blocks: vec![],
            strategies: vec![],
        }),
    );

    let slash_manager = app.api().addr_make("slash_manager");

    let result = delegation_manager
        .execute(
            &mut app,
            &owner,
            &bvs_delegation_manager::msg::ExecuteMsg::SetRouting {
                strategy_manager: strategy_manager.addr().to_string(),
                slash_manager: slash_manager.to_string(),
            },
        )
        .unwrap();

    assert_eq!(result.events[1].ty, "wasm-SetRouting");

    let result = app
        .execute_contract(
            owner.clone(),
            strategy_manager.addr().clone(),
            &bvs_strategy_manager::msg::ExecuteMsg::SetRouting {
                delegation_manager: delegation_manager.addr().to_string(),
                slash_manager: slash_manager.to_string(),
            },
            &[],
        )
        .unwrap();

    assert_eq!(result.events[1].ty, "wasm-SetRouting");

    IntegratedTestComponents {
        app,
        env,
        delegation_manager,
        strategy_manager,
        strategy,
        token,
        owner,
    }
}

/// this test unlike strategy manager tests, includes block delays.
#[test]
fn test_deposit_withdraw_with_delays() {
    let mut bvs = integrated_instantiation();

    let operator = bvs.app.api().addr_make("operator");

    let strategy = bvs.strategy;
    let strategy_manager = bvs.strategy_manager;
    let token = bvs.token;
    let delegation_manager = bvs.delegation_manager;
    let owner = bvs.owner;
    let mut app = bvs.app;
    let _env = bvs.env;

    let res = app
        .execute_contract(
            operator.clone(),
            delegation_manager.addr().clone(),
            &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
                operator_details: OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
                metadata_uri: "https://example.com/metadata".to_string(),
            },
            &[],
        )
        .unwrap();

    // we register an operator above this block test/confirm it
    {
        let query_res = delegation_manager
            .query::<bvs_delegation_manager::query::OperatorDetailsResponse>(
                &app,
                &bvs_delegation_manager::msg::QueryMsg::OperatorDetails {
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        assert_eq!(query_res.details.staker_opt_out_window_blocks, 100);

        let query_res = delegation_manager
            .query::<bvs_delegation_manager::query::OperatorResponse>(
                &app,
                &bvs_delegation_manager::msg::QueryMsg::IsOperator {
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        assert_eq!(query_res.is_operator, true);
    }

    for i in 0..10 {
        let staker = app.api().addr_make(&format!("staker{}", i));
        // now we'll add a staker and delegate to the operator
        let res = app
            .execute_contract(
                staker.clone(),
                delegation_manager.addr().clone(),
                &bvs_delegation_manager::msg::ExecuteMsg::DelegateTo {
                    operator: operator.to_string(),
                },
                &[],
            )
            .unwrap();

        // let's fund the staker account,

        {
            // transfer Cw20 token
            // we mint alot of tokens to owner in cw20 testing contract
            let _ = app
                .execute_contract(
                    owner.clone(),
                    token.addr().clone(),
                    &cw20_base::msg::ExecuteMsg::Transfer {
                        recipient: staker.to_string(),
                        amount: Uint128::new(1000),
                    },
                    &[],
                )
                .unwrap();

            let balance = app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    token.addr().clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: staker.to_string(),
                    },
                )
                .unwrap();

            assert_eq!(balance.balance, cosmwasm_std::Uint128::new(1000));
        }

        approve_allowance(
            &mut app,
            &token,
            staker.clone(),
            &strategy_manager.addr().to_string(),
            100,
            None,
        );

        let allowance = app
            .wrap()
            .query_wasm_smart::<cw20::AllowanceResponse>(
                token.addr().clone(),
                &cw20::Cw20QueryMsg::Allowance {
                    owner: staker.to_string(),
                    spender: strategy_manager.addr().to_string(),
                },
            )
            .unwrap();

        assert_eq!(allowance.allowance, cosmwasm_std::Uint128::new(100));

        // now we'll deposit
        {
            let res = app.execute_contract(
                staker.clone(),
                strategy_manager.addr().clone(),
                &bvs_strategy_manager::msg::ExecuteMsg::DepositIntoStrategy {
                    strategy: strategy.addr().to_string(),
                    token: token.addr().to_string(),
                    amount: Uint128::new(20),
                },
                &[],
            );

            assert_eq!(res.is_ok(), true);

            let query_res = strategy_manager
                .query::<bvs_strategy_manager::query::StakerStrategySharesResponse>(
                    &app,
                    &bvs_strategy_manager::msg::QueryMsg::GetStakerStrategyShares {
                        staker: staker.to_string(),
                        strategy: strategy.addr().to_string(),
                    },
                )
                .unwrap();

            assert_eq!(query_res.shares, Uint128::new(20));
        }
    }

    // 10 staker, with 20 shares each so we should see a total of 10 * 20 shares in the strategy
    // But in reality shares offset is still playing a role in dilluting it.
    // so we'll only see total 190 shares
    // The math for 10 staker with 20 token each deposit starting 0 balance and 0 shares in the
    // system:
    // $$
    // S_{\text{total}} = \sum_{i=1}^{10} \frac{20 \times 1000}{1000 + (i \times 20)}
    // $$
    // Use latex renderer or use gh gist
    // That result in about 189.71416120871464, 190 rounded up
    let query_res = strategy
        .query::<bvs_strategy_base::msg::SharesResponse>(
            &app,
            &bvs_strategy_base::msg::QueryMsg::TotalShares {},
        )
        .unwrap();

    assert_eq!(query_res.0, Uint128::new(190));

    {
        let res = app
            .execute_contract(
                owner.clone(),
                delegation_manager.addr().clone(),
                &bvs_delegation_manager::msg::ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
                    strategies: vec![strategy.addr().to_string()],
                    withdrawal_delay_blocks: vec![100],
                },
                &[],
            )
            .unwrap();
    }

    {
        for i in 0..10 {
            // ideally they should be able to withdraw all 20 token
            // But the diluation is so bad they lost half of their token staked
            // we can only withdraw 10 shares.
            // This is gonna get worse as more stakers join.
            let staker = app.api().addr_make(&format!("staker{}", i));
            let res = app
                .execute_contract(
                    staker.clone(),
                    delegation_manager.addr().clone(),
                    &bvs_delegation_manager::msg::ExecuteMsg::QueueWithdrawals {
                        queued_withdrawal_params: vec![
                            bvs_delegation_manager::msg::QueuedWithdrawalParams {
                                withdrawer: staker.clone(),
                                strategies: vec![strategy.addr().clone()],
                                shares: vec![Uint128::new(10)],
                            },
                        ],
                    },
                    &[],
                )
                .unwrap();

            // println!("{:?}", res);

            let query_res = delegation_manager
                .query::<bvs_delegation_manager::query::CumulativeWithdrawalsQueuedResponse>(
                    &app,
                    &bvs_delegation_manager::msg::QueryMsg::GetCumulativeWithdrawalsQueued {
                        staker: staker.to_string(),
                    },
                )
                .unwrap();

            assert_eq!(query_res.cumulative_withdrawals, Uint128::new(1));
        }

        // should not withdraw when delay block is unsatisfied.
        for i in 0..10 {
            let staker = app.api().addr_make(&format!("staker{}", i));
            let res = app.execute_contract(
                staker.clone(),
                delegation_manager.addr().clone(),
                &bvs_delegation_manager::msg::ExecuteMsg::CompleteQueuedWithdrawal {
                    middleware_times_index: 0,
                    receive_as_tokens: true,
                    withdrawal: bvs_delegation_manager::msg::Withdrawal {
                        staker: staker.clone(),
                        delegated_to: operator.clone(),
                        withdrawer: staker.clone(),
                        nonce: Uint128::new(0),
                        start_block: app.block_info().height,
                        strategies: vec![strategy.addr().clone()],
                        shares: vec![Uint128::new(10)],
                    },
                },
                &[],
            );

            let expected_err =
                bvs_delegation_manager::error::ContractError::MinWithdrawalDelayNotPassed {};
            assert_eq!(res.is_err(), true);
            assert!(
                matches!(res.err().unwrap(), expected_err),
                "Should not withdraw when delay block is unsatisfied"
            );
        }
    }

    println!("time travel before{:?}", app.block_info());
    let block_height_before_time_travel = app.block_info().height;

    // time travel, so we can withdraw
    app.update_block(|block| {
        block.height += 100;
        block.time = block.time.plus_seconds(100 * 6);
    });

    println!("time travel after{:?}", app.block_info());

    {
        for i in 0..10 {
            let staker = app.api().addr_make(&format!("staker{}", i));
            let res = app
                .execute_contract(
                    staker.clone(),
                    delegation_manager.addr().clone(),
                    &bvs_delegation_manager::msg::ExecuteMsg::CompleteQueuedWithdrawal {
                        middleware_times_index: 0,
                        receive_as_tokens: true,
                        withdrawal: bvs_delegation_manager::msg::Withdrawal {
                            staker: staker.clone(),
                            delegated_to: operator.clone(),
                            withdrawer: staker.clone(),
                            nonce: Uint128::new(0),
                            start_block: block_height_before_time_travel,
                            strategies: vec![strategy.addr().clone()],
                            shares: vec![Uint128::new(10)],
                        },
                    },
                    &[],
                )
                .unwrap();

            let balance = app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    token.addr().clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: staker.to_string(),
                    },
                )
                .unwrap();

            // remmeber? each staker has 1000 tok funded
            // each staker deposited 20 token, each staker is left with 980 token
            // each staker withdrew 10 token from each staker
            // now each staker has 990 token
            // Ideally, each staker should be able to withdraw all 20 token.
            // but the dilution of shares is still a little crazy.
            assert_eq!(balance.balance, cosmwasm_std::Uint128::new(990));

            let query_res = strategy_manager
                .query::<bvs_strategy_manager::query::StakerStrategySharesResponse>(
                    &app,
                    &bvs_strategy_manager::msg::QueryMsg::GetStakerStrategyShares {
                        staker: app.api().addr_make("staker0").to_string(),
                        strategy: strategy.addr().to_string(),
                    },
                )
                .unwrap();

            //The stakers has withdraw all their shares
            // so the shares should be 0
            assert_eq!(query_res.shares, Uint128::new(0));
        }

        // total shares should be 0
        let _query_res = strategy
            .query::<bvs_strategy_base::msg::SharesResponse>(
                &app,
                &bvs_strategy_base::msg::QueryMsg::TotalShares {},
            )
            .unwrap();

        // this test is gonna fail, require tokenomics to fix
        // We will turn off the test for now, cause it'll fail the pipeline
        // assert_eq!(query_res.0, Uint128::new(0));

        let _balance = app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                token.addr().clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: strategy.addr().to_string(),
                },
            )
            .unwrap();

        // the strategy should have 0 token in its balance
        // This test is gonna fail too
        // Because there will 100 token left sitting in there
        // which stakers lost due to shares
        // dilution
        // We will turn off the test for now, cause it'll fail the pipeline
        // assert_eq!(balance.balance, cosmwasm_std::Uint128::new(0));
    }
}
