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
        let registry = Self::get_contract_addr(app, "registry").to_string();

        let delegation_manager = app.api().addr_make("delegation-manager");
        let rewards_updater = app.api().addr_make("rewards-updater");
        let strategy_manager = app.api().addr_make("strategy-manager");
        InstantiateMsg {
            initial_owner: owner.to_string(),
            registry,
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            activation_delay: 60,
            delegation_manager: delegation_manager.to_string(),
            rewards_updater: rewards_updater.to_string(),
            strategy_manager: strategy_manager.to_string(),
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
