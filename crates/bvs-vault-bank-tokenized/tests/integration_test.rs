use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank_tokenized::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_bank_tokenized::testing::VaultBankTokenizedContract;
use bvs_vault_base::error::VaultError;
use bvs_vault_base::msg::{Amount, Recipient, RecipientAmount, VaultInfoResponse};
use bvs_vault_base::shares::QueuedWithdrawalInfo;
use bvs_vault_router::{msg::ExecuteMsg as RouterExecuteMsg, testing::VaultRouterContract};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, DenomMetadata, DenomUnit, Event, Timestamp, Uint128, Uint64,
    WasmMsg,
};
use cw2::ContractVersion;
use cw20::BalanceResponse;
use cw_multi_test::{App, Executor};

struct TestContracts {
    pauser: PauserContract,
    registry: RegistryContract,
    router: VaultRouterContract,
    vault: VaultBankTokenizedContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let denom_meta = DenomMetadata {
            description: "Test Token".to_string(),
            denom_units: vec![
                DenomUnit {
                    denom: "denom".to_string(),
                    exponent: 0,
                    aliases: vec![],
                },
                DenomUnit {
                    denom: "mdenom".to_string(),
                    exponent: 6,
                    aliases: vec!["microdenom".to_string()],
                },
            ],
            base: "mdenom".to_string(),
            display: "denom".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        };

        let mut app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .set_denom_metadata(storage, "denom".to_string(), denom_meta)
                .unwrap();
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });
        let env = mock_env();

        let pauser = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let router = VaultRouterContract::new(&mut app, &env, None);
        let vault = VaultBankTokenizedContract::new(&mut app, &env, None);

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

    let query_asset = QueryMsg::Assets {
        staker: staker.to_string(),
    };
    let asset: Uint128 = tc.vault.query(app, &query_asset).unwrap();
    assert_eq!(asset, Uint128::new(1_000_000)); // should be same as shares

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
    let res = tc
        .vault
        .execute_with_funds(app, &staker, &msg, coins(1_000_000, denom))
        .unwrap();
    println!("Deposit response: {:?}", res);

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(998_000_000, denom));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(3_000_000, denom));

        // assert staker share - for same asset should receive lesser shares (due to 3:1 exchange rate)
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1_500_000));

        let query_asset = QueryMsg::Assets {
            staker: staker.to_string(),
        };

        let asset: Uint128 = tc.vault.query(app, &query_asset).unwrap();
        assert_eq!(asset, Uint128::new(2_999_999)); // 1_000_000(staked) + 1_000_000(donated) +
                                                    // 1_000_000(staked again) = 3_000_000 (a few
                                                    // dust loose due to rounding in virtualOffset)
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
        Timestamp::from_nanos(1571797519879305533)
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
            asset_id: "cosmos:cosmos-testnet-14002/bank:denom".to_string(),
            contract: "crates.io:bvs-vault-bank-tokenized".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}

