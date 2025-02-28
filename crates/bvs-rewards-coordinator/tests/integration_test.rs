use bvs_library::testing::TestingContract;
use bvs_registry::api::RegistryError;
use bvs_registry::testing::RegistryContract;
use bvs_rewards_coordinator::msg::ExecuteMsg;
use bvs_rewards_coordinator::testing::RewardsContract;
use bvs_rewards_coordinator::ContractError;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event};
use cw_multi_test::App;

fn instantiate() -> (App, RewardsContract, RegistryContract) {
    let mut app = App::default();
    let env = mock_env();

    let registry = RegistryContract::new(&mut app, &env, None);
    let rewards = RewardsContract::new(&mut app, &env, None);
    (app, rewards, registry)
}

#[test]
fn set_activation_delay() {
    let (mut app, rewards, _) = instantiate();
    let owner = app.api().addr_make("owner");

    let msg = ExecuteMsg::SetActivationDelay {
        new_activation_delay: 100,
    };
    let res = rewards.execute(&mut app, &owner, &msg).unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-ActivationDelaySet")
            .add_attribute("_contract_address", rewards.addr.to_string())
            .add_attribute("old_activation_delay", "60")
            .add_attribute("new_activation_delay", "100")
    );
}

#[test]
fn set_activation_delay_but_paused() {
    let (mut app, rewards, registry) = instantiate();
    let owner = Addr::unchecked(registry.init.owner.clone());

    registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let msg = ExecuteMsg::SetActivationDelay {
        new_activation_delay: 100,
    };
    let err = rewards.execute(&mut app, &owner, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Registry(RegistryError::IsPaused).to_string()
    );
}

#[test]
fn set_routing() {
    let (mut app, rewards, registry) = instantiate();

    let delegation_manager = app.api().addr_make("delegation_manager");
    let strategy_manager = app.api().addr_make("strategy_manager");
    let msg = ExecuteMsg::SetRouting {
        delegation_manager: delegation_manager.to_string(),
        strategy_manager: strategy_manager.to_string(),
    };

    let owner = Addr::unchecked(registry.init.owner.clone());
    rewards.execute(&mut app, &owner, &msg).unwrap();
}

#[test]
fn set_routing_not_owner() {
    let (mut app, rewards, _registry) = instantiate();

    let delegation_manager = app.api().addr_make("delegation_manager");
    let strategy_manager = app.api().addr_make("strategy_manager");
    let msg = ExecuteMsg::SetRouting {
        delegation_manager: delegation_manager.to_string(),
        strategy_manager: strategy_manager.to_string(),
    };

    let not_owner = app.api().addr_make("not_owner");
    let err = rewards.execute(&mut app, &not_owner, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Ownership(bvs_library::ownership::OwnershipError::Unauthorized).to_string()
    );
}

#[test]
fn set_rewards_updater() {
    let (mut app, rewards, _) = instantiate();
    let owner = app.api().addr_make("owner");

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        addr: new_updater.to_string(),
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
    let (mut app, rewards, registry) = instantiate();
    let owner = Addr::unchecked(rewards.init.owner.clone());

    registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let new_updater = app.api().addr_make("new_updater");
    let msg = ExecuteMsg::SetRewardsUpdater {
        addr: new_updater.to_string(),
    };
    let err = rewards.execute(&mut app, &owner, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Registry(RegistryError::IsPaused).to_string()
    );
}
