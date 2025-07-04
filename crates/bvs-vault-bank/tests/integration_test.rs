use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_base::error::VaultError;
use bvs_vault_base::msg::{
    Amount, AssetType, QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams,
    SetApproveProxyParams, VaultInfoResponse,
};
use bvs_vault_base::shares::QueuedWithdrawalInfo;
use bvs_vault_router::{msg::ExecuteMsg as RouterExecuteMsg, testing::VaultRouterContract};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, coins, Addr, Event, Timestamp, Uint128, Uint64};
use cw_multi_test::{App, Executor};

struct TestContracts {
    pauser: PauserContract,
    registry: RegistryContract,
    router: VaultRouterContract,
    vault: VaultBankContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });
        let env = mock_env();

        let pauser = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let router = VaultRouterContract::new(&mut app, &env, None);
        let vault = VaultBankContract::new(&mut app, &env, None);

        // For easy of testing, we will whitelist the router.
        let msg = bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: vault.addr.to_string(),
            whitelisted: true,
        };
        let sender = Addr::unchecked(&router.init.owner);
        router.execute(&mut app, &sender, &msg).unwrap();

        (
            app,
            Self {
                pauser,
                registry,
                router,
                vault,
            },
        )
    }
}

#[test]
fn test_deposit_with_non_linear_exchange_rate() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // Deposit 1_000_000 tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(1_000_000, denom))
        .expect("staker deposit failed");

    // assert staker share
    let query_shares = QueryMsg::Shares {
        staker: staker.to_string(),
    };
    let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
    assert_eq!(shares, Uint128::new(1_000_000));

    // fund contract with extra assets, so now its 2:1 instead of 1:1
    app.send_tokens(
        owner.clone(),
        tc.vault.addr().clone(),
        &coins(1_000_000, denom),
    )
    .expect("failed to fund vault");

    // Second Deposit will now use 2:1 exchange rate
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(1_000_000, denom))
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(998_000_000, denom));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(3_000_000, denom));

        // assert staker share - for same asset should receive lesser shares (due to 2:1 exchange rate)
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1_000_000 + 500_000)); // 500_000 is the new share based on 2:1 exchange rate
    }
}

#[test]
fn test_deposit_not_enough_balance() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // Deposit 1_000_000_001 tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000_001),
    });
    let err = tc
        .vault
        .execute_with_funds(app, &staker, &msg, coins(1_000_000_001, denom))
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Cannot Sub with given operands"
    );
}

#[test]
fn test_queue_withdrawal_to_successfully() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let denom = "denom";

    // Fund the staker with some initial tokens
    {
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(), // set recipient to staker
            amount: Uint128::new(115_687_654),
        });
        tc.vault
            .execute_with_funds(app, &staker, &msg, coins(115_687_654, denom))
            .unwrap();
    }

    let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
    tc.router.execute(app, &owner, &msg).unwrap();

    let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
        controller: staker.clone(),
        owner: staker.clone(),
        amount: Uint128::new(10000),
    });
    let result = tc.vault.execute(app, &staker, &msg);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
            Event::new("wasm-QueueWithdrawalTo")
                .add_attribute("_contract_address", tc.vault.addr.to_string())
                .add_attribute("sender", staker.to_string())
                .add_attribute("owner", staker.to_string())
                .add_attribute("controller", staker.to_string())
                .add_attribute("queued_shares", "10000")
                .add_attribute(
                    "new_unlock_timestamp",
                    app.block_info()
                        .time
                        .plus_seconds(100)
                        .seconds()
                        .to_string()
                )
                .add_attribute("total_queued_shares", "10000")
        ]
    );

    let msg = QueryMsg::QueuedWithdrawal {
        controller: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(10000));
    assert_eq!(
        response.unlock_timestamp,
        Timestamp::from_nanos(1571797519879305533)
    );
}

