use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RewardsContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, env: &Env) -> InstantiateMsg {
        let owner = app.api().addr_make("owner");
        InstantiateMsg {
            initial_owner: owner.to_string(),
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            activation_delay: 60,
            delegation_manager: Addr::unchecked("").to_string(),
            rewards_updater: Addr::unchecked("").to_string(),
            strategy_manager: Addr::unchecked("").to_string(),
            registry: Addr::unchecked("").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
