use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_vault_bank::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_base::msg::{RecipientAmount, VaultInfoResponse};
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, coins, Addr, Uint128};
use cw_multi_test::{App, Executor};

struct TestContracts {
    pauser: PauserContract,
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
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_687_654));
    }

    // Withdraw Partially
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
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
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(115_000_000));
    }

    // Withdraw All
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
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
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
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
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
    let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
    assert_eq!(shares, Uint128::new(1_000_000));

    // fund contract with extra assets, so now its 2:1 instead of 1:1
    app.send_tokens(
        owner.clone(),
        tc.vault.addr().clone(),
        &coins(1_000_000, denom),
    )
    .expect("failed to fund vault");

    // Second Deposit will now use 2:1 exchange rate
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1_000_000 + 500_249)); // 500_249 is the new share based on 2:1 exchange rate
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
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
    let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
    assert_eq!(shares, Uint128::new(1_000_000));

    // fund contract with extra assets, so now its 2:1 instead of 1:1
    app.send_tokens(
        owner.clone(),
        tc.vault.addr().clone(),
        &coins(1_000_000, denom),
    )
    .expect("failed to fund vault");

    // Withdraw will now use 2:1 exchange rate
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(1_000_000),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(999_000_000 + 1_999_000, denom)); // 1_999_000 is the new asset based on 2:1 exchange rate, extra 999_000.

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(1_000, denom));

        // assert staker share should be 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
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

    // Deposit 1_000_000 tokens from staker to Vault
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
    let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
    assert_eq!(shares, Uint128::new(9));

    // Withdraw will now use inflated exchange rate
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(9),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, vec![])
        .unwrap();

    // Assert balances and shares after Deposit
    {
        let staker_balance = app.wrap().query_balance(&staker, denom).unwrap();
        assert_eq!(staker_balance, coin(999_900_900, denom)); // staker loses 99_100 assets from the inflated exchange rate

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), denom).unwrap();
        assert_eq!(contract_balance, coin(100_099_100, denom)); // contract gains 99_100 extra assets

        // assert staker share should be 0
        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
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
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
    let msg = ExecuteMsg::Deposit(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(999_999_999_999),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(999_999_999_999, denom))
        .expect("staker deposit failed");

    // Deposit 1 token from staker2 to Vault
    let msg2 = ExecuteMsg::Deposit(RecipientAmount {
        recipient: staker2.clone(),
        amount: Uint128::new(1),
    });
    tc.vault
        .execute_with_funds(app, &staker2, &msg2, coins(Uint128::new(1).u128(), denom))
        .expect("staker2 deposit failed");

    // staker withdraw all
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
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
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // staker2 share should be 1
        let query_shares = QueryMsg::Shares {
            staker: staker2.to_string(),
        };
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1));

        // total share should be 1
        let query_shares = QueryMsg::TotalShares {};
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(1));
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
            contract: "crate:bvs-vault-bank".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