#[test]
fn test_withdrawal_with_proxy_successfully() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker1 = app.api().addr_make("staker1");
    let staker2 = app.api().addr_make("staker2");
    let proxy = app.api().addr_make("proxy");
    let recipient = app.api().addr_make("recipient");
    let denom = "denom";

    let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
    tc.router.execute(app, &owner, &msg).unwrap();

    // Staker1 deposits tokens to the vault
    {
        app.send_tokens(owner.clone(), staker1.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 100_000_000 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker1.clone(), // set recipient to staker
            amount: Uint128::new(100_000_000),
        });
        tc.vault
            .execute_with_funds(app, &staker1, &msg, coins(100_000_000, denom))
            .unwrap();
    }

    // Staker2 deposits tokens to the vault
    {
        app.send_tokens(owner.clone(), staker2.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 99_999_999 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker2.clone(), // set recipient to staker
            amount: Uint128::new(99_999_999),
        });
        tc.vault
            .execute_with_funds(app, &staker2, &msg, coins(99_999_999, denom))
            .unwrap();
    }

    // Staker1 approves Proxy
    {
        let msg = ExecuteMsg::SetApproveProxy(SetApproveProxyParams {
            proxy: proxy.clone(),
            approve: true,
        });
        tc.vault.execute(app, &staker1, &msg).unwrap();
    }

    // Staker2 approves Proxy
    {
        let msg = ExecuteMsg::SetApproveProxy(SetApproveProxyParams {
            proxy: proxy.clone(),
            approve: true,
        });
        tc.vault.execute(app, &staker2, &msg).unwrap();
    }

    // Proxy queues withdrawal to Staker1 (owner and controller)
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let response = tc.vault.execute(app, &proxy, &msg).unwrap();
        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
                Event::new("wasm-QueueWithdrawalTo")
                    .add_attribute("_contract_address", tc.vault.addr.to_string())
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
        let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(10_000));
        assert_eq!(
            response.unlock_timestamp,
            Timestamp::from_nanos(1571797519879305533)
        );
    }

    // move blockchain forward
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // Proxy queues withdrawal to Staker2 (owner and controller)
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(20_000),
        });
        let response = tc.vault.execute(app, &proxy, &msg).unwrap();
        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
                Event::new("wasm-QueueWithdrawalTo")
                    .add_attribute("_contract_address", tc.vault.addr.to_string())
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
        let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

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
        let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(10_000));
        assert_eq!(
            response.unlock_timestamp,
            Timestamp::from_nanos(1571797519879305533)
        );
    }

    // move blockchain forward
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // Proxy redeems withdrawal to Staker1
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: staker1.clone(),
            recipient: staker1.clone(),
        });
        let response = tc.vault.execute(app, &proxy, &msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
                Event::new("wasm-RedeemWithdrawalTo")
                    .add_attribute("_contract_address", tc.vault.addr.to_string())
                    .add_attribute("sender", proxy.to_string())
                    .add_attribute("controller", staker1.to_string())
                    .add_attribute("recipient", staker1.to_string())
                    .add_attribute("sub_shares", "10000")
                    .add_attribute("claimed_assets", "10000")
                    .add_attribute("total_shares", "199989999"),
                Event::new("transfer")
                    .add_attribute("recipient", staker1.to_string())
                    .add_attribute("sender", tc.vault.addr.to_string())
                    .add_attribute("amount", "10000denom")
            ]
        );

        // check that staker1 received the assets
        let staker1_balance = app.wrap().query_balance(&staker1, denom).unwrap();
        assert_eq!(staker1_balance, coin(900_010_000, denom)); // 1_000_000_000 - 100_000_000 + 10_000
    }

    // Staker2 redeems withdrawal to recipient
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
            controller: staker2.clone(),
            recipient: recipient.clone(),
        });
        let response = tc.vault.execute(app, &staker2, &msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
                Event::new("wasm-RedeemWithdrawalTo")
                    .add_attribute("_contract_address", tc.vault.addr.to_string())
                    .add_attribute("sender", staker2.to_string())
                    .add_attribute("controller", staker2.to_string())
                    .add_attribute("recipient", recipient.to_string())
                    .add_attribute("sub_shares", "20000")
                    .add_attribute("claimed_assets", "20000")
                    .add_attribute("total_shares", "199969999"),
                Event::new("transfer")
                    .add_attribute("recipient", recipient.to_string())
                    .add_attribute("sender", tc.vault.addr.to_string())
                    .add_attribute("amount", "20000denom")
            ]
        );

        // check that recipient received the assets
        let recipient_balance = app.wrap().query_balance(&recipient, denom).unwrap();
        assert_eq!(recipient_balance, coin(20_000, denom)); // 20_000 tokens received
    }

    // check that Staker1 has no queued shares and shares are reduced
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker1.to_string(),
        };
        let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(0));
        assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));

        // check that shares are reduced
        let query_shares = QueryMsg::Shares {
            staker: staker1.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(99_990_000)); // 100_000_000 - 10_000
    }

    // check that Staker2 has no queued shares and shares are reduced
    {
        let msg = QueryMsg::QueuedWithdrawal {
            controller: staker2.to_string(),
        };
        let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

        assert_eq!(response.queued_shares, Uint128::new(0));
        assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));

        // check that shares are reduced
        let query_shares = QueryMsg::Shares {
            staker: staker2.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(99_979_999)); // 99_999_999 - 20_000
    }
}

