use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_vault_base::msg::{Amount, AssetType, Recipient, RecipientAmount, VaultInfoResponse};
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
fn test_withdraw_overflow() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(100e15 as u128),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(200e15 as u128),
    });

    let err = vault.execute(app, &staker, &msg).unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Overflow: Cannot Sub with given operands"
    );
}

#[test]
fn test_withdraw_error_when_operator_is_validating() {
    let app = &mut App::default();
    let TestContracts {
        vault,
        cw20,
        registry,
        ..
    } = TestContracts::init(app);
    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    {
        // register operator + service
        registry
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

        registry
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
        registry
            .execute(
                app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        // register service to operator
        registry
            .execute(
                app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
                    service: service.to_string(),
                },
            )
            .unwrap();
    }

    let staker = app.api().addr_make("staker");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(100e15 as u128),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(200e15 as u128),
    });

    let err = vault.execute(app, &staker, &msg).unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Vault is validating, withdrawal must be queued"
    );
}

#[test]
fn test_multi_deposit_withdraw_non_linear_exchange_rates() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let stake_amounts = 20;
    let staker_total = 10;

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{i}"));
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(stake_amounts),
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();

        {
            let staker_balance = cw20.balance(app, &staker);
            assert_eq!(staker_balance, (100e15 as u128) - (stake_amounts));

            let contract_balance = cw20.balance(app, vault.addr());
            assert_eq!(contract_balance, stake_amounts * (i + 1));

            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };

            // -----------
            // NOTE: query share handler is internally giving out balance of
            // receipt token for the supplied address
            // quering balance and querying shares should be the same
            // they're internally using the exact same code from cw20_base crate.
            // but shares execute msg is still supported for compatibility
            // will not be carrying out redundant checks in later tests
            let shares: Uint128 = vault.query(app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(stake_amounts));

            let msg = cw20_base::msg::QueryMsg::Balance {
                address: staker.to_string(),
            };
            let balance = app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(vault.addr(), &msg)
                .unwrap()
                .balance;
            assert_eq!(balance, Uint128::new(stake_amounts));
            // -----------

            let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
            assert_eq!(total_shares, Uint128::new(stake_amounts * (i + 1)));

            let msg = cw20_base::msg::QueryMsg::TokenInfo {};
            let total_circulating_receipt_token_supply: Uint128 = app
                .wrap()
                .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &msg)
                .unwrap()
                .total_supply;

            assert_eq!(
                total_circulating_receipt_token_supply,
                Uint128::new(stake_amounts * (i + 1))
            );
        }
    }

    // the donation should skew exchange rate to the staker's favor.
    let attacker = app.api().addr_make("attacker");
    let donation_amount = 1200_u128;
    cw20.increase_allowance(app, &attacker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &attacker, 100e15 as u128);
    cw20.transfer(app, &attacker, vault.addr(), donation_amount);

    // attacker should get no receipt tokens
    // and accounting is synced
    {
        let msg = cw20_base::msg::QueryMsg::Balance {
            address: attacker.to_string(),
        };
        let balance = app
            .wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(vault.addr(), &msg)
            .unwrap()
            .balance;
        assert_eq!(balance, Uint128::new(0));
    }

    // total circulating supply and offset's internal total share counter
    // remain synced
    {
        let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
        assert_eq!(total_shares, Uint128::new(stake_amounts * staker_total));

        let msg = cw20_base::msg::QueryMsg::TokenInfo {};
        let total_circulating_receipt_token_supply: Uint128 = app
            .wrap()
            .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &msg)
            .unwrap()
            .total_supply;
        assert_eq!(
            total_circulating_receipt_token_supply,
            Uint128::new(stake_amounts * staker_total)
        );
    }

    let balance = cw20.balance(app, vault.addr());
    assert_eq!(balance, stake_amounts * staker_total + donation_amount);

    fn get_staker_asset_profit(
        app: &App,
        vault: &VaultCw20TokenizedContract,
        shares: Uint128,
    ) -> Uint128 {
        vault
            .query(app, &QueryMsg::ConvertToAssets { shares })
            .unwrap()
    }

    let total_balance_before = balance;
    let mut total_asset_withdrawn: Uint128 = Uint128::new(0);

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{i}"));
        {
            let staker_asset_profit =
                get_staker_asset_profit(app, &vault, Uint128::new(stake_amounts));
            total_asset_withdrawn = total_asset_withdrawn
                .checked_add(staker_asset_profit)
                .unwrap();

            let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
                amount: Uint128::new(stake_amounts),
                recipient: staker.clone(),
            });
            vault.execute(app, &staker, &msg).unwrap();

            {
                // get staker balance after withdrawal
                let staker_balance = cw20.balance(app, &staker);
                assert_eq!(
                    Uint128::from(staker_balance),
                    Uint128::new(100e15 as u128)
                        .checked_sub(Uint128::from(stake_amounts))
                        .unwrap()
                        .checked_add(staker_asset_profit)
                        .unwrap()
                );

                let query_shares = QueryMsg::Shares {
                    staker: staker.to_string(),
                };
                let shares: Uint128 = vault.query(app, &query_shares).unwrap();
                assert_eq!(shares, Uint128::new(0));
            }
        }
    }

    let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
    assert_eq!(total_shares, Uint128::new(0));

    let msg = cw20_base::msg::QueryMsg::TokenInfo {};
    let total_circulating_receipt_token_supply: Uint128 = app
        .wrap()
        .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &msg)
        .unwrap()
        .total_supply;
    assert_eq!(total_circulating_receipt_token_supply, Uint128::new(0));

    let balance = cw20.balance(app, vault.addr());
    assert_eq!(balance, total_balance_before - total_asset_withdrawn.u128());
}

