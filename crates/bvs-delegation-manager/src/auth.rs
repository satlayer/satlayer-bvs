use crate::ContractError;
use bvs_library::ownership;
use cosmwasm_std::{
    Addr, Deps, DepsMut, Event, MessageInfo, Response, StdError, StdResult, Storage,
};
use cw_storage_plus::Item;

const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
const SLASH_MANAGER: Item<Addr> = Item::new("slash_manager");

/// Contract Control Plane, it defines how the contract messages get routed.
/// While instantiate creates the contract: gives the contract an address.
/// This sets up the contract for routing and access control management.
/// It can be called more than once to set new values but only by the owner.
pub fn set_routing(
    deps: DepsMut,
    info: MessageInfo,
    strategy_manager: Addr,
    slash_manager: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.as_ref(), &info)?;

    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;
    SLASH_MANAGER.save(deps.storage, &slash_manager)?;

    Ok(Response::new().add_event(
        Event::new("SetRouting")
            .add_attribute("strategy_manager", strategy_manager)
            .add_attribute("slash_manager", slash_manager),
    ))
}

/// Get the Strategy Manager address
/// If SetRouting has not been called, it will return an [StdError::NotFound]
pub fn get_strategy_manager(storage: &dyn Storage) -> StdResult<Addr> {
    STRATEGY_MANAGER
        .may_load(storage)?
        .ok_or(StdError::not_found("strategy_manager"))
}

pub fn assert_strategy_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let strategy_manager = STRATEGY_MANAGER
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;
    if info.sender != strategy_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

/// Currently, not used to be migrated and release in the future for Slash Manager
#[allow(dead_code)]
pub fn assert_strategy_or_slash_manager(
    deps: Deps,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let strategy_manager = STRATEGY_MANAGER
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if info.sender == strategy_manager {
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
    use crate::auth::{set_routing, STRATEGY_MANAGER};
    use crate::{auth, ContractError};
    use auth::assert_strategy_manager;
    use bvs_library::ownership::{self, OwnershipError};
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::{Event, Response};

    #[test]
    fn test_set_routing() {
        let mut deps = mock_dependencies();

        let owner_addr = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner_addr).unwrap();

        let owner_info = message_info(owner_addr, &[]);

        let new_strategy = deps.api.addr_make("strategy_manager/15");
        let new_slash = deps.api.addr_make("slash_manager/15");

        let res = set_routing(
            deps.as_mut(),
            owner_info,
            new_strategy.clone(),
            new_slash.clone(),
        )
        .unwrap();

        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("SetRouting")
                    .add_attribute("strategy_manager", new_strategy.as_str())
                    .add_attribute("slash_manager", new_slash.as_str())
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

        let new_strategy = deps.api.addr_make("strategy_manager/55");
        let new_slash = deps.api.addr_make("slash_manager/55");

        let sender = &deps.api.addr_make("random_sender");
        let sender_info = message_info(sender, &[]);

        let err = set_routing(
            deps.as_mut(),
            sender_info,
            new_strategy.clone(),
            new_slash.clone(),
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            ContractError::Ownership(OwnershipError::Unauthorized).to_string()
        );
    }

    #[test]
    fn test_get_strategy_manager() {
        let mut deps = mock_dependencies();

        let strategy_manager_addr = deps.api.addr_make("strategy_manager_addr");
        STRATEGY_MANAGER
            .save(deps.as_mut().storage, &strategy_manager_addr)
            .unwrap();

        let result = auth::get_strategy_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, strategy_manager_addr);
    }

    #[test]
    fn test_assert_strategy_manager() {
        let mut deps = mock_dependencies();

        let manager_addr = deps.api.addr_make("manager_addr");
        STRATEGY_MANAGER
            .save(deps.as_mut().storage, &manager_addr)
            .unwrap();

        let info = message_info(&manager_addr, &[]);
        let result = assert_strategy_manager(deps.as_ref(), &info);

        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_strategy_manager_fail() {
        let mut deps = mock_dependencies();

        let manager_addr = deps.api.addr_make("manager_addr");
        STRATEGY_MANAGER
            .save(&mut deps.storage, &manager_addr)
            .unwrap();

        let not_manager = deps.api.addr_make("not_manager");
        let sender_info = message_info(&not_manager, &[]);
        let result = assert_strategy_manager(deps.as_ref(), &sender_info);

        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }
}