#[test]
fn test_cw20_semi_compliance() {
    // tokenized vault is only semi compliance
    // Because we don't allow burning and minting at all except when stake/unstake
    let (mut app, tc) = TestContracts::init();
    let vault = tc.vault;

    //unlike regular cw20 token, we don't allow external minting
    // so can't fund it, we'll have to deposit some staking token to have
    // some amount of receipt token circulating
    let staker = app.api().addr_make("staker/4545");
    let owner = app.api().addr_make("owner");
    app.send_tokens(
        owner.clone(),
        staker.clone(),
        &coins(999_999_999_999, "denom"),
    )
    .unwrap();
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(200),
    });
    vault
        .execute_with_funds(&mut app, &staker, &msg, coins(200, "denom"))
        .unwrap();

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
        assert_eq!(token_info.name, "SatLayer Test Token".to_string());
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
    let (app, tc) = TestContracts::init();
    let vault = tc.vault;

    let raw = app
        .wrap()
        .query_wasm_raw(vault.addr(), b"contract_info".to_vec())
        .expect("failed to query contract_info");

    let binary = &cosmwasm_std::Binary::from(raw.unwrap());

    let contract_info: ContractVersion =
        cosmwasm_std::from_json(binary).expect("invalid contract_info format");

    assert_eq!(contract_info.contract, "crates.io:bvs-vault-bank-tokenized",);
    assert_eq!(contract_info.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_system_lock_assets() {
    let (mut app, tc) = TestContracts::init();
    let vault = tc.vault;
    let router = tc.router;

    let original_deposit_amount: u128 = 100_000_000;

    let stakers = [
        app.api().addr_make("staker/1"),
        app.api().addr_make("staker/2"),
        app.api().addr_make("staker/3"),
    ];

    // setup/fund/stake tokens
    {
        for staker in stakers.iter() {
            let owner = app.api().addr_make("owner");

            app.send_tokens(
                owner.clone(),
                staker.clone(),
                &coins(original_deposit_amount, "denom"),
            )
            .unwrap();
            let msg = ExecuteMsg::DepositFor(RecipientAmount {
                recipient: staker.clone(),
                amount: Uint128::new(original_deposit_amount),
            });
            vault
                .execute_with_funds(
                    &mut app,
                    staker,
                    &msg,
                    coins(original_deposit_amount, "denom"),
                )
                .unwrap();
        }
    }

    let vault_balance_pre_slash = app.wrap().query_balance(vault.addr(), "denom").unwrap();

    // positive test
    {
        let slash_amount = vault_balance_pre_slash
            .amount
            .checked_div(Uint128::from(2_u128))
            .unwrap();

        // don't really need to create dedicated var for this
        // but for readability
        let expected_vault_balance_post_slash = vault_balance_pre_slash
            .amount
            .checked_sub(slash_amount)
            .unwrap();

        let msg = ExecuteMsg::SlashLocked(Amount(slash_amount));
        vault.execute(&mut app, router.addr(), &msg).unwrap();

        let vault_balance = app.wrap().query_balance(vault.addr(), "denom").unwrap();
        let router_balance = app.wrap().query_balance(router.addr(), "denom").unwrap();

        // assert that the vault balance is halved
        assert_eq!(vault_balance.amount, expected_vault_balance_post_slash);

        // assert that router get the slashed amount
        assert_eq!(router_balance.amount, slash_amount);
    }

    // non linear ratio sanity checks
    {
        // before the slash shares to assets are mostly linear 1:1
        // Since we are slashing 50% of the assets
        // without effecting the shares
        // the shares to assets ratio should be 2:1 now
        // This is intended.
        // The proportion of how much each staker get (shares) stays the same

        for staker in stakers.iter() {
            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(&mut app, &query_shares).unwrap();

            // shares stays the same
            assert_eq!(shares, Uint128::new(original_deposit_amount));

            let query_assets = QueryMsg::Assets {
                staker: staker.to_string(),
            };
            let assets: Uint128 = vault.query(&mut app, &query_assets).unwrap();

            // assets should be halved
            assert_eq!(assets, Uint128::from(50_000_000_u128));
        }
    }

    //negative test
    {
        // non-callable address
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount)));
        let resp = vault.execute(&mut app, &stakers[0], &msg);
        assert!(resp.is_err());

        // larger than vault balance
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(original_deposit_amount * 3)));
        let resp = vault.execute(&mut app, router.addr(), &msg);
        assert!(resp.is_err());

        // zero amount
        let msg = ExecuteMsg::SlashLocked(Amount(Uint128::new(0)));
        let resp = vault.execute(&mut app, router.addr(), &msg);
        assert!(resp.is_err());
    }
}

#[test]
fn test_deposit_transfer_then_withdraw_to() {
    let (mut app, tc) = TestContracts::init();
    let vault = tc.vault;

    let staker = app.api().addr_make("staker/1");
    let beneficiary = app.api().addr_make("beneficiary/1");
    let initial_deposit_amount: u128 = 30_000_000;
    let owner = app.api().addr_make("owner");

    app.send_tokens(owner.clone(), staker.clone(), &coins(100_000_000, "denom"))
        .unwrap();

    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(initial_deposit_amount),
    });
    vault
        .execute_with_funds(
            &mut app,
            &staker,
            &msg,
            coins(initial_deposit_amount, "denom"),
        )
        .unwrap();

    {
        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();
        assert_eq!(staker_balance, coin(70_000_000, "denom")); // unstaked_capital

        let contract_balance = app.wrap().query_balance(vault.addr(), "denom").unwrap();
        assert_eq!(contract_balance, coin(30_000_000, "denom")); // staked_capital

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };

        let shares: Uint128 = vault.query(&mut app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(30_000_000)); // initial_deposit_amount
    }

    // Transfer to beneficiary
    {
        let msg = ExecuteMsg::Transfer {
            recipient: beneficiary.to_string(),
            amount: Uint128::new(1000),
        };
        vault.execute(&mut app, &staker, &msg).unwrap();

        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(&mut app, &msg).unwrap();

        assert_eq!(resp.balance, Uint128::new(1000)); // donated amount

        let resp: BalanceResponse = vault
            .query(
                &mut app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            resp.balance,
            Uint128::new(29999000) // initial_deposit_amount - 1000
        );
    }

    // Fully Withdraw
    {
        let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
            amount: Uint128::new(initial_deposit_amount - 1000),
            recipient: staker.clone(),
        });
        vault.execute(&mut app, &staker, &msg).unwrap();

        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();

        // initial_deposit_amount - 1000 + unstaked_capital
        assert_eq!(
            staker_balance,
            coin(30_000_000 - 1000 + 70_000_000, "denom")
        );

        let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
            amount: Uint128::new(1000),
            recipient: beneficiary.clone(),
        });
        vault.execute(&mut app, &beneficiary, &msg).unwrap();

        let beneficiary_balance = app.wrap().query_balance(&beneficiary, "denom").unwrap();

        assert_eq!(beneficiary_balance, coin(1000, "denom"));

        let contract_balance = app.wrap().query_balance(vault.addr(), "denom").unwrap();

        assert_eq!(contract_balance, coin(0, "denom")); // should be empty now
    }

    // should have 0 receipt token left
    {
        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(&mut app, &msg).unwrap();

        assert!(resp.balance.is_zero());

        let resp: BalanceResponse = vault
            .query(
                &mut app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        assert!(resp.balance.is_zero());
    }
}

