use bvs_guardrail::testing::GuardrailContract;
use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_factory::msg::VaultType;
use bvs_vault_factory::testing::VaultFactoryContract;
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::{testing::mock_env, Empty};
use cw_multi_test::{App, Contract};

struct TestContracts {
    vault_factory: VaultFactoryContract,
    cw20_token: Cw20TokenContract,
    registry: RegistryContract,
    bank_wrapper: Box<dyn Contract<Empty>>,
    bank_tokenized_wrapper: Box<dyn Contract<Empty>>,
    cw20_vault_wrapper: Box<dyn Contract<Empty>>,
    cw20_tokenized_wrapper: Box<dyn Contract<Empty>>,
    pauser: PauserContract,
    router: VaultRouterContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::default();
        let env = mock_env();

        let pauser = PauserContract::new(&mut app, &env, None);
        let _ = GuardrailContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);

        let msg = Some(bvs_vault_factory::msg::InstantiateMsg {
            pauser: pauser.addr().to_string(),
            registry: registry.addr().to_string(),
            router: vault_router.addr().to_string(),
            owner: app.api().addr_make("owner").to_string(),
        });
        let vault_factory = VaultFactoryContract::new(&mut app, &env, msg);

        let bank_wrapper = VaultBankContract::wrapper();
        let cw20_vault_wrapper = VaultCw20Contract::wrapper();
        let bank_tokenized_wrapper =
            bvs_vault_bank_tokenized::testing::VaultBankTokenizedContract::wrapper();
        let cw20_tokenized_wrapper =
            bvs_vault_cw20_tokenized::testing::VaultCw20TokenizedContract::wrapper();

        (
            app,
            Self {
                registry,
                cw20_token: cw20,
                vault_factory,
                bank_wrapper,
                bank_tokenized_wrapper,
                cw20_vault_wrapper,
                cw20_tokenized_wrapper,
                pauser,
                router: vault_router,
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

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: cw20_vault_code_id,
        vault_type: VaultType::Cw20,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20 {
        cw20: cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg).unwrap();

    let event = res.events.iter().find(|e| e.ty == "instantiate");

    assert!(event.is_some());

    let vault_addr = event
        .unwrap()
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.key == "_contract_address" {
                Some(attr.value.clone())
            } else {
                None
            }
        })
        .unwrap();

    let query_res: bvs_vault_base::msg::VaultInfoResponse = app
        .wrap()
        .query_wasm_smart(&vault_addr, &bvs_vault_cw20::msg::QueryMsg::VaultInfo {})
        .unwrap();

    assert_eq!(query_res.router, contracts.router.addr());
    assert_eq!(query_res.pauser, contracts.pauser.addr());
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

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: bank_vault_code_id,
        vault_type: VaultType::Bank,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBank {
        denom: "SATL".to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg).unwrap();

    let event = res.events.iter().find(|e| e.ty == "instantiate");

    assert!(event.is_some());

    let vault_addr = event
        .unwrap()
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.key == "_contract_address" {
                Some(attr.value.clone())
            } else {
                None
            }
        })
        .unwrap();

    let query_res: bvs_vault_base::msg::VaultInfoResponse = app
        .wrap()
        .query_wasm_smart(&vault_addr, &bvs_vault_cw20::msg::QueryMsg::VaultInfo {})
        .unwrap();

    assert_eq!(query_res.router, contracts.router.addr());
    assert_eq!(query_res.pauser, contracts.pauser.addr());
}

#[test]
fn test_bank_tokenized_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_tokenized_wrapper;

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

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: bank_vault_code_id,
        vault_type: VaultType::BankTokenized,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBankTokenized {
        denom: "SATL".to_string(),
        decimals: 6,
        symbol: "satl".to_string(),
        name: "Satlayer test receipt token".to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg).unwrap();

    let event = res.events.iter().find(|e| e.ty == "instantiate");

    assert!(event.is_some());

    let vault_addr = event
        .unwrap()
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.key == "_contract_address" {
                Some(attr.value.clone())
            } else {
                None
            }
        })
        .unwrap();

    let query_res: bvs_vault_base::msg::VaultInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &vault_addr,
            &bvs_vault_bank_tokenized::msg::QueryMsg::VaultInfo {},
        )
        .unwrap();

    assert_eq!(query_res.router, contracts.router.addr());
    assert_eq!(query_res.pauser, contracts.pauser.addr());

    let query_res: cw20::TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &vault_addr,
            &bvs_vault_cw20_tokenized::msg::QueryMsg::TokenInfo {},
        )
        .unwrap();

    assert_eq!(query_res.decimals, 6);
    assert_eq!(query_res.symbol, "satl");
    assert_eq!(query_res.name, "Satlayer test receipt token");
}

