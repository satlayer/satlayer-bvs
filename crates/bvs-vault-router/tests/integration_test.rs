use bvs_library::{
    ownership::OwnershipError,
    testing::{Cw20TokenContract, TestingContract},
};
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_router::{
    msg::{ExecuteMsg, QueryMsg, VaultListResponse},
    testing::VaultRouterContract,
    ContractError,
};
use cosmwasm_std::{testing::mock_env, Event, Uint64};
use cw_multi_test::App;

struct TestContracts {
    vault_router: VaultRouterContract,
    bank_vault: VaultBankContract,
    cw20_vault: VaultCw20Contract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::default();
        let env = mock_env();

        let _ = PauserContract::new(&mut app, &env, None);
        let _ = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let bank_vault = VaultBankContract::new(&mut app, &env, None);
        let _ = Cw20TokenContract::new(&mut app, &env, None);
        let cw20_vault = VaultCw20Contract::new(&mut app, &env, None);

        (
            app,
            Self {
                vault_router,
                bank_vault,
                cw20_vault,
            },
        )
    }
}

#[test]
fn set_vault_whitelist_false_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");
    let vault = app.api().addr_make("vault");

    let msg = &ExecuteMsg::SetVault {
        vault: vault.to_string(),
        whitelisted: false,
    };

    let response = tc.vault_router.execute(&mut app, &owner, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", vault.to_string())
                .add_attribute("whitelisted", "false"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: vault.to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(is_whitelisted, false);

    // query is delegated
    let operator = app.api().addr_make("operator");
    let msg = QueryMsg::IsValidating {
        operator: operator.to_string(),
    };
    let is_validating: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(is_validating, false);

    // list vaults
    let msg = QueryMsg::ListVaults {
        start_after: None,
        limit: None,
    };
    let response: VaultListResponse = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(response.0.len(), 1);
}

#[test]
fn set_vault_whitelist_true_bank_vault_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let msg = &ExecuteMsg::SetVault {
        vault: tc.bank_vault.addr().to_string(),
        whitelisted: true,
    };

    let response = tc.vault_router.execute(&mut app, &owner, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", tc.bank_vault.addr().to_string())
                .add_attribute("whitelisted", "true"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: tc.bank_vault.addr().to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(is_whitelisted, true);
}

#[test]
fn set_vault_whitelist_true_cw20_vault_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let msg = &ExecuteMsg::SetVault {
        vault: tc.cw20_vault.addr().to_string(),
        whitelisted: true,
    };

    let response = tc.vault_router.execute(&mut app, &owner, &msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", tc.cw20_vault.addr().to_string())
                .add_attribute("whitelisted", "true"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: tc.cw20_vault.addr().to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(is_whitelisted, true);
}

#[test]
fn set_withdrawal_lock_period() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let withdrawal_lock_period1 = Uint64::new(120);

    // set withdrawal lock period for the first time
    {
        let msg = &ExecuteMsg::SetWithdrawalLockPeriod(withdrawal_lock_period1);

        let response = tc.vault_router.execute(&mut app, &owner, &msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-SetWithdrawalLockPeriod")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute("prev_withdrawal_lock_period", 0.to_string())
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    ),
            ]
        );
    }

    let withdrawal_lock_period2 = Uint64::new(150);

    // update withdrawal lock period
    {
        let msg = &ExecuteMsg::SetWithdrawalLockPeriod(withdrawal_lock_period2);

        let response = tc.vault_router.execute(&mut app, &owner, &msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-SetWithdrawalLockPeriod")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period2.to_string()
                    ),
            ]
        );
    }

    // query the withdrawal lock period
    let msg = QueryMsg::WithdrawalLockPeriod {};
    let result: Uint64 = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(result, withdrawal_lock_period2);
}

#[test]
fn transfer_ownership_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");
    let new_owner = app.api().addr_make("new_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };

    let response = tc
        .vault_router
        .execute(&mut app, &owner, &transfer_msg)
        .unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-TransferredOwnership")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("old_owner", owner.as_str())
                .add_attribute("new_owner", new_owner.as_str()),
        ]
    );
}

#[test]
fn transfer_ownership_but_not_owner() {
    let (mut app, tc) = TestContracts::init();
    let not_owner = app.api().addr_make("not_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: not_owner.to_string(),
    };

    let err = tc
        .vault_router
        .execute(&mut app, &not_owner, &transfer_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Ownership {
            0: OwnershipError::Unauthorized
        }
        .to_string()
    );
}
