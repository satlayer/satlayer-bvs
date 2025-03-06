#![cfg(not(target_arch = "wasm32"))]

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DelegationManagerContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for DelegationManagerContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _: &Env) -> InstantiateMsg {
        let owner = app.api().addr_make("owner").to_string();
        let strategy1 = app.api().addr_make("strategy1").to_string();
        let strategy2 = app.api().addr_make("strategy2").to_string();
        let registry = Self::get_contract_addr(app, "registry").to_string();

        InstantiateMsg {
            owner,
            registry,
            min_withdrawal_delay_blocks: 5,
            strategies: vec![strategy1, strategy2],
            withdrawal_delay_blocks: vec![50, 60],
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "delegation-manager", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