#[test]
fn test_negative_bank_tokenized_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_tokenized_wrapper;

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

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: bank_vault_code_id,
        vault_type: VaultType::BankTokenized,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBankTokenized {
        denom: "SATL".to_string(),
        decimals: 6,
        symbol: "notSatPrefixed".to_string(),
        name: "Satlayer test receipt token".to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg);

    assert!(res.is_err());
}

#[test]
fn test_cw20_tokenized_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let cw20_vault = contracts.cw20_tokenized_wrapper;

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

    let cw20_vault_code_id = app.store_code(cw20_vault);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: cw20_vault_code_id,
        vault_type: VaultType::Cw20Tokenized,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20Tokenized {
        symbol: "satl".to_string(),
        name: "Satlayer test receipt token".to_string(),
        cw20: contracts.cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg).unwrap();

    let event = res.events.iter().find(|e| e.ty == "instantiate");

    assert!(event.is_some());

    let vault_addr = event
        .unwrap()
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.key == "_contract_address" {
                Some(attr.value.clone())
            } else {
                None
            }
        })
        .unwrap();

    let query_res: bvs_vault_base::msg::VaultInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &vault_addr,
            &bvs_vault_cw20_tokenized::msg::QueryMsg::VaultInfo {},
        )
        .unwrap();

    assert_eq!(query_res.router, contracts.router.addr());
    assert_eq!(query_res.pauser, contracts.pauser.addr());

    let query_res: cw20::TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &vault_addr,
            &bvs_vault_cw20_tokenized::msg::QueryMsg::TokenInfo {},
        )
        .unwrap();
    assert_eq!(query_res.decimals, 18);
    assert_eq!(query_res.symbol, "satl");
    assert_eq!(query_res.name, "Satlayer test receipt token");
}

#[test]
fn test_negative_cw20_tokenized_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let cw20_vault = contracts.cw20_tokenized_wrapper;

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

    let cw20_vault_code_id = app.store_code(cw20_vault);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: cw20_vault_code_id,
        vault_type: VaultType::Cw20Tokenized,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20Tokenized {
        symbol: "notSatPrefixed".to_string(),
        name: "Satlayer test receipt token".to_string(),
        cw20: contracts.cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg);
    assert!(res.is_err());
}

#[test]
fn test_unauthorized_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBank {
        denom: "SATL".to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap_err();

    assert_eq!(
        res.root_cause().to_string(),
        bvs_vault_factory::ContractError::Unauthorized {}.to_string()
    );

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20 {
        cw20: contracts.cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap_err();

    assert_eq!(
        res.root_cause().to_string(),
        bvs_vault_factory::ContractError::Unauthorized {}.to_string()
    );
}

#[test]
fn test_unauthorized_code_id_whitelist() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;

    let bank_vault_code_id = app.store_code(bank_vault);

    let random = app.api().addr_make("random");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: bank_vault_code_id,
        vault_type: VaultType::Bank,
    };

    let res = factory.execute(&mut app, &random, &msg).unwrap_err();

    assert!(matches!(
        res.downcast_ref::<bvs_vault_factory::ContractError>(),
        Some(bvs_vault_factory::ContractError::Ownership(..))
    ));

    let err = factory
        .query::<u64>(
            &app,
            &bvs_vault_factory::msg::QueryMsg::CodeId {
                vault_type: VaultType::Bank,
            },
        )
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: Querier contract error: Code id not found"
    );
}

#[test]
fn test_set_code_id() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let bank_vault = contracts.bank_wrapper;

    let bank_vault_code_id = app.store_code(bank_vault);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: bank_vault_code_id,
        vault_type: VaultType::Bank,
    };

    let _res = factory.execute(&mut app, &owner, &msg).unwrap();

    let code_id: u64 = factory
        .query(
            &app,
            &bvs_vault_factory::msg::QueryMsg::CodeId {
                vault_type: VaultType::Bank,
            },
        )
        .unwrap();

    assert_eq!(code_id, bank_vault_code_id);
}