#[test]
fn test_multi_deposit_withdraw() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let stake_amounts = 200;
    let staker_total = 500;

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{i}"));
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(stake_amounts),
        });
        cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();

        {
            let staker_balance = cw20.balance(app, &staker);
            assert_eq!(staker_balance, (100e15 as u128) - (stake_amounts));

            let contract_balance = cw20.balance(app, vault.addr());
            assert_eq!(contract_balance, stake_amounts * (i + 1));

            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(stake_amounts));

            let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
            assert_eq!(total_shares, Uint128::new(stake_amounts * (i + 1)));

            let msg = cw20_base::msg::QueryMsg::TokenInfo {};

            let total_circulating_receipt_token_supply: Uint128 = app
                .wrap()
                .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &msg)
                .unwrap()
                .total_supply;

            assert_eq!(
                total_circulating_receipt_token_supply,
                Uint128::new(stake_amounts * (i + 1))
            );
        }
    }

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{i}"));
        {
            let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
                amount: Uint128::new(stake_amounts),
                recipient: staker.clone(),
            });
            vault.execute(app, &staker, &msg).unwrap();

            {
                let staker_balance = cw20.balance(app, &staker);
                assert_eq!(staker_balance, 100e15 as u128);

                let contract_balance = cw20.balance(app, vault.addr());
                assert_eq!(
                    contract_balance,
                    (stake_amounts * staker_total) - stake_amounts * (i + 1)
                );

                let query_shares = QueryMsg::Shares {
                    staker: staker.to_string(),
                };
                let shares: Uint128 = vault.query(app, &query_shares).unwrap();
                assert_eq!(shares, Uint128::new(0));
            }
        }
    }

    let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
    assert_eq!(total_shares, Uint128::new(0));

    let msg = cw20_base::msg::QueryMsg::TokenInfo {};
    let total_circulating_receipt_token_supply: Uint128 = app
        .wrap()
        .query_wasm_smart::<cw20::TokenInfoResponse>(vault.addr(), &msg)
        .unwrap()
        .total_supply;
    assert_eq!(total_circulating_receipt_token_supply, Uint128::new(0));
}