#[test]
fn test_deposit_transfer_then_queue_redeem_withdraw() {
    let (mut app, tc) = TestContracts::init();
    let vault = tc.vault;

    let staker = app.api().addr_make("staker/1");
    let beneficiary = app.api().addr_make("beneficiary/1");
    let initial_deposit_amount: u128 = 30_000_000;

    let owner = app.api().addr_make("owner");

    app.send_tokens(owner.clone(), staker.clone(), &coins(100_000_000, "denom"))
        .unwrap();

    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(initial_deposit_amount),
    });
    vault
        .execute_with_funds(
            &mut app,
            &staker,
            &msg,
            coins(initial_deposit_amount, "denom"),
        )
        .unwrap();

    {
        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();
        assert_eq!(staker_balance, coin(70_000_000, "denom")); // staker_unstaked_capital

        let contract_balance = app.wrap().query_balance(vault.addr(), "denom").unwrap();
        assert_eq!(contract_balance, coin(30_000_000, "denom")); // initial_deposit_amount

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(&mut app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(30_000_000)); // initial_deposit_amount
    }

    // Transfer to beneficiary
    {
        let msg = ExecuteMsg::Transfer {
            recipient: beneficiary.to_string(),
            amount: Uint128::new(1000),
        };
        vault.execute(&mut app, &staker, &msg).unwrap();

        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(&mut app, &msg).unwrap();

        assert_eq!(resp.balance, Uint128::new(1000));

        let resp: BalanceResponse = vault
            .query(
                &mut app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        // staker donated 1000 to beneficiary
        // so his receipt token balance should be reduced
        assert_eq!(
            resp.balance,
            Uint128::new(29999000) // initial_deposit_amount - 1000
        );
    }

    // Fully Withdraw
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            amount: Uint128::new(initial_deposit_amount - 1000),
            recipient: staker.clone(),
        });
        vault.execute(&mut app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            amount: Uint128::new(1000),
            recipient: beneficiary.clone(),
        });
        vault.execute(&mut app, &beneficiary, &msg).unwrap();
    }

    // fail premature redeem withdrawal to
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));
        let res = vault.execute(&mut app, &staker, &msg);
        assert!(res.is_err());

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(beneficiary.clone()));
        let res = vault.execute(&mut app, &beneficiary, &msg);
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
        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));
        vault.execute(&mut app, &staker, &msg).unwrap();

        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();

        // initial_deposit_amount + unstaked_capital - 1000
        assert_eq!(
            staker_balance,
            coin(30_000_000 + 70_000_000 - 1000, "denom")
        );

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(beneficiary.clone()));
        vault.execute(&mut app, &beneficiary, &msg).unwrap();

        let beneficiary_balance = app.wrap().query_balance(&beneficiary, "denom").unwrap();

        assert_eq!(beneficiary_balance, coin(1000, "denom")); // 1000

        let contract_balance = app.wrap().query_balance(vault.addr(), "denom").unwrap();

        assert_eq!(contract_balance, coin(0, "denom")); // should be empty now
    }

    // should have 0 receipt token left
    {
        let msg = QueryMsg::Balance {
            address: beneficiary.to_string(),
        };

        let resp: BalanceResponse = vault.query(&mut app, &msg).unwrap();

        assert!(resp.balance.is_zero());

        let resp: BalanceResponse = vault
            .query(
                &mut app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        assert!(resp.balance.is_zero());
    }
}
