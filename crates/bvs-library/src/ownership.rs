use cosmwasm_std::{
    Addr, Deps, DepsMut, Event, MessageInfo, Response, StdError, StdResult, Storage,
};
use cw_storage_plus::Item;

const OWNER: Item<Addr> = Item::new("_owner");

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OwnershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

/// Set the owner of the contract (this is internal, no checks are done)
pub fn set_owner(storage: &mut dyn Storage, owner: &Addr) -> Result<(), OwnershipError> {
    OWNER.save(storage, owner)?;
    Ok(())
}

/// Get the owner of the contract
/// If [set_owner] has not been called, it will return an [StdError::NotFound]
pub fn get_owner(storage: &dyn Storage) -> StdResult<Addr> {
    OWNER.may_load(storage)?.ok_or(StdError::not_found("owner"))
}

/// Transfer ownership of the contract to a new owner.
/// Contract admin (set for all BVS contracts, a cosmwasm feature)
/// has the omni-ability to override by migration;
/// this logic is app-level.
/// > 2-step ownership transfer is mostly redundant for CosmWasm contracts with the admin set.
/// > You can override ownership with using CosmWasm migrate `entry_point`.
pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, OwnershipError> {
    assert_owner(deps.as_ref(), &info)?;

    let old_owner = OWNER.load(deps.storage)?;
    OWNER.save(deps.storage, &new_owner)?;
    Ok(Response::new().add_event(
        Event::new("TransferredOwnership")
            .add_attribute("old_owner", old_owner.as_str())
            .add_attribute("new_owner", new_owner.as_str()),
    ))
}

/// Asserts that the sender of the message is the owner of the contract
pub fn assert_owner(deps: Deps, info: &MessageInfo) -> Result<(), OwnershipError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(OwnershipError::Unauthorized);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ownership::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies};
    use cosmwasm_std::Addr;

    #[test]
    fn test_assert_owner() {
        let mut deps = mock_dependencies();

        let owner_addr = deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let owner_info = message_info(&owner_addr, &[]);

        let result = assert_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = assert_owner(deps.as_ref(), &info);
        assert_eq!(result, Err(OwnershipError::Unauthorized));
    }

    #[test]
    fn test_assert_owner_fail() {
        let mut deps = mock_dependencies();

        let owner_addr = deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let owner_info = message_info(&owner_addr, &[]);

        let result = assert_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = assert_owner(deps.as_ref(), &info);
        assert_eq!(result, Err(OwnershipError::Unauthorized));
    }

    #[test]
    fn test_transfer_ownership() {
        let mut deps = mock_dependencies();

        let owner_addr = deps.api.addr_make("owner");
        OWNER.save(deps.as_mut().storage, &owner_addr).unwrap();

        let new_owner_addr = deps.api.addr_make("new_owner");

        let owner_info = message_info(&owner_addr, &[]);
        transfer_ownership(deps.as_mut(), owner_info.clone(), new_owner_addr.clone()).unwrap();

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_owner_addr);

        let res = transfer_ownership(deps.as_mut(), owner_info.clone(), owner_addr.clone());
        assert_eq!(res, Err(OwnershipError::Unauthorized));

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_owner_addr);

        let new_new_owner_addr = deps.api.addr_make("new_new_owner");
        let new_owner_info = message_info(&new_owner_addr, &[]);
        transfer_ownership(
            deps.as_mut(),
            new_owner_info.clone(),
            new_new_owner_addr.clone(),
        )
        .unwrap();

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_new_owner_addr);
    }
}
