use crate::msg::InstantiateMsg;
use bvs_library::testing::TestingContract;
use bvs_registry::msg::{ExecuteMsg, QueryMsg};
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RewardsContract {
    fn new_wrapper() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn setup(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> RewardsContract {
        let code_id = app.store_code(Self::new_wrapper());

        let owner = app.api().addr_make("owner");
        let init = msg.unwrap_or(InstantiateMsg {
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
        });

        let addr = Self::instantiate(app, code_id, &init);
        RewardsContract { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
