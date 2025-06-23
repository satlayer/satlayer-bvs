use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_factory::msg::VaultType;
use bvs_vault_factory::testing::VaultFactoryContract;
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{testing::mock_env, Empty};
use cw_multi_test::{App, Contract};

pub mod old_contract {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::entry_point;
    use cosmwasm_std::StdError;
    use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
    use cw_multi_test::{Contract, ContractWrapper};
    use cw_storage_plus::Item;
    use thiserror::Error;

    static FOO: Item<String> = Item::new("foo");

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: bvs_vault_bank::msg::InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, "mocked_empty_contract", "1.0.0")?;

        let s = String::from("Foo");

        FOO.save(deps.storage, &s)?;

        Ok(Response::new())
    }

    #[derive(Error, Debug)]
    pub enum ContractError {
        #[error("{0}")]
        Std(#[from] StdError),
    }

    #[cw_serde]
    pub enum QueryMsg {}

    #[cw_serde]
    pub enum ExecuteMsg {}

    #[cw_serde]
    pub struct MigrateMsg {}

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, ContractError> {
        Err(ContractError::Std(StdError::generic_err("Not implemented")))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Err(ContractError::Std(StdError::generic_err("Not implemented")))
    }

    pub fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(execute, instantiate, query))
    }
}

pub mod new_contract {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use cosmwasm_std::entry_point;
    use cosmwasm_std::StdError;
    use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
    use cw_multi_test::{Contract, ContractWrapper};
    use cw_storage_plus::Item;
    use thiserror::Error;

    static FOO: Item<String> = Item::new("foo");
    static BAR: Item<String> = Item::new("bar");

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: bvs_vault_bank::msg::InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, "mocked_empty_contract", "1.0.0")?;

        let s = String::from("Foo");

        FOO.save(deps.storage, &s)?;

        Ok(Response::new())
    }

    #[derive(Error, Debug)]
    pub enum ContractError {
        #[error("{0}")]
        Std(#[from] StdError),
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(ShowStatesResponse)]
        ShowStates {},
    }

    #[cw_serde]
    pub struct ShowStatesResponse {
        pub states: String,
    }

    #[cw_serde]
    pub enum ExecuteMsg {}

    #[cw_serde]
    pub struct MigrateMsg {}

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, ContractError> {
        match _msg {
            QueryMsg::ShowStates {} => {
                let var_foo = FOO.load(_deps.storage)?;
                let var_bar = BAR.load(_deps.storage)?;
                let resp = ShowStatesResponse {
                    states: format!("Foo: {}, Bar: {}", var_foo, var_bar),
                };

                Ok(to_json_binary(&resp)?)
            }
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Err(ContractError::Std(StdError::generic_err("Not implemented")))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
        cw2::ensure_from_older_version(deps.storage, "mocked_empty_contract", "2.0.0")?;

        BAR.save(deps.storage, &String::from("Bar"))?;

        Ok(Response::new())
    }

    pub fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(execute, instantiate, query).with_migrate(migrate))
    }
}

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
fn test_migrate_vault() {
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

    // old contract is a blank mocked contract with arbitrary early version
    // old contract only has `FOO` state at this point
    // later in this test we will migrate to new contract that has added `BAR` state
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

    // new contract is pretty much identical to old contract
    // except it has an additional `BAR` state
    // and migration logic with version 2.0.0
    // (old contract is version 1.0.0)
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
