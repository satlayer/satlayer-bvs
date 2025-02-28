use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response, Storage};
use cw_storage_plus::Item;

const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
const REWARDS_UPDATER: Item<Addr> = Item::new("rewards_updater");

/// Contract Control Plane, it defines how the contract messages get routed.
/// While instantiate creates the contract: gives the contract an address.
/// This sets up the contract for routing and access control management.
/// It can be called more than once to set new values but only by the owner.
pub fn set_routing(
    deps: DepsMut,
    info: MessageInfo,
    strategy_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    Ok(Response::new()
        .add_event(Event::new("SetRouting").add_attribute("strategy_manager", strategy_manager)))
}

pub fn get_strategy_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    STRATEGY_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

pub fn set_rewards_updater(
    deps: DepsMut,
    info: MessageInfo,
    new_updater: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    REWARDS_UPDATER.save(deps.storage, &new_updater)?;

    Ok(Response::new()
        .add_event(Event::new("SetRewardsUpdater").add_attribute("addr", new_updater.as_str())))
}

pub fn assert_rewards_updater(
    storage: &dyn Storage,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let rewards_updater = REWARDS_UPDATER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})?;
    if info.sender != rewards_updater {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::auth::{set_routing, REWARDS_UPDATER};
    use crate::{auth, ContractError};
    use auth::assert_rewards_updater;
    use bvs_library::ownership::{self, OwnershipError};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::{Event, Response};

    #[test]
    fn test_set_routing() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let owner_info = message_info(owner_addr, &[]);

        let new_strategy_manager = deps.api.addr_make("new_strategy_manager");

        let res = set_routing(deps.as_mut(), owner_info, new_strategy_manager.clone()).unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRouting")
                    .add_attribute("strategy_manager", new_strategy_manager.to_string())
            )
        );
    }

    #[test]
    fn test_set_routing_not_authorized() {
        let mut deps = mock_dependencies();

        {
            // Setup Owner
            let owner_addr = &deps.api.addr_make("owner");
            ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();
        }

        let new_strategy_manager = deps.api.addr_make("new_strategy_manager");

        let sender = &deps.api.addr_make("random_sender");
        let sender_info = message_info(sender, &[]);

        let err =
            set_routing(deps.as_mut(), sender_info, new_strategy_manager.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            ContractError::Ownership(OwnershipError::Unauthorized).to_string()
        );
    }

    #[test]
    fn test_set_rewards_updater() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();
        let owner_info = message_info(owner_addr, &[]);

        let new_updater = deps.api.addr_make("new_updater");
        let res =
            auth::set_rewards_updater(deps.as_mut(), owner_info, new_updater.clone()).unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRewardsUpdater").add_attribute("addr", new_updater.as_str())
            )
        );

        let loaded_addr = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(loaded_addr, new_updater);
    }

    #[test]
    fn test_set_rewards_updater_not_owner() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(deps.as_mut().storage, &rewards_updater_addr)
            .unwrap();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let sender = &deps.api.addr_make("not_owner");
        let sender_info = message_info(sender, &[]);
        let new_updater = deps.api.addr_make("new_updater");

        let err =
            auth::set_rewards_updater(deps.as_mut(), sender_info, new_updater.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            ContractError::Ownership(OwnershipError::Unauthorized).to_string()
        );

        let stored_addr = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_ne!(stored_addr, new_updater);
    }

    #[test]
    fn test_assert_rewards_updater() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(deps.as_mut().storage, &rewards_updater_addr)
            .unwrap();

        let info = message_info(&rewards_updater_addr, &[]);
        let result = assert_rewards_updater(&deps.storage, &info);

        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_rewards_updater_fail() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");
        REWARDS_UPDATER
            .save(&mut deps.storage, &rewards_updater_addr)
            .unwrap();

        let rewards_updater_addr = deps.api.addr_make("not_rewards_updater");
        let info = message_info(&rewards_updater_addr, &[]);
        let result = assert_rewards_updater(&deps.storage, &info);

        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }
}
