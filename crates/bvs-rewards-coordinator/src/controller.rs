use crate::ContractError;
use cosmwasm_std::{Addr, Deps, DepsMut, Event, MessageInfo, Response};
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
pub const REWARDS_UPDATER: Item<Addr> = Item::new("rewards_updater");

pub fn set_controller(
    deps: DepsMut,
    info: &MessageInfo,
    owner: Addr,
    rewards_updater: Addr,
    delegation_manager: Addr,
    strategy_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), info)?;

    OWNER.save(deps.storage, &owner)?;
    REWARDS_UPDATER.save(deps.storage, &rewards_updater)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetController")
            .add_attribute("owner", owner)
            .add_attribute("rewards_updater", rewards_updater)
            .add_attribute("delegation_manager", delegation_manager)
            .add_attribute("strategy_manager", strategy_manager),
    ))
}

pub fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn only_rewards_updater(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let rewards_updater = REWARDS_UPDATER.load(deps.storage)?;
    if info.sender != rewards_updater {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::controller::REWARDS_UPDATER;
    use crate::{controller, ContractError};
    use controller::{only_owner, only_rewards_updater, OWNER};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::Addr;

    #[test]
    fn test_only_owner() {
        let mut deps = mock_dependencies();

        let owner_addr = deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let owner_info = message_info(&owner_addr, &[]);

        let result = only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = only_owner(deps.as_ref(), &info);
        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }

    #[test]
    fn test_only_owner_fail() {
        let mut deps = mock_dependencies();

        let owner_addr = deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let owner_info = message_info(&owner_addr, &[]);

        let result = only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = only_owner(deps.as_ref(), &info);
        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }

    #[test]
    fn test_only_rewards_updater() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(deps.as_mut().storage, &rewards_updater_addr)
            .unwrap();

        let info = message_info(&rewards_updater_addr, &[]);
        let result = only_rewards_updater(deps.as_ref(), &info);

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
        let result = only_rewards_updater(deps.as_ref(), &info);

        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }
}
