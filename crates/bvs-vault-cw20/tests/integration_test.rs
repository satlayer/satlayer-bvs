use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_vault_base::msg::{JailDetail, Recipient, RecipientAmount, VaultInfoResponse};
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
        let staker = app.api().addr_make(&format!("staker/{}", i));
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
        }
    }

    // the donation should skew exchange rate to the staker's favor.
    let attacker = app.api().addr_make("attacker");
    let donation_amount = 1200_u128;
    cw20.increase_allowance(app, &attacker, vault.addr(), 100e15 as u128);
    cw20.fund(app, &attacker, 100e15 as u128);
    cw20.transfer(app, &attacker, vault.addr(), donation_amount);
    let shares: Uint128 = vault
        .query(
            app,
            &QueryMsg::Shares {
                staker: attacker.to_string(),
            },
        )
        .unwrap();

    assert_eq!(shares, Uint128::new(0));

    let total_shares: Uint128 = vault.query(app, &QueryMsg::TotalShares {}).unwrap();
    assert_eq!(total_shares, Uint128::new(stake_amounts * staker_total));

    let balance = cw20.balance(app, vault.addr());
    assert_eq!(balance, stake_amounts * staker_total + donation_amount);

    fn get_staker_asset_profit(app: &App, vault: &VaultCw20Contract, shares: Uint128) -> Uint128 {
        vault
            .query(app, &QueryMsg::ConvertToAssets { shares })
            .unwrap()
    }

    let total_balance_before = balance;
    let mut total_asset_withdrawn: Uint128 = Uint128::new(0);

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{}", i));
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
        let staker = app.api().addr_make(&format!("staker/{}", i));
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
        }
    }

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{}", i));
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
    vault.execute(app, &staker, &msg).unwrap();

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

fn test_transfer_asset_custody(slash_percent: u64) {
    let app = &mut App::default();
    let TestContracts {
        vault,
        cw20,
        router,
        ..
    } = TestContracts::init(app);

    let original_stake_amount = 200;
    let staker_total = 10;

    for i in 0..staker_total {
        let staker = app.api().addr_make(&format!("staker/{}", i));
        let msg = ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(original_stake_amount),
        });
        cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();

        {
            let staker_balance = cw20.balance(app, &staker);
            assert_eq!(staker_balance, (100e15 as u128) - (original_stake_amount));

            let contract_balance = cw20.balance(app, &vault.addr());
            assert_eq!(contract_balance, original_stake_amount * (i + 1));

            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(original_stake_amount));

            let total_shares: Uint128 = vault.query(&app, &QueryMsg::TotalShares {}).unwrap();
            assert_eq!(total_shares, Uint128::new(original_stake_amount * (i + 1)));
        }
    }

    // enable slashing
    {
        let msg = ExecuteMsg::SetSlashable(true);
        vault.execute(app, router.addr(), &msg).unwrap();
    }

    let jail_address = app.api().addr_make("jail_address");
    let vault_balance_preslash = cw20.balance(app, &vault.addr());
    {
        let msg = ExecuteMsg::TransferAssetCustody(JailDetail {
            jail_address: jail_address.clone(),
            percentage: slash_percent,
        });
        vault.execute(app, router.addr(), &msg).unwrap();

        let vault_balance_post_slash = cw20.balance(app, &vault.addr());

        let jail_balance = cw20.balance(app, &jail_address);

        let expected_jail_balance = vault_balance_preslash * (slash_percent as u128) / 100;

        let expected_vault_balance_post_slash = vault_balance_preslash - expected_jail_balance;

        assert_eq!(jail_balance, expected_jail_balance);
        assert_eq!(vault_balance_post_slash, expected_vault_balance_post_slash);

        // all the stakers should have same shares but reduced asset
        for i in 0..staker_total {
            let staker = app.api().addr_make(&format!("staker/{}", i));
            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };

            // shares of the staker stays the same
            let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(original_stake_amount));

            // they should have reduced asset now
            let asset_post_slash: Uint128 = vault
                .query(app, &QueryMsg::ConvertToAssets { shares })
                .unwrap();

            // reduce the asset by the slash percent
            let expected_asset_post_slash = Uint128::from(original_stake_amount)
                .checked_sub(
                    Uint128::from(original_stake_amount)
                        .checked_mul(Uint128::from(slash_percent))
                        .unwrap()
                        .checked_div(Uint128::from(100))
                        .unwrap(),
                )
                .unwrap();
        }
    }
}

#[test]
fn test_transfer_asset_custody_every_percent() {
    // test slash precent from 1 to 100
    // except 0% slashing, which is not allowed.
    for i in 1..100 {
        test_transfer_asset_custody(i);
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
        Timestamp::from_seconds(1571797519)
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
            slashing: false,
            asset_id: format!("cosmos:cosmos-testnet-14002/cw20:{}", tc.cw20.addr).to_string(),
            contract: "crates.io:bvs-vault-cw20".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