#[test]
fn test_deposit_withdraw() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker/4545");
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(80_189_462_987_009_847),
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    let response = vault.execute(app, &staker, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", vault.addr.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", vault.addr.to_string())
                .add_attribute("action", "mint")
                .add_attribute("to", staker.to_string())
                .add_attribute("amount", "80189462987009847"),
            Event::new("wasm-DepositFor")
                .add_attribute("_contract_address", vault.addr.to_string())
                .add_attribute("sender", staker.to_string())
                .add_attribute("recipient", staker.to_string())
                .add_attribute("assets", "80189462987009847")
                .add_attribute("shares", "80189462987009847")
                .add_attribute("total_shares", "80189462987009847"),
            Event::new("execute").add_attribute("_contract_address", cw20.addr.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", cw20.addr.to_string())
                .add_attribute("action", "transfer_from")
                .add_attribute("from", staker.to_string())
                .add_attribute("to", vault.addr.to_string())
                .add_attribute("by", vault.addr().to_string())
                .add_attribute("amount", "80189462987009847")
        ]
    );

    {
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, 19_810_537_012_990_153);

        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 80_189_462_987_009_847);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(80_189_462_987_009_847));
    }

    // Partially Withdraw
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(40e15 as u128),
        recipient: staker.clone(),
    });
    vault.execute(app, &staker, &msg).unwrap();

    {
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, 59_810_537_012_990_153);

        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 40_189_462_987_009_847);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(40_189_462_987_009_847));
    }

    // Fully Withdraw
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(40_189_462_987_009_847),
        recipient: staker.clone(),
    });
    vault.execute(app, &staker, &msg).unwrap();

    {
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, 100e15 as u128);

        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 0);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
    }
}

