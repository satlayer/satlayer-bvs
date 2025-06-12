use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_factory::msg::VaultType;
use bvs_vault_factory::testing::{new_contract, old_contract, VaultFactoryContract};
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{testing::mock_env, Empty};
use cw_multi_test::{App, Contract};

struct TestContracts {
    vault_factory: VaultFactoryContract,
    registry: RegistryContract,
    pauser: PauserContract,
    router: VaultRouterContract,
    old_contract_wrapper: Box<dyn Contract<Empty>>,
    new_contract_wrapper: Box<dyn Contract<Empty>>,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::default();
        let env = mock_env();

        let pauser = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);

        let msg = Some(bvs_vault_factory::msg::InstantiateMsg {
            pauser: pauser.addr().to_string(),
            registry: registry.addr().to_string(),
            router: vault_router.addr().to_string(),
            owner: app.api().addr_make("owner").to_string(),
        });
        let vault_factory = VaultFactoryContract::new(&mut app, &env, msg);
        let old_contract_wrapper = old_contract::wrapper();
        let new_contract_wrapper = new_contract::wrapper();

        (
            app,
            Self {
                registry,
                vault_factory,
                pauser,
                router: vault_router,
                old_contract_wrapper,
                new_contract_wrapper,
            },
        )
    }
}

#[test]
fn test_bank_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let factory = contracts.vault_factory;
    let old_contract = contracts.old_contract_wrapper;

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

    let old_contract_code_id = app.store_code(old_contract);

    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
        code_id: old_contract_code_id,
        vault_type: VaultType::Bank,
    };

    factory.execute(&mut app, &owner, &msg).unwrap();

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployBank {
        denom: "SATL".to_string(),
    };

    let res = factory.execute(&mut app, &operator, &msg).unwrap();

    let event = res.events.iter().find(|e| e.ty == "instantiate");

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

    let new_contract_code_id = app.store_code(contracts.new_contract_wrapper);

    factory
        .execute(
            &mut app,
            &owner,
            &bvs_vault_factory::msg::ExecuteMsg::SetCodeId {
                vault_type: VaultType::Bank,
                code_id: new_contract_code_id,
            },
        )
        .unwrap();

    let inner_opaque_migrate_msg = bvs_vault_bank::msg::MigrateMsg {};

    let upgrade_msg = bvs_vault_factory::msg::ExecuteMsg::MigrateVault {
        vault_address: vault_addr.clone(),
        vault_type: VaultType::Bank,
        migrate_msg: to_json_binary(&inner_opaque_migrate_msg).unwrap(),
    };

    factory.execute(&mut app, &owner, &upgrade_msg).unwrap();

    let query_msg = new_contract::QueryMsg::ShowStates {};

    let query_res: new_contract::ShowStatesResponse = app
        .wrap()
        .query_wasm_smart(&vault_addr, &query_msg)
        .unwrap();

    assert_eq!(query_res.states, "Foo: Foo, Bar: Bar");
}
