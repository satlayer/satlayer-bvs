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
                .init_balance(storage, &owner, coins(1_000_000_000_000, "denom"))
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
    app.send_tokens(
        owner.clone(),
        staker.clone(),
        &coins(1_000_000_000, "denom"),
    )
    .unwrap();

    let msg = ExecuteMsg::Deposit(RecipientAmount {
        recipient: staker.clone(),
        amount: Uint128::new(115_687_654),
    });
    tc.vault
        .execute_with_funds(app, &staker, &msg, coins(115_687_654, "denom"))
        .unwrap();

    {
        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();
        assert_eq!(staker_balance, coin(884_312_346, "denom"));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), "denom").unwrap();
        assert_eq!(contract_balance, coin(115_687_654, "denom"));

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
    {
        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();
        assert_eq!(staker_balance, coin(885_000_000, "denom"));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), "denom").unwrap();
        assert_eq!(contract_balance, coin(115_000_000, "denom"));

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

    {
        let staker_balance = app.wrap().query_balance(&staker, "denom").unwrap();
        assert_eq!(staker_balance, coin(1_000_000_000, "denom"));

        let contract_balance = app.wrap().query_balance(tc.vault.addr(), "denom").unwrap();
        assert_eq!(contract_balance, coin(0, "denom"));

        let query_shares = QueryMsg::Shares {
            staker: staker.to_string(),
        };
        let shares: Uint128 = tc.vault.query(&app, &query_shares).unwrap();
        assert_eq!(shares, Uint128::new(0));
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
