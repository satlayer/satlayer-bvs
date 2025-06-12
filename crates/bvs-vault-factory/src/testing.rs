#![cfg(not(target_arch = "wasm32"))]
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VaultFactoryContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for VaultFactoryContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
            pauser: Self::get_contract_addr(app, "pauser").to_string(),
            router: Self::get_contract_addr(app, "vault_router").to_string(),
            registry: Self::get_contract_addr(app, "registry").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "vault_factory", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}

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
                let foo = FOO.load(_deps.storage)?;
                let bar = BAR.load(_deps.storage)?;
                let resp = ShowStatesResponse {
                    states: format!("Foo: {}, Bar: {}", foo, bar),
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
