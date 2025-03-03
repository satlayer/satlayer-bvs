use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StrategyManagerContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for StrategyManagerContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        let owner = app.api().addr_make("owner").to_string();
        let registry = app.api().addr_make("registry").to_string();
        InstantiateMsg {
            owner: owner.clone(),
            registry: registry.clone(),
            initial_strategy_whitelister: owner.clone(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "Strategy Manager Contract", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
