use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{Addr, Deps, DepsMut, Event, MessageInfo, Response, Storage};
use cw_storage_plus::Item;

pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const SLASH_MANAGER: Item<Addr> = Item::new("slash_manager");

/// Contract Control Plane, it defines how the contract messages get routed.
/// While instantiate creates the contract: gives the contract an address.
/// This sets up the contract for routing and access control management.
/// It can be called more than once to set new values but only by the owner.
pub fn set_routing(
    deps: DepsMut,
    info: MessageInfo,
    delegation_manager: Addr,
    slash_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.as_ref(), &info)?;

    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    SLASH_MANAGER.save(deps.storage, &slash_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetRouting")
            .add_attribute("delegation_manager", delegation_manager)
            .add_attribute("slash_manager", slash_manager),
    ))
}

/// Get the Delegation Manager address
/// If SetRouting has not been called, it will return an Unauthorized error
pub fn get_delegation_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    DELEGATION_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

/// Get the Slash Manager address
/// If SetRouting has not been called, it will return an Unauthorized error
/// Currently, not used to be migrated and release in the future for Slash Manager
#[allow(dead_code)]
pub fn get_slash_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    SLASH_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

/// Assert that the sender is the Delegation Manager
pub fn assert_delegation_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let delegation_manager = DELEGATION_MANAGER
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if info.sender != delegation_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

/// Currently, not used to be migrated and release in the future for Slash Manager
#[allow(dead_code)]
pub fn assert_delegation_or_slash_manager(
    deps: Deps,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let delegation_manager = DELEGATION_MANAGER
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if info.sender == delegation_manager {
        return Ok(());
    }

    let slash_manager = SLASH_MANAGER
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if info.sender == slash_manager {
        return Ok(());
    }

    Err(ContractError::Unauthorized {})
}

#[cfg(test)]
mod tests {
    use crate::auth::{set_routing, DELEGATION_MANAGER, SLASH_MANAGER};
    use crate::{auth, ContractError};
    use bvs_library::ownership::{self, OwnershipError};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::{Event, Response};

    #[test]
    fn test_set_routing() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let owner_info = message_info(owner_addr, &[]);

        let new_delegation_manager = deps.api.addr_make("delegation_manager/10");
        let new_slash_manager = deps.api.addr_make("slash_manager/10");

        let res = set_routing(
            deps.as_mut(),
            owner_info,
            new_delegation_manager.clone(),
            new_slash_manager.clone(),
        )
        .unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRouting")
                    .add_attribute("delegation_manager", new_delegation_manager.as_str())
                    .add_attribute("slash_manager", new_slash_manager.as_str())
            )
        );
    }

    #[test]
    fn test_set_routing_not_authorized() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let new_delegation_manager = deps.api.addr_make("delegation_manager/15");
        let new_slash_manager = deps.api.addr_make("slash_manager/15");

        let sender = &deps.api.addr_make("random_sender");
        let sender_info = message_info(sender, &[]);

        let err = set_routing(
            deps.as_mut(),
            sender_info,
            new_delegation_manager.clone(),
            new_slash_manager.clone(),
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            ContractError::Ownership(OwnershipError::Unauthorized).to_string()
        );
    }

    #[test]
    fn test_get_delegation_manager() {
        let mut deps = mock_dependencies();

        let delegation_manager_addr = deps.api.addr_make("delegation_manager/25");
        DELEGATION_MANAGER
            .save(deps.as_mut().storage, &delegation_manager_addr)
            .unwrap();

        let result = auth::get_delegation_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, delegation_manager_addr);
    }

    #[test]
    fn test_get_slash_manager() {
        let mut deps = mock_dependencies();

        let slash_manager_addr = deps.api.addr_make("slash_manager/5");
        SLASH_MANAGER
            .save(deps.as_mut().storage, &slash_manager_addr)
            .unwrap();

        let result = auth::get_slash_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, slash_manager_addr);
    }
}
