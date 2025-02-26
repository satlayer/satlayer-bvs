use crate::msg::InstantiateMsg;
use bvs_registry::testing::RegistryContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
    pub registry: RegistryContract,
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

impl InstantiateMsg {
    pub fn default(app: &mut App, env: &Env, registry: &Addr) -> Self {
        let owner = app.api().addr_make("owner");

        Self {
            owner: owner.to_string(),
            registry: registry.to_string(),
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            activation_delay: 60,
        }
    }
}

pub fn instantiate(app: &mut App, code_id: u64, msg: InstantiateMsg) -> (Addr, InstantiateMsg) {
    let addr = app
        .instantiate_contract(
            code_id,
            app.api().addr_make("sender"),
            &msg,
            &[],
            "BVS Rewards Coordinator",
            None,
        )
        .unwrap();

    (addr, msg)
}
