use cosmwasm_std::Empty;
use cw_multi_test::{App, Contract, ContractWrapper};

pub fn mock_app() -> App {
    App::default()
}

pub fn mock_bvs_delegation_manager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_delegation_manager::contract::execute,
        bvs_delegation_manager::contract::instantiate,
        bvs_delegation_manager::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_directory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_directory::contract::execute,
        bvs_directory::contract::instantiate,
        bvs_directory::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_driver() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_driver::contract::execute,
        bvs_driver::contract::instantiate,
        bvs_driver::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_rewards_coordinator() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_rewards_coordinator::contract::execute,
        bvs_rewards_coordinator::contract::instantiate,
        bvs_rewards_coordinator::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_slash_manager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_slash_manager::contract::execute,
        bvs_slash_manager::contract::instantiate,
        bvs_slash_manager::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_state_bank() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_state_bank::contract::execute,
        bvs_state_bank::contract::instantiate,
        bvs_state_bank::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_strategy_base() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_strategy_base::contract::execute,
        bvs_strategy_base::contract::instantiate,
        bvs_strategy_base::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_strategy_base_tvl_limits() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_strategy_base_tvl_limits::contract::execute,
        bvs_strategy_base_tvl_limits::contract::instantiate,
        bvs_strategy_base_tvl_limits::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_strategy_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_strategy_factory::contract::execute,
        bvs_strategy_factory::contract::instantiate,
        bvs_strategy_factory::contract::query,
    );
    Box::new(contract)
}

pub fn mock_bvs_strategy_manager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_strategy_manager::contract::execute,
        bvs_strategy_manager::contract::instantiate,
        bvs_strategy_manager::contract::query,
    );
    Box::new(contract)
}
