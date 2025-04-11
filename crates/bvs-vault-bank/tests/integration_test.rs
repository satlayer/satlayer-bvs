use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_base::error::VaultError;
use bvs_vault_base::msg::{Recipient, RecipientAmount, VaultInfoResponse};
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
fn test_deposit_withdraw() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // Deposit 115_687_654 tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(115_687_654),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(115_687_654, denom))
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000 - 115_687_654, denom));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(115_687_654, denom));

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_687_654));
    }

    // Withdraw Partially
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(687_654),
        recipient: staker.clone(),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after partial Withdraw
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(
            staker_balance,
            coin(1_000_000_000 - 115_687_654 + 687_654, denom)
        );

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(115_687_654 - 687_654, denom));

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_000_000));
    }

    // Withdraw All
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(115_000_000),
        recipient: staker.clone(),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Withdraw All
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000, denom));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(0, denom));

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
    }
}

#[test]
fn test_deposit_for_and_withdraw_to_other_address() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let random_lucky_dude = app.api().addr_make("random_lucky_dude");
    let random_lucky_dude2 = app.api().addr_make("random_lucky_dude2");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // Deposit 115_687_654 tokens from staker to random_lucky_dude's Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: random_lucky_dude.clone(), // set recipient to random_lucky_dude
        amount: Uint128::new(115_687_654),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(115_687_654, denom))
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000 - 115_687_654, denom));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(115_687_654, denom));

        // assert that staker receives 0 shares
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that random_lucky_dude receives 115_687_654 shares from staker asset
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_687_654));
    }

    // Withdraw Partially to random_lucky_dude2
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(687_654),
        recipient: random_lucky_dude2.clone(),
    });
    tc.vault
        .execute_with_funds(app, &random_lucky_dude, &msg, vec![])
        .unwrap();

    // Assert balances and shares after partial Withdraw
    {
        // staker balance should remain the same
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000 - 115_687_654, denom));

        // contract balance should decrease
        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(115_687_654 - 687_654, denom));

        // assert that random_lucky_dude2 receives 687_654 assets
        let random_lucky_dude2_balance = app
            .wrap()
            .query_balance(&random_lucky_dude2, denom)
            .unwrap();
        assert_eq!(random_lucky_dude2_balance, coin(687_654, denom));

        // assert that random_lucky_dude has reduced shares
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_687_654 - 687_654)); // should equal contract balance
    }

    // Withdraw All to random_lucky_dude
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(115_000_000),
        recipient: random_lucky_dude.clone(),
    });
    tc.vault
        .execute_with_funds(app, &random_lucky_dude, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Withdraw All
    {
        // staker balance should remain the same
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000 - 115_687_654, denom));

        // contract balance should be empty
        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(0, denom));

        // assert that random_lucky_dude receives 115_000_000 assets
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that random_lucky_dude has 0 shares
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that random_lucky_dude2 still has 687_654 assets
        let random_lucky_dude2_balance = app
            .wrap()
            .query_balance(&random_lucky_dude2, denom)
            .unwrap();
        assert_eq!(random_lucky_dude2_balance, coin(687_654, denom));
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
fn test_withdraw_with_non_linear_exchange_rate() {
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

    // Withdraw will now use 2:1 exchange rate
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(999_000_000 + 1_999_999, denom)); // 1_999_999 is the new asset based on 2:1 exchange rate, extra 999_999.

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(1, denom));

        // assert staker share should be 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
    }
}

