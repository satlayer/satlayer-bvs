use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response, Storage};
use cw_storage_plus::Item;

pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");

/// Contract Control Plane, it defines how the contract messages get routed.
/// While instantiate creates the contract: gives the contract an address.
/// This sets up the contract for routing and access control management.
/// It can be called more than once to set new values but only by the owner.
pub fn set_routing(
    deps: DepsMut,
    info: MessageInfo,
    delegation_manager: Addr,
    strategy_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.as_ref(), &info)?;

    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetRouting")
            .add_attribute("delegation_manager", delegation_manager)
            .add_attribute("strategy_manager", strategy_manager),
    ))
}

/// Get the Delegation Manager address
/// If SetRouting has not been called, it will return an Unauthorized error
pub fn get_delegation_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    DELEGATION_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

/// Get the Strategy Manager address
/// If SetRouting has not been called, it will return an Unauthorized error
pub fn get_strategy_manager(storage: &dyn Storage) -> Result<Addr, ContractError> {
    STRATEGY_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})
}

#[cfg(test)]
mod tests {
    use crate::auth::{set_routing, DELEGATION_MANAGER, STRATEGY_MANAGER};
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

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");
        let new_strategy_manager = deps.api.addr_make("new_strategy_manager");

        let res = set_routing(
            deps.as_mut(),
            owner_info,
            new_delegation_manager.clone(),
            new_strategy_manager.clone(),
        )
        .unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRouting")
                    .add_attribute("delegation_manager", new_delegation_manager.as_str())
                    .add_attribute("strategy_manager", new_strategy_manager.as_str())
            )
        );
    }

    #[test]
    fn test_set_routing_not_authorized() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");
        let new_strategy_manager = deps.api.addr_make("new_strategy_manager");

        let sender = &deps.api.addr_make("random_sender");
        let sender_info = message_info(sender, &[]);

        let err = set_routing(
            deps.as_mut(),
            sender_info,
            new_delegation_manager.clone(),
            new_strategy_manager.clone(),
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

        let delegation_manager_addr = deps.api.addr_make("delegation_manager_addr-2");
        DELEGATION_MANAGER
            .save(deps.as_mut().storage, &delegation_manager_addr)
            .unwrap();

        let result = auth::get_delegation_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, delegation_manager_addr);
    }

    #[test]
    fn test_get_strategy_manager() {
        let mut deps = mock_dependencies();

        let strategy_manager_addr = deps.api.addr_make("strategy_manager_addr-3");
        STRATEGY_MANAGER
            .save(deps.as_mut().storage, &strategy_manager_addr)
            .unwrap();

        let result = auth::get_strategy_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, strategy_manager_addr);
    }
}
