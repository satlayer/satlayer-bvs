use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_vault_base::msg::{RecipientAmount, VaultInfoResponse};
use bvs_vault_cw20::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Uint128};
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
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
        cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();

        {
            let staker_balance = cw20.balance(app, &staker);
            assert_eq!(staker_balance, (100e15 as u128) - (stake_amounts));

            let contract_balance = cw20.balance(app, &vault.addr());
            assert_eq!(contract_balance, stake_amounts * (i + 1));

            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(stake_amounts));

            let total_shares: Uint128 = vault.query(&app, &QueryMsg::TotalShares {}).unwrap();
            assert_eq!(total_shares, Uint128::new(stake_amounts * (i + 1)));
        }
    }

    // the donation should skew exchange rate to the staker's favor.
    let attacker = app.api().addr_make("attacker");
    let donation_amount = 1200 as u128;
    cw20.increase_allowance(app, &attacker, &vault.addr(), 100e15 as u128);
    cw20.fund(app, &attacker, 100e15 as u128);
    cw20.transfer(app, &attacker, &vault.addr(), donation_amount);
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

    let balance = cw20.balance(app, &vault.addr());
    assert_eq!(balance, stake_amounts * staker_total + donation_amount);

    let total_shares = total_shares.u128();
    let virtual_total_asset = balance + 1e3 as u128;
    let virtual_total_shares = total_shares + 1e3 as u128;

    // should be 2:1 with current test configuration
    let new_exchange_rate = virtual_total_asset as f64 / virtual_total_shares as f64;

    let staker_asset_profit = (new_exchange_rate * stake_amounts as f64) - stake_amounts as f64;

    let total_balance_before = balance;

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
                assert_eq!(staker_balance as f64, 100e15 + staker_asset_profit);

                let query_shares = QueryMsg::Shares {
                    staker: staker.to_string(),
                };
                let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
                assert_eq!(shares, Uint128::new(0));
            }
        }
    }

    let total_asset_withdrawn = staker_total as f64 * stake_amounts as f64 * new_exchange_rate;

    let total_shares: Uint128 = vault.query(&app, &QueryMsg::TotalShares {}).unwrap();
    assert_eq!(total_shares, Uint128::new(0));

    let balance = cw20.balance(app, &vault.addr());
    assert_eq!(
        balance as f64,
        total_balance_before as f64 - total_asset_withdrawn
    );
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
        cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
        cw20.fund(app, &staker, 100e15 as u128);
        vault.execute(app, &staker, &msg).unwrap();

        {
            let staker_balance = cw20.balance(app, &staker);
            assert_eq!(staker_balance, (100e15 as u128) - (stake_amounts));

            let contract_balance = cw20.balance(app, &vault.addr());
            assert_eq!(contract_balance, stake_amounts * (i + 1));

            let query_shares = QueryMsg::Shares {
                staker: staker.to_string(),
            };
            let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
            assert_eq!(shares, Uint128::new(stake_amounts));

            let total_shares: Uint128 = vault.query(&app, &QueryMsg::TotalShares {}).unwrap();
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

                let contract_balance = cw20.balance(app, &vault.addr());
                assert_eq!(
                    contract_balance,
                    (stake_amounts * staker_total) - stake_amounts * (i + 1)
                );

                let query_shares = QueryMsg::Shares {
                    staker: staker.to_string(),
                };
                let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
                assert_eq!(shares, Uint128::new(0));
            }
        }
    }

    let total_shares: Uint128 = vault.query(&app, &QueryMsg::TotalShares {}).unwrap();
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    {
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, 19_810_537_012_990_153);

        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, 80_189_462_987_009_847);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
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

        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, 40_189_462_987_009_847);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
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

        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, 0);

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
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
    cw20.increase_allowance(app, &staker, &vault.addr(), 100e15 as u128);
    cw20.fund(app, &staker, 100e15 as u128);
    vault.execute(app, &staker, &msg).unwrap();

    {
        // assert that the staker's balance is reduced by the deposit_amount
        let staker_balance = cw20.balance(app, &staker);
        assert_eq!(staker_balance, (100e15 as u128) - deposit_amount.u128()); // final balance 19_810_537_012_990_153

        // assert contract balance is increased by the deposit_amount
        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, deposit_amount.u128());

        // assert that the staker's share is 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that the random_lucky_dude's share is increased
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
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
        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, deposit_amount.u128() - 40e15 as u128); // final balance 40_189_462_987_009_847

        // assert that random_lucky_dude is reduced
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
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
        let contract_balance = cw20.balance(app, &vault.addr());
        assert_eq!(contract_balance, 0);

        // assert that random_lucky_dude's shares is 0
        let query_shares = QueryMsg::Shares {
            staker: random_lucky_dude.to_string(),
        };
        let shares: Uint128 = vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // assert that random_lucky_dude's balance is increased
        let random_lucky_dude_balance = cw20.balance(app, &random_lucky_dude);
        assert_eq!(random_lucky_dude_balance, 40_189_462_987_009_847);
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
            asset_id: format!(
                "cosmos:cosmos-testnet-14002/cw20:{}",
                tc.cw20.addr.to_string()
            )
            .to_string(),
            contract: "crates.io:bvs-vault-cw20".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
