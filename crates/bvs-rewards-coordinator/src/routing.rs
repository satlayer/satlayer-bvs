use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{Addr, Deps, DepsMut, Event, MessageInfo, Response};
use cw_storage_plus::Item;

pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
pub const REWARDS_UPDATER: Item<Addr> = Item::new("rewards_updater");

/// Contract Control Plane, it defines how the contract messages get routed.
pub fn set_routing(
    deps: DepsMut,
    info: &MessageInfo,
    rewards_updater: Addr,
    delegation_manager: Addr,
    strategy_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.as_ref(), info)?;

    REWARDS_UPDATER.save(deps.storage, &rewards_updater)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetRouting")
            .add_attribute("rewards_updater", rewards_updater)
            .add_attribute("delegation_manager", delegation_manager)
            .add_attribute("strategy_manager", strategy_manager),
    ))
}

pub fn assert_rewards_updater(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let rewards_updater = REWARDS_UPDATER.load(deps.storage)?;
    if info.sender != rewards_updater {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::routing::REWARDS_UPDATER;
    use crate::{routing, ContractError};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use routing::assert_rewards_updater;

    #[test]
    fn test_only_rewards_updater() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(deps.as_mut().storage, &rewards_updater_addr)
            .unwrap();

        let info = message_info(&rewards_updater_addr, &[]);
        let result = assert_rewards_updater(deps.as_ref(), &info);

        assert!(result.is_ok());
    }

    #[test]
    fn test_only_rewards_updater_fail() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(&mut deps.storage, &rewards_updater_addr)
            .unwrap();

        let rewards_updater_addr = deps.api.addr_make("not_rewards_updater");
        let info = message_info(&rewards_updater_addr, &[]);
        let result = assert_rewards_updater(deps.as_ref(), &info);

        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }
}