#[test]
fn test_withdrawal_with_proxy_errors() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker1 = app.api().addr_make("staker1");
    let staker2 = app.api().addr_make("staker2");
    let proxy = app.api().addr_make("proxy");

    // Staker1 deposits tokens to the vault
    {
        app.send_tokens(
            owner.clone(),
            staker1.clone(),
            &coins(1_000_000_000, "denom"),
        )
        .unwrap();

        // Deposit 100_000_000 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker1.clone(),
            amount: Uint128::new(100_000_000),
        });
        tc.vault
            .execute_with_funds(app, &staker1, &msg, coins(100_000_000, "denom"))
            .unwrap();
    }

    // Staker2 deposits tokens to the vault
    {
        app.send_tokens(
            owner.clone(),
            staker2.clone(),
            &coins(1_000_000_000, "denom"),
        )
        .unwrap();

        // Deposit 99_999_999 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker2.clone(),
            amount: Uint128::new(99_999_999),
        });
        tc.vault
            .execute_with_funds(app, &staker2, &msg, coins(99_999_999, "denom"))
            .unwrap();
    }

    // Staker1 queue withdrawal to staker2
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(10_000),
        });
        let err = tc.vault.execute(app, &staker1, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Unauthorized {
                msg: "Unauthorized sender".into()
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
        let err = tc.vault.execute(app, &staker2, &msg).unwrap_err();
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
        tc.vault.execute(app, &staker1, &msg).unwrap();
    }

    // proxy queue withdrawal for staker2 with staker1 as owner
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker2.clone(),
            owner: staker1.clone(),
            amount: Uint128::new(10_000),
        });
        let err = tc.vault.execute(app, &proxy, &msg).unwrap_err();
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
        let err = tc.vault.execute(app, &proxy, &msg).unwrap_err();
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
        let result = tc.vault.execute(app, &proxy, &msg);
        assert!(result.is_ok());

        // staker 2 tries to grief by queueing withdrawal to staker1 as controller
        let grief_msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker1.clone(),
            owner: staker2.clone(),
            amount: Uint128::new(1000),
        });
        let err = tc.vault.execute(app, &staker2, &grief_msg).unwrap_err();
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
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let denom = "denom";
    let token_amount: u128 = 999_999_999_999;

    // Fund tokens
    {
        // Fund the staker with some initial tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(token_amount, denom))
            .unwrap();
    }

    // Deposit some tokens from staker to Vault
    {
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(token_amount),
        });
        tc.vault
            .execute_with_funds(app, &staker, &msg, coins(token_amount, denom))
            .expect("staker deposit failed");
    }

    // queue withdrawal to
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        tc.router.execute(app, &owner, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        controller: staker.clone(),
        recipient: staker.clone(),
    });

    app.update_block(|block| {
        block.time = block.time.plus_seconds(115);
    });
    let response = tc.vault.execute(app, &staker, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault.addr.to_string()),
            Event::new("wasm-RedeemWithdrawalTo")
                .add_attribute("_contract_address", tc.vault.addr.to_string())
                .add_attribute("sender", staker.to_string())
                .add_attribute("controller", staker.to_string())
                .add_attribute("recipient", staker.to_string())
                .add_attribute("sub_shares", "10000")
                .add_attribute("claimed_assets", "10000")
                .add_attribute("total_shares", "999999989999"),
            Event::new("transfer")
                .add_attribute("recipient", staker.to_string())
                .add_attribute("sender", tc.vault.addr.to_string())
                .add_attribute("amount", "10000denom")
        ]
    );

    let msg = QueryMsg::QueuedWithdrawal {
        controller: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(0));
    assert_eq!(response.unlock_timestamp, Timestamp::from_seconds(0));
}

#[test]
fn test_redeem_withdrawal_to_no_queued_shares_error() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let staker = app.api().addr_make("staker");

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        controller: staker.clone(),
        recipient: staker.clone(),
    });

    let err = tc.vault.execute(app, &staker, &msg).unwrap_err();
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
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let denom = "denom";

    // Fund the staker with some initial tokens
    {
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(), // set recipient to staker
            amount: Uint128::new(115_687_654),
        });
        tc.vault
            .execute_with_funds(app, &staker, &msg, coins(115_687_654, denom))
            .unwrap();
    }

    // queue withdrawal to
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        tc.router.execute(app, &owner, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        controller: staker.clone(),
        recipient: staker.clone(),
    });

    let err = tc.vault.execute(app, &staker, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        VaultError::Locked {
            msg: "The shares are locked".into()
        }
        .to_string()
    );
}