#[test]
fn test_deposit_for_and_withdraw_to_other_address() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker/4545");
    let random_lucky_dude = app.api().addr_make("random_lucky_dude");
    // Staker deposits for random_lucky_dude
    let deposit_amount = Uint128::new(80_189_462_987_009_847);
    let msg = ExecuteMsg::DepositFor(RecipientAmount {
        recipient: random_lucky_dude.clone(), // recipient is not staker
        amount: deposit_amount,
    });
    cw20.increase_allowance(app, &staker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    {
        // assert that the staker's balance is reduced by the deposit_amount
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, (100e15 as u128) - deposit_amount.u128()); // final balance 19_810_537_012_990_153

        // assert contract balance is increased by the deposit_amount
        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, deposit_amount.u128());

        // assert that the staker's share is 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that the random_lucky_dude's share is increased
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(80_189_462_987_009_847));
    }

    // Partially Withdraw to random_lucky_dude2 from random_lucky_dude
    let random_lucky_dude2 = app.api().addr_make("random_lucky_dude2");
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(40e15 as u128),
        recipient: random_lucky_dude2.clone(),
    });
    vault.execute(app, &random_lucky_dude, &msg).unwrap();

    {
        // assert that the staker_balance is unchanged
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, (100e15 as u128) - deposit_amount.u128());

        // assert that the contract's balance is reduced
        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, deposit_amount.u128() - 40e15 as u128); // final balance 40_189_462_987_009_847

        // assert that random_lucky_dude is reduced
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(40_189_462_987_009_847));

        // assert that random_lucky_dude2 balance is increased
        let random_lucky_dude2_balance = cw20.balance(app, &random_lucky_dude2);
        assert_eq!(random_lucky_dude2_balance, 40e15 as u128);
    }

    // Fully Withdraw to random_lucky_dude from random_lucky_dude
    let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
        amount: Uint128::new(40_189_462_987_009_847),
        recipient: random_lucky_dude.clone(),
    });
    vault.execute(app, &random_lucky_dude, &msg).unwrap();

    {
        // assert that the staker_balance is unchanged
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, (100e15 as u128) - deposit_amount.u128());

        // assert that the contract's balance is 0
        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 0);

        // assert that random_lucky_dude's shares is 0
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that random_lucky_dude's balance is increased
        let random_lucky_dude_balance = cw20.balance(app, &random_lucky_dude);
        assert_eq!(random_lucky_dude_balance, 40_189_462_987_009_847);
    }
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

    let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
        recipient: staker.clone(),
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
                .add_attribute("recipient", staker.to_string())
                .add_attribute("queued_shares", "10000")
                .add_attribute("new_unlock_timestamp", "1571797519")
                .add_attribute("total_queued_shares", "10000")
        ]
    );

    let msg = QueryMsg::QueuedWithdrawal {
        staker: staker.to_string(),
    };
    let response: QueuedWithdrawalInfo = vault.query(app, &msg).unwrap();

    assert_eq!(response.queued_shares, Uint128::new(10000));
    assert_eq!(
        response.unlock_timestamp,
        Timestamp::from_nanos(1571797519879305533)
    );
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

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
        staker: staker.to_string(),
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

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // queue withdrawal to for the second time
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();
    }

    let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

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
fn test_queue_redeem_withdrawal_with_different_recipient() {
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
        cw20.fund(app, &staker, token_amount.into());
        vault.execute(app, &staker, &msg).unwrap();
    }

    // set withdrawal lock period
    {
        let msg = RouterExecuteMsg::SetWithdrawalLockPeriod(Uint64::new(100));
        router.execute(app, &owner, &msg).unwrap();
    }

    // queue and redeem withdrawal to staker
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        vault.execute(app, &staker, &msg).unwrap();

        let balance = cw20.balance(app, &staker);
        assert_eq!(balance, 10000u128);
    }

    let new_staker = app.api().addr_make("new_staker");

    // queue withdrawal to staker, redeem withdrawal to new_staker
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(new_staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        vault.execute(app, &staker, &msg).unwrap();

        let balance = cw20.balance(app, &staker);
        assert_eq!(balance, 10000u128);
    }

    // queue withdrawal to staker, redeem withdrawal to staker with wrong info.sender
    {
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(10000),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(new_staker.clone()));

        app.update_block(|block| {
            block.time = block.time.plus_seconds(115);
        });
        let err = vault.execute(app, &new_staker, &msg).unwrap_err();
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
fn test_deposit_transfer_then_withdraw_to() {
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
        assert_eq!(staker_balance, 19_810_537_012_990_153); // unstaked_capital

        let contract_balance = cw20.balance(app, vault.addr());
        assert_eq!(contract_balance, 80_189_462_987_009_847); // staked_capital

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

        assert_eq!(resp.balance, Uint128::new(1000)); // donated amount

        let resp: BalanceResponse = vault
            .query(
                app,
                &QueryMsg::Balance {
                    address: staker.to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            resp.balance,
            Uint128::new(80_189_462_987_008_847) // initial_deposit_amount - donation_amount
        );
    }

    // Fully Withdraw
    {
        let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
            amount: Uint128::new(initial_deposit_amount - 1000),
            recipient: staker.clone(),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let staker_balance = cw20.balance(app, &staker);

        // initial_deposit_amount + unstaked_capital - donation_amount
        assert_eq!(staker_balance, 99_999_999_999_999_000);

        let msg = ExecuteMsg::WithdrawTo(RecipientAmount {
            amount: Uint128::new(1000),
            recipient: beneficiary.clone(),
        });
        vault.execute(app, &beneficiary, &msg).unwrap();

        let beneficiary_balance = cw20.balance(app, &beneficiary);

        assert_eq!(beneficiary_balance, 1000);

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
        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            amount: Uint128::new(initial_deposit_amount - 1000),
            recipient: staker.clone(),
        });
        vault.execute(app, &staker, &msg).unwrap();

        let msg = ExecuteMsg::QueueWithdrawalTo(RecipientAmount {
            amount: Uint128::new(1000),
            recipient: beneficiary.clone(),
        });
        vault.execute(app, &beneficiary, &msg).unwrap();
    }

    // fail premature redeem withdrawal to
    {
        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));
        let res = vault.execute(app, &staker, &msg);
        assert!(res.is_err());

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(beneficiary.clone()));
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
        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(staker.clone()));
        vault.execute(app, &staker, &msg).unwrap();

        let staker_balance = cw20.balance(app, &staker);

        // initial_deposit_amount + unstaked_capital - donation_amount
        assert_eq!(staker_balance, 99_999_999_999_999_000);

        let msg = ExecuteMsg::RedeemWithdrawalTo(Recipient(beneficiary.clone()));
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
