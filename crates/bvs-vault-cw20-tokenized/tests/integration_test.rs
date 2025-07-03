use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_base::msg::{
    Amount, AssetType, QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams,
    SetApproveProxyParams, VaultInfoResponse,
};
use bvs_vault_base::shares::QueuedWithdrawalInfo;
use bvs_vault_base::VaultError;
use bvs_vault_cw20_tokenized::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_cw20_tokenized::testing::VaultCw20TokenizedContract;
use bvs_vault_router::{msg::ExecuteMsg as RouterExecuteMsg, testing::VaultRouterContract};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_json_binary, Addr, Event, Timestamp, Uint128, Uint64, WasmMsg};
use cw2::ContractVersion;
use cw20::BalanceResponse;
use cw_multi_test::{App, Executor};

struct TestContracts {
    pauser: PauserContract,
    registry: RegistryContract,
    router: VaultRouterContract,
    cw20: Cw20TokenContract,
    vault: VaultCw20TokenizedContract,
}

impl TestContracts {
    fn init(app: &mut App) -> TestContracts {
        let env = mock_env();

        let pauser = PauserContract::new(app, &env, None);
        let registry = RegistryContract::new(app, &env, None);
        let router = VaultRouterContract::new(app, &env, None);
        let cw20 = Cw20TokenContract::new(app, &env, None);
        let vault = VaultCw20TokenizedContract::new(app, &env, None);

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
    let vault = VaultCw20TokenizedContract::new(app, &env, None);

    let staker = app.api().addr_make("staker");
    let msg_bin = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(20),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);

    let err = vault.execute(app, &staker, &msg_bin).unwrap_err();

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

    // Staker1 deposits tokens to the vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker1.clone(),
            amount: Uint128::new(100_000_000),
        });
        cw20.increase_allowance(app, &staker1, vault.addr(), 1e9 as u128);
        cw20.fund(app, &staker1, 1e9 as u128);
        vault.execute(app, &staker1, &msg).unwrap();
    }

    // Staker2 deposits tokens to the vault
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
        controller: staker.clone(),
        recipient: staker.clone(),
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
        controller: staker.clone(),
        recipient: staker.clone(),
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
        controller: staker.clone(),
        recipient: staker.clone(),
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
        controller: staker.clone(),
        recipient: staker.clone(),
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
            contract: "crates.io:bvs-vault-cw20-tokenized".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}

