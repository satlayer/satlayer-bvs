use bvs_library::testing::TestingContract;
use bvs_registry::api::RegistryError;
use bvs_registry::testing::RegistryContract;
use bvs_rewards_coordinator::msg::ExecuteMsg;
use bvs_rewards_coordinator::testing::RewardsContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, StdError};
use cw_multi_test::App;

fn instantiate() -> (App, RegistryContract, RewardsContract) {
    let mut app = App::default();
    let env = mock_env();

    let registry = RegistryContract::new(&mut app, &env, None);
    let rewards = RewardsContract::new(&mut app, &env, None);
    (app, registry, rewards)
}

#[test]
fn set_rewards_updater() {
    let (mut app, _, rewards) = instantiate();
    let owner = app.api().addr_make("owner");

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        new_updater: new_updater.to_string(),
    };
    let res = rewards.execute(&mut app, &owner, &msg).unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-SetRewardsUpdater")
            .add_attribute("_contract_address", rewards.addr.to_string())
            .add_attribute("method", "set_rewards_updater")
            .add_attribute("new_updater", new_updater.to_string())
    );
}

#[test]
fn set_rewards_updater_but_paused() {
    let (mut app, registry, rewards) = instantiate();
    let owner = Addr::unchecked(registry.init.owner.clone());

    registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        new_updater: new_updater.to_string(),
    };
    let err = rewards.execute(&mut app, &owner, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        bvs_rewards_coordinator::ContractError::RegistryError(RegistryError::IsPaused).to_string()
    );
}
