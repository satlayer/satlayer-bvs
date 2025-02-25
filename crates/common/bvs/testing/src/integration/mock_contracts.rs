use cosmwasm_std::{testing::MockApi, Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper};

pub fn mock_app() -> App {
    AppBuilder::new()
        .with_api(MockApi::default().with_prefix("bbn"))
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("admin"),
                    vec![Coin::new(Uint128::new(100), "ubbn")],
                )
                .unwrap();
        })
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

pub fn mock_bvs_registry() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_registry::contract::execute,
        bvs_registry::contract::instantiate,
        bvs_registry::contract::query,
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

pub fn mock_bvs_strategy_base() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        bvs_strategy_base::contract::execute,
        bvs_strategy_base::contract::instantiate,
        bvs_strategy_base::contract::query,
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