#[test]
fn test_cw20_semi_compliance() {
    // tokenized vault is only semi compliance
    // Because we don't allow burning and minting at all except when stake/unstake
    let app = &mut App::default();
    let TestContracts {
        vault,
        cw20: underlying_cw20,
        ..
    } = TestContracts::init(app);

    //unlike regular cw20 token, we don't allow external minting
    // so can't fund it, we'll have to deposit some staking token to have
    // some amount of receipt token circulating
    let staker = app.api().addr_make("staker/4545");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(200),
    });
    underlying_cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    underlying_cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    // We can increase allowance like normal cw20 token.
    {
        let inner_msg = cw20_base::msg::ExecuteMsg::IncreaseAllowance {
            spender: app.api().addr_make("spender").to_string(),
            amount: Uint128::new(100),
            expires: None,
        };

        let msg = WasmMsg::Execute {
            contract_addr: vault.addr().to_string(),
            msg: to_json_binary(&inner_msg).unwrap(),
            funds: vec![],
        };

        let res = app.execute(staker.clone(), cosmwasm_std::CosmosMsg::Wasm(msg));

        assert!(res.is_ok());

        let query = cw20_base::msg::QueryMsg::Allowance {
            owner: staker.to_string(),
            spender: app.api().addr_make("spender").to_string(),
        };
        let allowance: cw20::AllowanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllowanceResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(allowance.allowance, Uint128::new(100));
    }

    // We can decrease allowance like normal cw20 token would
    {
        let inner_msg = cw20_base::msg::ExecuteMsg::DecreaseAllowance {
            spender: app.api().addr_make("spender").to_string(),
            amount: Uint128::new(50),
            expires: None,
        };
        let msg = WasmMsg::Execute {
            contract_addr: vault.addr().to_string(),
            msg: to_json_binary(&inner_msg).unwrap(),
            funds: vec![],
        };
        let res = app.execute(staker.clone(), cosmwasm_std::CosmosMsg::Wasm(msg));
        // let res = vault.execute(app, &staker, &inner_msg.into());
        assert!(res.is_ok());

        let query = cw20_base::msg::QueryMsg::Allowance {
            owner: staker.to_string(),
            spender: app.api().addr_make("spender").to_string(),
        };
        let allowance: cw20::AllowanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllowanceResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(allowance.allowance, Uint128::new(50));
    }

    // We can transfer like normal cw20 token would
    {
        let inner_msg = cw20_base::msg::ExecuteMsg::Transfer {
            recipient: app.api().addr_make("recipient").to_string(),
            amount: Uint128::new(50),
        };
        let msg = WasmMsg::Execute {
            contract_addr: vault.addr().to_string(),
            msg: to_json_binary(&inner_msg).unwrap(),
            funds: vec![],
        };
        let res = app.execute(staker.clone(), cosmwasm_std::CosmosMsg::Wasm(msg));
        assert!(res.is_ok());

        let query = cw20_base::msg::QueryMsg::Balance {
            address: app.api().addr_make("recipient").to_string(),
        };
        let balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(balance.balance, Uint128::new(50));
    }

    // We can transfer_from like normal cw20 token would
    {
        let inner_msg = cw20_base::msg::ExecuteMsg::TransferFrom {
            owner: staker.to_string(),
            recipient: app.api().addr_make("random").to_string(),
            amount: Uint128::new(25),
        };
        let msg = WasmMsg::Execute {
            contract_addr: vault.addr().to_string(),
            msg: to_json_binary(&inner_msg).unwrap(),
            funds: vec![],
        };

        // spender is the caller because we are using transfer_from
        // remember spender has been given allowance of 100 and then reduced to 50
        // in the earlier tests
        let res = app.execute(
            app.api().addr_make("spender"),
            cosmwasm_std::CosmosMsg::Wasm(msg),
        );
        assert!(res.is_ok());

        let query = cw20_base::msg::QueryMsg::Balance {
            address: app.api().addr_make("random").to_string(),
        };
        let balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(balance.balance, Uint128::new(25));
    }

    // We can query the token info like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::TokenInfo {};
        let token_info: cw20::TokenInfoResponse = app
            .wrap()
            .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(token_info.name, "Test Receipt Token".to_string());
        assert_eq!(token_info.symbol, "satTEST".to_string());
        assert_eq!(token_info.decimals, 18);

        // remember a staker staked 200 tokens in ealier tests?
        assert_eq!(token_info.total_supply, Uint128::new(200));
    }

    // We can query the balance like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::Balance {
            address: staker.to_string(),
        };
        let balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(vault.addr(), &query)
            .unwrap();

        // he staked 200 tokens
        // but due to transfer and transfer from testings only 125 receipt token left
        assert_eq!(balance.balance, Uint128::new(125));
    }

    // We can query the allowance like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::Allowance {
            owner: staker.to_string(),
            spender: app.api().addr_make("spender").to_string(),
        };
        let allowance: cw20::AllowanceResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllowanceResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(allowance.allowance, Uint128::new(25));
    }

    // We can query AllAllowance like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::AllAllowances {
            owner: staker.to_string(),
            start_after: None,
            limit: None,
        };
        let allowances: cw20::AllAllowancesResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllAllowancesResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        assert_eq!(
            allowances.allowances[0].spender,
            app.api().addr_make("spender").to_string()
        );
        assert_eq!(allowances.allowances[0].allowance, Uint128::new(25));
    }

    // We can query AllSpenderAllowances like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::AllSpenderAllowances {
            spender: app.api().addr_make("spender").to_string(),
            start_after: None,
            limit: None,
        };
        let allowances: cw20::AllSpenderAllowancesResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllSpenderAllowancesResponse>(vault.addr(), &query)
            .unwrap();
        assert_eq!(allowances.allowances.len(), 1);
        assert_eq!(allowances.allowances[0].owner, staker.to_string());
        assert_eq!(allowances.allowances[0].allowance, Uint128::new(25));
    }

    // We can query AllAccounts like normal cw20 token
    {
        let query = cw20_base::msg::QueryMsg::AllAccounts {
            start_after: None,
            limit: None,
        };
        let accounts: cw20::AllAccountsResponse = app
            .wrap()
            .query_wasm_smart::<cw20::AllAccountsResponse>(vault.addr(), &query)
            .unwrap();
        // we have 3 account
        // staker, spender that has allowance and random guy that has balance
        assert_eq!(accounts.accounts.len(), 3);
    }
}

