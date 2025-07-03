use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_base::msg::{
    Amount, AssetType, QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams,
    SetApproveProxyParams, VaultInfoResponse,
};
use bvs_vault_base::shares::QueuedWithdrawalInfo;
use bvs_vault_base::VaultError;
use bvs_vault_cw20::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_router::{msg::ExecuteMsg as RouterExecuteMsg, testing::VaultRouterContract};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, Timestamp, Uint128, Uint64};
use cw_multi_test::App;

struct TestContracts {
    pauser: PauserContract,
    registry: RegistryContract,
    router: VaultRouterContract,
    cw20: Cw20TokenContract,
    vault: VaultCw20Contract,
}

impl TestContracts {
    fn init(app: &mut App) -> TestContracts {
        let env = mock_env();

        let pauser = PauserContract::new(app, &env, None);
        let registry = RegistryContract::new(app, &env, None);
        let router = VaultRouterContract::new(app, &env, None);
        let cw20 = Cw20TokenContract::new(app, &env, None);
        let vault = VaultCw20Contract::new(app, &env, None);

        // For easy of testing, we will whitelist the router.
        let msg = bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: vault.addr.to_string(),
            whitelisted: true,
        };
        let sender = Addr::unchecked(&router.init.owner);
        router.execute(app, &sender, &msg).unwrap();

        Self {
            pauser,
            registry,
            router,
            vault,
            cw20,
        }
    }
}

#[test]
fn test_not_whitelisted() {
    let app = &mut App::default();
    let env = mock_env();

    let _ = PauserContract::new(app, &env, None);
    let _ = RegistryContract::new(app, &env, None);
    let _ = VaultRouterContract::new(app, &env, None);
    let cw20 = Cw20TokenContract::new(app, &env, None);
    let vault = VaultCw20Contract::new(app, &env, None);

    let staker = app.api().addr_make("staker");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(20),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);

    let err = vault.execute(app, &staker, &msg).unwrap_err();

    assert_eq!(err.root_cause().to_string(), "Vault is not whitelisted");
}

#[test]
fn test_not_enough_balance_deposit() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(100e15 as u128),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 50e15 as u128);

    let err = vault.execute(app, &staker, &msg).unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Overflow: Cannot Sub with given operands"
    );
}

#[test]
fn test_queue_withdrawal_to_successfully() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let token_amount = Uint128::new(999_999_999_999);

    // Fund tokens
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
    router.execute(app, &owner, &msg).unwrap();

    let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
        controller: staker.clone(),
        owner: staker.clone(),
        amount: Uint128::new(10000),
    });
    let result = vault.execute(app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
            Event::new("wasm-QueueWithdrawalTo")
                .add_attribute("_contract_address", vault.addr.to_string())
                .add_attribute("sender", staker.to_string())
                .add_attribute("owner", staker.to_string())
                .add_attribute("controller", staker.to_string())
                .add_attribute("queued_shares", "10000")
                .add_attribute("new_unlock_timestamp", "1571797519")
                .add_attribute("total_queued_shares", "10000")
        ]
    );

    let msg = QueryMsg::QueuedWithdrawal {
        controller: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(10000));
    assert_eq!(
        response.unlock_timestamp,
        Timestamp::from_nanos(1571797519879305533)
    );
}

