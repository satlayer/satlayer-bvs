use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_factory::{state::CodeIdLabel, testing::VaultFactoryContract};
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::{testing::mock_env, Empty};
use cw_multi_test::{App, Contract};

struct TestContracts {
    vault_router: VaultRouterContract,
    vault_factory: VaultFactoryContract,
    cw20_token: Cw20TokenContract,
    registry: RegistryContract,
    bank_wrapper: Box<dyn Contract<Empty>>,
    cw20_vault_wrapper: Box<dyn Contract<Empty>>,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::default();
        let env = mock_env();

        let _ = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);
        let vault_factory = VaultFactoryContract::new(&mut app, &env, None);

        let bank_wrapper = VaultBankContract::wrapper();
        let cw20_vault_wrapper = VaultCw20Contract::wrapper();

        (
            app,
            Self {
                registry,
                cw20_token: cw20,
                vault_router,
                vault_factory,
                bank_wrapper,
                cw20_vault_wrapper,
            },
        )
    }
}

#[test]
fn test_cw20_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let cw20_token = contracts.cw20_token;
    let owner = app.api().addr_make("owner");

    // link the factory to router
    {
        let msg = bvs_vault_factory::msg::ExecuteMsg::SetVaults {
            router: contracts.vault_router.addr().to_string(),
            registry: contracts.registry.addr().to_string(),
        };

        factory.execute(&mut app, &owner, &msg).unwrap();
    }

    // register an operator
    {
        let msg = bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
            metadata: bvs_registry::msg::Metadata {
                name: Some("operator".to_string()),
                uri: Some("https://example.com".to_string()),
            },
        };
        contracts
            .registry
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let cw20_vault_code_id = app.store_code(contracts.cw20_vault_wrapper);

    let msg = bvs_vault_factory::msg::ExecuteMsg::AddCodeId {
        code_id: cw20_vault_code_id,
        label: CodeIdLabel::Cw20Vault,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20 {
        code_id: cw20_vault_code_id,
        cw20: cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg);

    assert_eq!(res.is_ok(), true);
}

#[test]
fn test_bank_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;

    // register an operator
    {
        let msg = bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
            metadata: bvs_registry::msg::Metadata {
                name: Some("operator".to_string()),
                uri: Some("https://example.com".to_string()),
            },
        };
        contracts
            .registry
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let bank_vault_code_id = app.store_code(bank_vault);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::AddCodeId {
        code_id: bank_vault_code_id,
        label: CodeIdLabel::BankVault,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBank {
        code_id: bank_vault_code_id,
        denom: "SATL".to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg);

    assert_eq!(res.is_ok(), true);
}

#[test]
fn test_unauthorized_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;
    let owner = app.api().addr_make("owner");

    // link the factory to router
    {
        let msg = bvs_vault_factory::msg::ExecuteMsg::SetVaults {
            router: contracts.vault_router.addr().to_string(),
            registry: contracts.registry.addr().to_string(),
        };

        factory.execute(&mut app, &owner, &msg).unwrap();
    }

    let bank_vault_code_id = app.store_code(bank_vault);

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBank {
        code_id: bank_vault_code_id,
        denom: "SATL".to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap_err();

    assert_eq!(
        res.root_cause().to_string(),
        bvs_vault_factory::error::ContractError::Unauthorized {}.to_string()
    );

    let cw20_vault_code_id = app.store_code(contracts.cw20_vault_wrapper);

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20 {
        code_id: cw20_vault_code_id,
        cw20: contracts.cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap_err();

    assert_eq!(
        res.root_cause().to_string(),
        bvs_vault_factory::error::ContractError::Unauthorized {}.to_string()
    );
}

#[test]
fn test_unauthorized_code_id_whitelist() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;

    let bank_vault_code_id = app.store_code(bank_vault);

    let random = app.api().addr_make("random");

    let msg = bvs_vault_factory::msg::ExecuteMsg::AddCodeId {
        code_id: bank_vault_code_id,
        label: CodeIdLabel::BankVault,
    };

    let res = factory.execute(&mut app, &random, &msg).unwrap_err();

    assert!(matches!(
        res.downcast_ref::<bvs_vault_factory::error::ContractError>(),
        Some(bvs_vault_factory::error::ContractError::Ownership(..))
    ));

    let query_res: bvs_vault_factory::msg::AllowedCodeIdsResponse = factory
        .query(
            &app,
            &bvs_vault_factory::msg::QueryMsg::GetAllowedCodeIds {},
        )
        .unwrap();

    assert_eq!(query_res.code_ids.len(), 0);
}

#[test]
fn test_add_remove_code_id() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;

    let bank_vault_code_id = app.store_code(bank_vault);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::AddCodeId {
        code_id: bank_vault_code_id,
        label: CodeIdLabel::BankVault,
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap();

    let query_res: bvs_vault_factory::msg::AllowedCodeIdsResponse = factory
        .query(
            &app,
            &bvs_vault_factory::msg::QueryMsg::GetAllowedCodeIds {},
        )
        .unwrap();

    assert_eq!(query_res.code_ids.len(), 1);
    assert_eq!(query_res.code_ids[0].0, bank_vault_code_id);
    assert_eq!(query_res.code_ids[0].1, CodeIdLabel::BankVault);

    let res = factory
        .execute(
            &mut app,
            &owner,
            &bvs_vault_factory::msg::ExecuteMsg::RemoveCodeId {
                code_id: bank_vault_code_id,
            },
        )
        .unwrap();

    let query_res: bvs_vault_factory::msg::AllowedCodeIdsResponse = factory
        .query(
            &app,
            &bvs_vault_factory::msg::QueryMsg::GetAllowedCodeIds {},
        )
        .unwrap();

    assert_eq!(query_res.code_ids.len(), 0);
}