#[test]
fn test_redeem_withdrawal_to_future_time_locked_shares_error() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let denom = "denom";

    // Fund the staker with some initial tokens
    {
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to staker's Vault
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(), // set recipient to staker
            amount: Uint128::new(115_687_654),
        });
        tc.vault
            .execute_with_funds(app, &staker, &msg, coins(115_687_654, denom))
            .unwrap();
    }

    // queue withdrawal to for the first time
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        tc.router.execute(app, &owner, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // queue withdrawal to for the second time
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(QueueWithdrawalToParams {
            controller: staker.clone(),
            owner: staker.clone(),
            amount: Uint128::new(12000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(RedeemWithdrawalToParams {
        controller: staker.clone(),
        recipient: staker.clone(),
    });

    let err = tc.vault.execute(app, &staker, &msg).unwrap_err();
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
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;

    let response: VaultInfoResponse = tc.vault.query(app, &QueryMsg::VaultInfo {}).unwrap();
    assert_eq!(
        response,
        VaultInfoResponse {
            total_shares: Uint128::new(0),
            total_assets: Uint128::new(0),
            router: tc.router.addr,
            pauser: tc.pauser.addr,
            operator: app.api().addr_make("operator"),
            asset_id: "cosmos:cosmos-testnet-14002/bank:denom".to_string(),
            asset_type: AssetType::Bank,
            asset_reference: "denom".to_string(),
            contract: "crates.io:bvs-vault-bank".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}

#[test]
fn test_system_lock_assets() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let denom = "denom";
    let original_deposit_amount: u128 = 100_000_000;

    let stakers = [
        app.api().addr_make("staker/1"),
        app.api().addr_make("staker/2"),
        app.api().addr_make("staker/3"),
    ];

    // setup/fund/stake tokens
    {
        for staker in stakers.iter() {
            app.send_tokens(
                owner.clone(),
                staker.clone(),
                &coins(original_deposit_amount, denom),
            )
            .unwrap();

            let msg = ExecuteMsg::DepositFor(RecipientAmount {
                recipient: staker.clone(), // set recipient to staker
                amount: Uint128::new(original_deposit_amount),
            });
            tc.vault
                .execute_with_funds(app, staker, &msg, coins(original_deposit_amount, denom))
                .unwrap();
        }
    }

    let vault_balance_pre_slash = app
        .wrap()
        .query_balance(tc.vault.addr(), denom)
        .unwrap()
        .amount;

    // positive test
    {
        let slash_amount = vault_balance_pre_slash
            .checked_div(Uint128::new(2))
            .unwrap();

        // don't really need to create dedicated var for this
        // but for readability
        let expected_vault_balance_post_slash =
            vault_balance_pre_slash.checked_sub(slash_amount).unwrap();

        let msg = ExecuteMsg::SlashLocked(Amount(slash_amount));
        tc.vault.execute(app, tc.router.addr(), &msg).unwrap();

        let vault_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        let router_balance = app.wrap().query_balance(tc.router.addr(), denom).unwrap();

        // assert that the vault balance is halved
        assert_eq!(
            vault_balance,
            coin(expected_vault_balance_post_slash.into(), denom)
        );

        assert_eq!(router_balance, coin(slash_amount.into(), denom));
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
            let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();

            // shares stays the same
            assert_eq!(shares, Uint128::new(original_deposit_amount));

            let query_assets = QueryMsg::Assets {
                staker: staker.to_string(),
            };
            let assets: Uint128 = tc.vault.query(app, &query_assets).unwrap();

            // assets should be halved
            assert_eq!(assets, Uint128::new(original_deposit_amount / 2));
        }
    }

    //negative test
    {
        // non-callable address
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount)));
        let resp = tc.vault.execute(app, &stakers[0], &msg);
        assert!(resp.is_err());

        // larger than vault balance
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount * 3)));
        let resp = tc.vault.execute(app, tc.router.addr(), &msg);
        assert!(resp.is_err());

        // zero amount
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(0)));
        let resp = tc.vault.execute(app, tc.router.addr(), &msg);
        assert!(resp.is_err());
    }
}
