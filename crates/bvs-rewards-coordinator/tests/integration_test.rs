use bvs_rewards_coordinator::testing::RewardsContract;
use bvs_rewards_coordinator::{
    msg::{ExecuteMsg, InstantiateMsg},
    testing,
};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_json_binary, Event, StdError, WasmMsg};
use cw_multi_test::{App, Executor};

fn instantiate() -> (App, RewardsContract) {
    let mut app = App::default();
    let env = mock_env();

    let code_id = app.store_code(bvs_registry::testing::contract());
    let registry = bvs_registry::testing::instantiate(&mut app, code_id, None);

    let code_id = app.store_code(testing::contract());

    let delegate_manager = app.api().addr_make("delegation_manager");
    let strategy_manager = app.api().addr_make("strategy_manager");
    let init_msg = InstantiateMsg::default(
        &mut app,
        &env,
        &registry.addr,
        &delegate_manager,
        &strategy_manager,
    );

    let (addr, msg) = testing::instantiate(&mut app, code_id, init_msg);
    (
        app,
        RewardsContract {
            addr,
            init: msg,
            registry,
        },
    )
}

#[test]
fn set_rewards_updater() {
    let (mut app, contract) = instantiate();

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
    let (mut app, contract) = instantiate();
    contract.registry.pause(&mut app);

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