#[test]
fn test_withdrawal_with_proxy_successfully() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker1 = app.api().addr_make("staker1");
    let staker2 = app.api().addr_make("staker2");
    let recipient = app.api().addr_make("recipient");
    let proxy = app.api().addr_make("proxy");

    let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
    router.execute(app, &owner, &msg).unwrap();

    // Staker1 deposits into vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker1.clone(),
            amount: Uint128::new(100_000_000),
        });
        cw20.increase_allowance(app, &staker1, vault.addr(), 1e9 as u128);
        cw20.fund(app, &staker1, 1e9 as u128);
        vault.execute(app, &staker1, &msg).unwrap();
    }

    // Staker2 deposits into vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker2.clone(),
            amount: Uint128::new(99_999_999),
        });
        cw20.increase_allowance(app, &staker2, vault.addr(), 1e9 as u128);
        cw20.fund(app, &staker2, 1e9 as u128);
        vault.execute(app, &staker2, &msg).unwrap();
    }

    // Staker1 approves proxy
    {
        let msg = ExecuteMsg::SetApproveProxy(SetApproveProxyParams {
            proxy: proxy.clone(),
            approve: true,
        });
        vault.execute(app, &staker1, &msg).unwrap();
    }

    // Staker2 approves proxy
    {
        let msg = ExecuteMsg::SetApproveProxy(SetApproveProxyParams {
            proxy: proxy.clone(),
            approve: true,
        });
        vault.execute(app, &staker2, &msg).unwrap();
    }

    // Proxy queues withdrawal to Staker1 (owner and controller)
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let result = vault.execute(app, &proxy, &msg).unwrap();
        assert_eq!(
            result.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
                Event::new("wasm-QueueWithdrawalTo")
                    .add_attribute("_contract_address", vault.addr.to_string())
                    .add_attribute("sender", proxy.to_string())
                    .add_attribute("owner", staker1.to_string())
                    .add_attribute("controller", staker1.to_string())
                    .add_attribute("queued_shares", "10000")
                    .add_attribute("new_unlock_timestamp", "1571797519")
                    .add_attribute("total_queued_shares", "10000")
            ]
        );
    }

    // check that Staker1 has queued shares
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker1.to_string(),
        };
        let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(10_000));
        assert_eq!(
            response.unlock_timestamp,
            Timestamp::from_nanos(1571797519879305533)
        );
    }

    // move blockchain forward
    app.update_block(|block| {
        block.height = block.height + 10;
        block.time = block.time.plus_seconds(100);
    });

    // Proxy queues withdrawal to Staker2 (owner and controller)
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(20_000),
        });
        let result = vault.execute(app, &proxy, &msg).unwrap();
        assert_eq!(
            result.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
                Event::new("wasm-QueueWithdrawalTo")
                    .add_attribute("_contract_address", vault.addr.to_string())
                    .add_attribute("sender", proxy.to_string())
                    .add_attribute("owner", staker2.to_string())
                    .add_attribute("controller", staker2.to_string())
                    .add_attribute("queued_shares", "20000")
                    .add_attribute("new_unlock_timestamp", "1571797619")
                    .add_attribute("total_queued_shares", "20000")
            ]
        );
    }

    // check that Staker2 has queued shares
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker2.to_string(),
        };
        let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(20_000));
        assert_eq!(
            response.unlock_timestamp,
            Timestamp::from_nanos(1571797619879305533)
        );
    }

    // check that Staker1 queued shares unlock time and shares are not changed
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker1.to_string(),
        };
        let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(10_000));
        assert_eq!(
            response.unlock_timestamp,
            Timestamp::from_nanos(1571797519879305533)
        );
    }

    // move blockchain forward
    app.update_block(|block| {
        block.height = block.height + 10;
        block.time = block.time.plus_seconds(100);
    });

    // Proxy redeems withdrawal to Staker1
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            recipient: staker1.clone(),
            controller: staker1.clone(),
        });

        let result = vault.execute(app, &proxy, &msg).unwrap();
        assert_eq!(
            result.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
                Event::new("wasm-RedeemWithdrawalTo")
                    .add_attribute("_contract_address", vault.addr.to_string())
                    .add_attribute("sender", proxy.to_string())
                    .add_attribute("controller", staker1.to_string())
                    .add_attribute("recipient", staker1.to_string())
                    .add_attribute("sub_shares", "10000")
                    .add_attribute("claimed_assets", "10000")
                    .add_attribute("total_shares", "199989999"),
                Event::new("execute").add_attribute("_contract_address", cw20.addr.to_string()),
                Event::new("wasm")
                    .add_attribute("_contract_address", cw20.addr.to_string())
                    .add_attribute("action", "transfer")
                    .add_attribute("from", vault.addr.to_string())
                    .add_attribute("to", staker1.to_string())
                    .add_attribute("amount", "10000")
            ]
        );

        // check that staker1 received the assets
        let balance = cw20.balance(app, &staker1);
        assert_eq!(balance, 900_010_000); // 1_000_000_000 - 100_000_000 + 10_000
    }

    // Staker2 redeems withdrawal to recipient
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            recipient: recipient.clone(),
            controller: staker2.clone(),
        });

        let result = vault.execute(app, &staker2, &msg).unwrap();
        assert_eq!(
            result.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
                Event::new("wasm-RedeemWithdrawalTo")
                    .add_attribute("_contract_address", vault.addr.to_string())
                    .add_attribute("sender", staker2.to_string())
                    .add_attribute("controller", staker2.to_string())
                    .add_attribute("recipient", recipient.to_string())
                    .add_attribute("sub_shares", "20000")
                    .add_attribute("claimed_assets", "20000")
                    .add_attribute("total_shares", "199969999"),
                Event::new("execute").add_attribute("_contract_address", cw20.addr.to_string()),
                Event::new("wasm")
                    .add_attribute("_contract_address", cw20.addr.to_string())
                    .add_attribute("action", "transfer")
                    .add_attribute("from", vault.addr.to_string())
                    .add_attribute("to", recipient.to_string())
                    .add_attribute("amount", "20000")
            ]
        );

        // check that recipient received the assets
        let balance = cw20.balance(app, &recipient);
        assert_eq!(balance, 20_000);
    }

    // check that Staker1 has no queued shares and shares are reduced
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker1.to_string(),
        };
        let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(0));
        assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));

        // check that staker1 shares are reduced
        let msg = QueryMsg::Shares {
            staker: staker1.to_string(),
        };
        let response: Uint128 = vault.query(app, &msg).unwrap();
        assert_eq!(response, Uint128::new(99_990_000)); // 100_000_000 - 10_000
    }
    // check that Staker2 has no queued shares and shares are reduced
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker2.to_string(),
        };
        let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(0));
        assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));

        // check that shares are reduced
        let query_shares = QueryMsg::Shares {
            staker: staker2.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(99_979_999)); // 99_999_999 - 20_000
    }
}

