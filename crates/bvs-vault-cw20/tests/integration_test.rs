use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_vault_base::msg::{RecipientAmount, VaultInfoResponse};
use bvs_vault_cw20::msg::{ExecuteMsg, QueryMsg};
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::App;

struct TestContracts {
    pauser: PauserContract,
    router: VaultRouterContract,
    cw20: Cw20TokenContract,
    vault: VaultCw20Contract,
}

impl TestContracts {
    fn init(app: &mut App) -> TestContracts {
        let env = mock_env();

        let pauser = PauserContract::new(app, &env, None);
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
            router,
            vault,
            cw20,
        }
    }
}

#[test]
fn test_deposit_withdraw() {
    let app = &mut App::default();
    let TestContracts { vault, cw20, .. } = TestContracts::init(app);

    let staker = app.api().addr_make("staker/4545");
    let msg = ExecuteMsg::Deposit(RecipientAmount {
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
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
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
    let msg = ExecuteMsg::Withdraw(RecipientAmount {
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
            contract: "crate:bvs-vault-cw20".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
