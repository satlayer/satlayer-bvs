#![cfg(not(target_arch = "wasm32"))]

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

pub const ONE_DAY: u64 = 86_400;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsCoordinatorContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RewardsCoordinatorContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, env: &Env) -> InstantiateMsg {
        let owner = app.api().addr_make("owner");
        let registry = Self::get_contract_addr(app, "registry").to_string();

        let today_rounded_down = env.block.time.seconds() / ONE_DAY * ONE_DAY;
        InstantiateMsg {
            owner: owner.to_string(),
            registry,
            calculation_interval_seconds: ONE_DAY,
            max_rewards_duration: 30 * ONE_DAY,
            max_retroactive_length: 5 * ONE_DAY,
            max_future_length: 10 * ONE_DAY,
            genesis_rewards_timestamp: today_rounded_down,
            activation_delay: 60,
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "rewards-coordinator", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