#[test]
fn test_withdrawal_with_proxy_errors() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker1 = app.api().addr_make("staker1");
    let staker2 = app.api().addr_make("staker2");
    let proxy = app.api().addr_make("proxy");
    let token_amount = Uint128::new(999_999_999_999);

    let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
    router.execute(app, &owner, &msg).unwrap();

    // Staker1 deposits into vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker1.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker1, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker1, 100e15 as u128);
        vault.execute(app, &staker1, &msg).unwrap();
    }

    // Staker2 deposits into vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker2.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker2, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker2, 100e15 as u128);
        vault.execute(app, &staker2, &msg).unwrap();
    }

    // Staker1 queue withdrawal to staker2
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let err = vault.execute(app, &staker1, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized controller".into()
            }
            .to_string()
        );
    }

    // Staker2 queue withdrawal to staker2 with staker1 as owner
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let err = vault.execute(app, &staker2, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized sender".into()
            }
            .to_string()
        );
    }

    // Staker1 approves proxy
    {
        let msg = ExecuteMsg::SetApproveProxy(SetApproveProxyParams {
            proxy: proxy.clone(),
            approve: true,
        });
        vault.execute(app, &staker1, &msg).unwrap();
    }

    // proxy queue withdrawal for staker2 with staker1 as owner
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let err = vault.execute(app, &proxy, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized controller".into()
            }
            .to_string()
        );
    }

    // proxy queue withdrawal for staker1 with staker2 as owner
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(10_000),
        });
        let err = vault.execute(app, &proxy, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized sender".into()
            }
            .to_string()
        );
    }

    // staker2 tries to reset the withdrawal lock period for staker1 by QueueWithdrawalTo staker1 as controller
    {
        // proxy queue withdrawal for staker1 with staker1 as owner and controller
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let result = vault.execute(app, &proxy, &msg);
        assert!(result.is_ok());

        // staker 2 tries to grief by queueing withdrawal to staker1 as controller
        let grief_msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(1000),
        });
        let err = vault.execute(app, &staker2, &grief_msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized controller".into()
            }
            .to_string()
        );
    }
}

#[test]
fn test_redeem_withdrawal_to_successfully() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let token_amount = Uint128::new(999_999_999_999);

    // Fund tokens
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();
    }

    // queue withdrawal to
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        router.execute(app, &owner, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        recipient: staker.clone(),
        controller: staker.clone(),
    });

    app.update_block(|block| {
        block.time = block.time.plus_seconds(115);
    });
    let response = vault.execute(app, &staker, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
            Event::new("wasm-RedeemWithdrawalTo")
                .add_attribute("_contract_address", vault.addr.to_string())
                .add_attribute("sender", staker.to_string())
                .add_attribute("controller", staker.to_string())
                .add_attribute("recipient", staker.to_string())
                .add_attribute("sub_shares", "10000")
                .add_attribute("claimed_assets", "10000")
                .add_attribute("total_shares", "999999989999"),
            Event::new("execute").add_attribute("_contract_address", cw20.addr.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.to_string())
                .add_attribute("action", "transfer")
                .add_attribute("from", vault.addr.to_string())
                .add_attribute("to", staker.to_string())
                .add_attribute("amount", "10000")
        ]
    );

    let msg = QueryMsg::QueuedWithdrawal {
        controller: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(0));
    assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));
}

#[test]
fn test_redeem_withdrawal_to_no_queued_shares_error() {
    let app = &mut App::default();
    let TestContracts { vault, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker");

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        recipient: staker.clone(),
        controller: staker.clone(),
    });

    let err = vault.execute(app, &staker, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        VaultError::Zero {
            msg: "No queued shares".into()
        }
        .to_string()
    );
}

