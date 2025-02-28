use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response, Storage};
use cw_storage_plus::Item;

const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");

/// Contract Control Plane, it defines how the contract messages get routed.
/// While instantiate creates the contract: gives the contract an address.
/// This sets up the contract for routing and access control management.
/// It can be called more than once to set new values but only by the owner.
pub fn set_routing(
    deps: DepsMut,
    info: MessageInfo,
    delegation_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.as_ref(), &info)?;

    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetRouting").add_attribute("delegation_manager", delegation_manager.as_str()),
    ))
}

/// Get the Delegation Manager address
/// If SetRouting has not been called, it will return an Unauthorized error
pub fn get_delegation_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    DELEGATION_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

#[cfg(test)]
mod tests {
    use crate::auth::{set_routing, DELEGATION_MANAGER};
    use crate::{auth, ContractError};
    use bvs_library::ownership::{OwnershipError, OWNER};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::{Event, Response};

    #[test]
    fn test_set_routing() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let owner_info = message_info(owner_addr, &[]);

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");

        let res = set_routing(deps.as_mut(), owner_info, new_delegation_manager.clone()).unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRouting")
                    .add_attribute("delegation_manager", new_delegation_manager.as_str())
            )
        );
    }

    #[test]
    fn test_set_routing_not_authorized() {
        let mut deps = mock_dependencies();

        {
            let owner_addr = &deps.api.addr_make("owner");
            OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();
        }

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");

        let sender = &deps.api.addr_make("random_sender");
        let sender_info = message_info(sender, &[]);

        let err =
            set_routing(deps.as_mut(), sender_info, new_delegation_manager.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            ContractError::Ownership(OwnershipError::Unauthorized).to_string()
        );
    }

    #[test]
    fn test_get_delegation_manager() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let delegation_manager = deps.api.addr_make("some_delegation_manager");
        DELEGATION_MANAGER
            .save(deps.as_mut().storage, &delegation_manager)
            .unwrap();
        let addr = auth::get_delegation_manager(&deps.storage).unwrap();
        let loaded_addr = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(loaded_addr, delegation_manager);
        assert_eq!(addr, loaded_addr);
    }

    #[test]
    fn test_get_delegation_manager_unauthorized() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let err = auth::get_delegation_manager(&deps.storage).unwrap_err();

        assert_eq!(err.to_string(), ContractError::Unauthorized {}.to_string());
    }
}
