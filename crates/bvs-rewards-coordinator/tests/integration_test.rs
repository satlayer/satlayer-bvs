use bvs_library::testing::TestingContract;
use bvs_registry::testing::RegistryContract;
use bvs_rewards_coordinator::msg::ExecuteMsg;
use bvs_rewards_coordinator::testing::RewardsContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_json_binary, Addr, Event, StdError, WasmMsg};
use cw_multi_test::{App, Executor};

fn instantiate() -> (App, RegistryContract, RewardsContract) {
    let mut app = App::default();
    let env = mock_env();

    let registry = RegistryContract::setup(&mut app, &env, None);
    let rewards = RewardsContract::setup(&mut app, &env, None);
    (app, registry, rewards)
}

#[test]
fn set_rewards_updater() {
    let (mut app, _, contract) = instantiate();

    let new_updater = app.api().addr_make("new_updater");
    let msg = to_json_binary(&ExecuteMsg::SetRewardsUpdater {
        new_updater: new_updater.to_string(),
    });
    let execute_msg = WasmMsg::Execute {
        contract_addr: contract.addr.to_string(),
        msg: msg.unwrap(),
        funds: vec![],
    };

    let owner = app.api().addr_make("owner");
    let res = app.execute(owner, execute_msg.into()).unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-SetRewardsUpdater")
            .add_attribute("_contract_address", contract.addr.to_string())
            .add_attribute("method", "set_rewards_updater")
            .add_attribute("new_updater", new_updater.to_string())
    );
}

#[test]
fn set_rewards_updater_but_paused() {
    let (mut app, registry, contract) = instantiate();
    let sender = Addr::unchecked(registry.init.owner.clone());
    registry.execute(&mut app, &sender, &bvs_registry::msg::ExecuteMsg::Pause {});

    let new_updater = app.api().addr_make("new_updater");
    let msg = to_json_binary(&ExecuteMsg::SetRewardsUpdater {
        new_updater: new_updater.to_string(),
    });
    let execute_msg = WasmMsg::Execute {
        contract_addr: contract.addr.to_string(),
        msg: msg.unwrap(),
        funds: vec![],
    };

    let owner = app.api().addr_make("owner");
    let err = app.execute(owner, execute_msg.into()).unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        bvs_rewards_coordinator::ContractError::Std(StdError::generic_err("Paused")).to_string()
    );
}
