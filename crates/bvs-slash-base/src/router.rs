use crate::error::SlasherError;
use cosmwasm_std::{Addr, MessageInfo, StdError, StdResult, Storage};
use cw_storage_plus::Item;

const ROUTER: Item<Addr> = Item::new("router");
const VAULT: Item<Addr> = Item::new("vault");

/// Set the `vault-router` address, called once during `initialization`.
/// The `router` is the address where the vault calls
/// or receives calls from to perform slashing or delegation operations.
pub fn set_router(storage: &mut dyn Storage, router: &Addr) -> StdResult<()> {
    ROUTER.save(storage, router)?;
    Ok(())
}

/// Get the `vault-router` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub fn get_router(storage: &dyn Storage) -> StdResult<Addr> {
    ROUTER
        .may_load(storage)?
        .ok_or(StdError::not_found("router"))
}

/// Asserts that the sender is the `vault-router`
/// For delegation/slashing operations.
pub fn assert_router(storage: &dyn Storage, info: &MessageInfo) -> Result<(), SlasherError> {
    let router = ROUTER
        .may_load(storage)?
        .ok_or(SlasherError::unauthorized("no router set"))?;
    if info.sender != router {
        return Err(SlasherError::unauthorized("not router"));
    }
    Ok(())
}

/// Set the `vault` address, called once during `initialization`.
pub fn set_vault(storage: &mut dyn Storage, vault: &Addr) -> StdResult<()> {
    VAULT.save(storage, vault)?;
    Ok(())
}

/// Get the `vault` address
pub fn get_vault(storage: &dyn Storage) -> StdResult<Addr> {
    VAULT.may_load(storage)?.ok_or(StdError::not_found("vault"))
}

/// Asserts that the sender is the `vault`
pub fn assert_vault(storage: &dyn Storage, info: &MessageInfo) -> Result<(), SlasherError> {
    let vault = VAULT
        .may_load(storage)?
        .ok_or(SlasherError::unauthorized("no vault set"))?;
    if info.sender != vault {
        return Err(SlasherError::unauthorized("not vault"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        error::SlasherError,
        router,
        router::{assert_router, set_router, ROUTER},
    };
    use cosmwasm_std::testing::{message_info, mock_dependencies};

    #[test]
    fn test_set_router() {
        let mut deps = mock_dependencies();
        let router = deps.api.addr_make("router/1");
        set_router(deps.as_mut().storage, &router).unwrap();
    }

    #[test]
    fn test_get_router() {
        let mut deps = mock_dependencies();

        let router = deps.api.addr_make("router/2");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        let result = router::get_router(deps.as_mut().storage).unwrap();

        assert_eq!(result, router);
    }

    #[test]
    fn test_assert_router() {
        let mut deps = mock_dependencies();

        let router = deps.api.addr_make("router/3");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        let info = message_info(&router, &[]);
        assert_router(&deps.storage, &info).unwrap();
    }

    #[test]
    fn test_assert_router_fail() {
        let mut deps = mock_dependencies();

        let router = deps.api.addr_make("router/4");
        ROUTER.save(&mut deps.storage, &router).unwrap();

        let not_router = deps.api.addr_make("router/not");
        let sender_info = message_info(&not_router, &[]);
        let error = assert_router(&deps.storage, &sender_info).unwrap_err();

        assert_eq!(
            error.to_string(),
            SlasherError::Unauthorized {
                msg: "not router".to_string()
            }
            .to_string()
        );
    }
}
