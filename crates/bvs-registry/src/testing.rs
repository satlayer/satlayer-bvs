use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegistryContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RegistryContract {
    fn new_wrapper() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn setup(app: &mut App, _env: &Env, msg: Option<InstantiateMsg>) -> RegistryContract {
        let code_id = app.store_code(Self::new_wrapper());
        let init = msg.unwrap_or(InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
            initial_paused: false,
        });

        let addr = Self::instantiate(app, code_id, &init);
        RegistryContract { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
