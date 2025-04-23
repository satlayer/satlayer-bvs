#![cfg(not(target_arch = "wasm32"))]

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegistryContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg> for RegistryContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_migrate(crate::contract::migrate),
        )
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
            pauser: Self::get_contract_addr(app, "pauser").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "registry", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