#[test]
fn test_redeem_withdrawal_to_locked_shares_error() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let token_amount = Uint128::new(999_999_999_999);

    // Fund tokens
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();
    }

    // set withdrawal lock period
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        router.execute(app, &owner, &msg).unwrap();
    }

    // queue withdrawal to
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        recipient: staker.clone(),
        controller: staker.clone(),
    });

    let err = vault.execute(app, &staker, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        VaultError::Locked {
            msg: "The shares are locked".into()
        }
        .to_string()
    );
}

#[test]
fn test_redeem_withdrawal_future_time_to_locked_shares_error() {
    let app = &mut App::default();
    let TestContracts {
        router,
        vault,
        cw20,
        ..
    } = TestContracts::init(app);

    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let token_amount = Uint128::new(999_999_999_999);

    // Fund tokens
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: token_amount,
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();
    }

    // set withdrawal lock period
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        router.execute(app, &owner, &msg).unwrap();
    }

    // queue withdrawal to for the first time
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // queue withdrawal to for the second time
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        recipient: staker.clone(),
        controller: staker.clone(),
    });

    let err = vault.execute(app, &staker, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        VaultError::Locked {
            msg: "The shares are locked".into()
        }
        .to_string()
    );
}

#[test]
fn test_vault_info() {
    let app = &mut App::default();
    let tc = TestContracts::init(app);

    let response: VaultInfoResponse = tc.vault.query(app, &QueryMsg::VaultInfo {}).unwrap();
    assert_eq!(
        response,
        VaultInfoResponse {
            total_shares: Uint128::new(0),
            total_assets: Uint128::new(0),
            router: tc.router.addr,
            pauser: tc.pauser.addr,
            operator: app.api().addr_make("operator"),
            asset_id: format!("cosmos:cosmos-testnet-14002/cw20:{}", tc.cw20.addr).to_string(),
            asset_type: AssetType::Cw20,
            asset_reference: tc.cw20.addr.to_string(),
            contract: "crates.io:bvs-vault-cw20".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}

#[test]
fn test_system_lock_assets() {
    let app = &mut App::default();
    let TestContracts {
        router,
        cw20,
        vault,
        ..
    } = TestContracts::init(app);
    let original_deposit_amount: u128 = 100_000_000;

    let stakers = [
        app.api().addr_make("staker/1"),
        app.api().addr_make("staker/2"),
        app.api().addr_make("staker/3"),
    ];

    // setup/fund/stake tokens
    {
        for staker in stakers.iter() {
            let msg = ExecuteMsg::DepositFor(RecipientAmount {
                recipient: staker.clone(),
                amount: Uint128::new(original_deposit_amount),
            });
            cw20.increase_allowance(app, staker, vault.addr(), original_deposit_amount);
            cw20.fund(app, staker, original_deposit_amount);
            vault.execute(app, staker, &msg).unwrap();
        }
    }

    let vault_balance_pre_slash = cw20.balance(app, vault.addr());

    // positive test
    {
        let slash_amount = Uint128::from(vault_balance_pre_slash / 2);

        // don't really need to create dedicated var for this
        // but for readability
        let expected_vault_balance_post_slash = Uint128::from(vault_balance_pre_slash)
            .checked_sub(slash_amount)
            .unwrap();

        let msg = ExecuteMsg::SlashLocked(Amount(slash_amount));
        vault.execute(app, router.addr(), &msg).unwrap();

        let vault_balance = cw20.balance(app, vault.addr());
        let router_balance = cw20.balance(app, router.addr());

        // assert that the vault balance is halved
        assert_eq!(vault_balance, expected_vault_balance_post_slash.into());

        // assert that router get the slashed amount
        assert_eq!(router_balance, slash_amount.into());
    }

    // non linear ratio sanity checks
    {
        // before the slash shares to assets are mostly linear 1:1
        // Since we are slashing 50% of the assets
        // without effecting the shares
        // the shares to assets ratio should be 2:1 now
        // This is intended.
        // The propotion of how much each staker get (shares) stays the same

        for staker in stakers.iter() {
            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(app, &query_shares).unwrap();

            // shares stays the same
            assert_eq!(shares, Uint128::new(original_deposit_amount));

            let query_assets = QueryMsg::Assets {
                staker: staker.to_string(),
            };
            let assets: Uint128 = vault.query(app, &query_assets).unwrap();

            // assets should be halved
            assert_eq!(assets, Uint128::new(original_deposit_amount / 2));
        }
    }

    //negative test
    {
        // non-callable address
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount)));
        let resp = vault.execute(app, &stakers[0], &msg);
        assert!(resp.is_err());

        // larger than vault balance
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount * 3)));
        let resp = vault.execute(app, router.addr(), &msg);
        assert!(resp.is_err());

        // zero amount
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(0)));
        let resp = vault.execute(app, router.addr(), &msg);
        assert!(resp.is_err());
    }
}