#[test]
fn test_proper_contract_name_and_version() {
    // we need to test this to make sure
    // that cw20 base instantiation doesn't interfere with
    // the contract name and version
    // check the instantiate entry point in this vault for more details
    let app = &mut App::default();
    let TestContracts { vault, .. } = TestContracts::init(app);

    let raw = app
        .wrap()
        .query_wasm_raw(vault.addr(), b"contract_info".to_vec())
        .expect("failed to query contract_info");

    let binary = &cosmwasm_std::Binary::from(raw.unwrap());

    let contract_info: ContractVersion =
        cosmwasm_std::from_json(binary).expect("invalid contract_info format");

    assert_eq!(contract_info.contract, "crates.io:bvs-vault-cw20-tokenized",);
    assert_eq!(contract_info.version, env!("CARGO_PKG_VERSION"));
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

#[test]
fn test_deposit_transfer_then_queue_redeem_withdraw() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker/1");
    let beneficiary = app.api().addr_make("beneficiary/1");
    let initial_deposit_amount: u128 = 80_189_462_987_009_847;

    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(initial_deposit_amount),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    {
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, 19_810_537_012_990_153); // staker_unstaked_capital

        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 80_189_462_987_009_847); // initial_deposit_amount

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(80_189_462_987_009_847)); // initial_deposit_amount
    }

    // Transfer to beneficiary
    {
        let msg = ExecuteMsg::Transfer {
            recipient: beneficiary.to_string(),
            amount: Uint128::new(1000),
        };
        vault.execute(app, &staker, &msg).unwrap();

        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(app, &msg).unwrap();

        assert_eq!(resp.balance, Uint128::new(1000)); // donation_amount

        let resp: BalanceResponse = vault
            .query(
                app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        // staker donated 1000 to beneficiary
        // so his receipt token balance should be reduced
        assert_eq!(
            resp.balance,
            Uint128::new(80_189_462_987_008_847) // initial_deposit_amount - donation_amount
        );
    }

    // Fully Withdraw
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            amount: Uint128::new(initial_deposit_amount - 1000),
            owner: staker.clone(),
            controller: staker.clone(),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            amount: Uint128::new(1000),
            owner: beneficiary.clone(),
            controller: beneficiary.clone(),
        });
        vault.execute(app, &beneficiary, &msg).unwrap();
    }

    // fail premature redeem withdrawal to
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: staker.clone(),
            recipient: staker.clone(),
        });
        let res = vault.execute(app, &staker, &msg);
        assert!(res.is_err());

        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: beneficiary.clone(),
            recipient: beneficiary.clone(),
        });
        let res = vault.execute(app, &beneficiary, &msg);
        assert!(res.is_err());
    }

    // time travel
    {
        app.update_block(|block| {
            // default lock period is 7 days
            block.time = block.time.plus_seconds(604800);
        });
    }

    // redeem withdrawal to
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: staker.clone(),
            recipient: staker.clone(),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let staker_balance = cw20.balance(app, &staker);

        // initial_deposit_amount + unstaked_capital - donation_amount
        assert_eq!(staker_balance, 99_999_999_999_999_000);

        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: beneficiary.clone(),
            recipient: beneficiary.clone(),
        });
        vault.execute(app, &beneficiary, &msg).unwrap();

        let beneficiary_balance = cw20.balance(app, &beneficiary);

        assert_eq!(beneficiary_balance, 1000); // donation_amount

        let contract_balance = cw20.balance(app, vault.addr());

        assert_eq!(contract_balance, 0);
    }

    // should have 0 receipt token left
    {
        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(app, &msg).unwrap();

        assert!(resp.balance.is_zero());

        let resp: BalanceResponse = vault
            .query(
                app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        assert!(resp.balance.is_zero());
    }
}