#[test]
fn test_withdraw_with_inflated_exchange_rate() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // fund contract with massive assets to inflate exchange rate
    app.send_tokens(
        owner.clone(),
        tc.vault.addr().clone(),
        &coins(100_000_000, denom),
    )
    .expect("failed to fund vault");

    // Deposit 100_000_001 tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(100_000_001),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(100_000_001, denom))
        .expect("staker deposit failed");

    // assert staker share
    let query_shares = QueryMsg::Shares {
        staker: staker.to_string(),
    };
    let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
    assert_eq!(shares, Uint128::new(1));

    // Withdraw will now use inflated exchange rate
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000, denom)); // staker gets back initial asset

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(100_000_000, denom)); // contract back to donation balance

        // assert staker share should be 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
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
fn test_withdraw_error_when_operator_is_validating() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
        .unwrap();

    // Deposit tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000_000),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(1_000_000_000, denom))
        .expect("staker deposit failed");

    {
        // register operator + service
        tc.registry
            .execute(
                app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: Some("https://example.com".to_string()),
                    },
                },
            )
            .unwrap();

        tc.registry
            .execute(
                app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: Some("https://example.com".to_string()),
                    },
                },
            )
            .unwrap();

        // register operator to service
        tc.registry
            .execute(
                app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        // register service to operator
        tc.registry
            .execute(
                app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
                    service: service.to_string(),
                },
            )
            .unwrap();
    }

    // withdraw
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000_000),
    });
    let err = tc
        .vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap_err();

    // assert error using withdrawTo because vault is validating
    assert_eq!(
        err.root_cause().to_string(),
        "Vault is validating, withdrawal must be queued"
    );
}

#[test]
fn test_massive_deposit_and_withdraw() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker/934");
    let staker2 = app.api().addr_make("staker/900");
    let denom = "denom";

    // Fund the staker with some initial tokens
    app.send_tokens(
        owner.clone(),
        staker.clone(),
        &coins(999_999_999_999, denom),
    )
    .unwrap();
    app.send_tokens(owner.clone(), staker2.clone(), &coins(1, denom))
        .unwrap();

    // Deposit 999_999_999_999 tokens from staker to Vault
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(999_999_999_999),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(999_999_999_999, denom))
        .expect("staker deposit failed");

    // Deposit 1 token from staker2 to Vault
    let msg2 = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker2.clone(),
        amount: Uint128::new(1),
    });
    tc.vault
        .execute_with_funds(app, &staker2, &msg2, coins(Uint128::new(1).u128(), denom))
        .expect("staker2 deposit failed");

    // staker withdraw all
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(999_999_999_999),
        recipient: staker.clone(),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .expect("staker withdraw failed");

    // Assert balances and shares after Withdraw All
    {
        // staker should have initial balance
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(999_999_999_999, denom));

        // contract should have 1 token (from staker2)
        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(1, denom));

        // staker share should be 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // staker2 share should be 1
        let query_shares = QueryMsg::Shares {
            staker: staker2.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1));

        // total share should be 1
        let query_shares = QueryMsg::TotalShares {};
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1));
    }
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

    let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
        recipient: staker.clone(),
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
                .add_attribute("recipient", staker.to_string())
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
        staker: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = tc.vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(10000));
    assert_eq!(
        response.unlock_timestamp,
        Timestamp::from_seconds(1571797519)
    );
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

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
        staker: staker.to_string(),
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

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // queue withdrawal to for the second time
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(12000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
fn test_queue_redeem_withdrawal_with_different_recipient() {
    let (mut app, tc) = TestContracts::init();
    let app = &mut app;
    let owner = app.api().addr_make("owner");
    let staker = app.api().addr_make("staker");
    let denom = "denom";
    let token_amount: u128 = 999_999_999_999;

    // Fund tokens
    {
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

    // set withdrawal lock period
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        tc.router.execute(app, &owner, &msg).unwrap();
    }

    // queue and redeem withdrawal to staker
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        tc.vault.execute(app, &staker, &msg).unwrap();

        let balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(balance.amount, Uint128::from(10000u128));
    }

    let new_staker = app.api().addr_make("new_staker");

    // queue withdrawal to staker, redeem withdrawal to new_staker
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(new_staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        tc.vault.execute(app, &staker, &msg).unwrap();

        let balance = app.wrap().query_balance(&new_staker, denom).unwrap();
        assert_eq!(balance.amount, Uint128::from(10000u128));
    }

    // queue withdrawal to staker, redeem withdrawal to staker with wrong info.sender
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        tc.vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(new_staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        let err = tc.vault.execute(app, &new_staker, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            VaultError::Zero {
                msg: "No queued shares".into()
            }
            .to_string()
        );
    }
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
            slashing: false,
            asset_id: "cosmos:cosmos-testnet-14002/bank:denom".to_string(),
            contract: "crates.io:bvs-vault-bank".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
