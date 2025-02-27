use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo, StdError};
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("_owner");

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OwnershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,
}

/// Set the owner of the contract (this is internal, no checks are done)
pub fn _set_owner(deps: DepsMut, owner: &Addr) -> Result<(), OwnershipError> {
    OWNER.save(deps.storage, owner)?;
    Ok(())
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: &MessageInfo,
    new_owner: &Addr,
) -> Result<(), OwnershipError> {
    assert_owner(deps.as_ref(), info)?;

    OWNER.save(deps.storage, new_owner)?;

    Ok(())
}

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
        transfer_ownership(deps.as_mut(), &owner_info, &new_owner_addr).unwrap();

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_owner_addr);

        let res = transfer_ownership(deps.as_mut(), &owner_info, &owner_addr);
        assert_eq!(res, Err(OwnershipError::Unauthorized));

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_owner_addr);

        let new_new_owner_addr = deps.api.addr_make("new_new_owner");
        let new_owner_info = message_info(&new_owner_addr, &[]);
        transfer_ownership(deps.as_mut(), &new_owner_info, &new_new_owner_addr).unwrap();

        let saved_owner = OWNER.load(deps.as_ref().storage).unwrap();
        assert_eq!(saved_owner, new_new_owner_addr);
    }
}
