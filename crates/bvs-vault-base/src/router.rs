use crate::error::VaultError;
use bvs_vault_router::msg::QueryMsg;
use cosmwasm_std::{Addr, Deps, Env, MessageInfo, StdError, StdResult, Storage};
use cw_storage_plus::Item;

const ROUTER: Item<Addr> = Item::new("router");
const OPERATOR: Item<Addr> = Item::new("operator");

/// Set the `vault-router` address, called once during `initialization`.
/// The `router` is the address where the vault calls
/// or receives calls from to perform slashing or delegation operations.
pub fn set_router(storage: &mut dyn Storage, router: &Addr) -> StdResult<()> {
    ROUTER.save(storage, router)?;
    Ok(())
}

/// Set the `operator` address, called once during `initialization`.
/// The `operator` is the address where the vault is delegated to.
/// The `operator` cannot withdraw assets from the vault.
pub fn set_operator(storage: &mut dyn Storage, operator: &Addr) -> StdResult<()> {
    OPERATOR.save(storage, operator)?;
    Ok(())
}

/// Get the `vault-router` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub fn get_router(storage: &dyn Storage) -> StdResult<Addr> {
    ROUTER
        .may_load(storage)?
        .ok_or(StdError::not_found("router"))
}

/// Get the `operator` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub fn get_operator(storage: &dyn Storage) -> StdResult<Addr> {
    OPERATOR
        .may_load(storage)?
        .ok_or(StdError::not_found("operator"))
}

/// Asserts that the sender is the `vault-router`
/// For delegation/slashing operations.
pub fn assert_router(storage: &dyn Storage, info: &MessageInfo) -> Result<(), VaultError> {
    let router = ROUTER
        .may_load(storage)?
        .ok_or(VaultError::unauthorized("no router set"))?;
    if info.sender != router {
        return Err(VaultError::unauthorized("not router"));
    }
    Ok(())
}

/// Asserts that the current vault is whitelisted in the `vault-router`
pub fn assert_whitelisted(deps: &Deps, env: &Env) -> Result<(), VaultError> {
    let router = get_router(deps.storage)?;
    let vault = &env.contract.address;
    let is_whitelisted: bool = deps.querier.query_wasm_smart(
        router.to_string(),
        &QueryMsg::IsWhitelisted {
            vault: vault.to_string(),
        },
    )?;
    if !is_whitelisted {
        return Err(VaultError::NotWhitelisted {});
    }
    Ok(())
}

/// Asserts that the vault is not delegated to any services.
/// Used to prevent unauthorized withdrawals, delegated vaults withdrawal must be queued.
pub fn assert_not_validating(deps: &Deps) -> Result<(), VaultError> {
    let router = get_router(deps.storage)?;
    let operator = get_operator(deps.storage)?;

    let is_delegated: bool = deps.querier.query_wasm_smart(
        router.to_string(),
        &QueryMsg::IsValidating {
            operator: operator.to_string(),
        },
    )?;
    if is_delegated {
        return Err(VaultError::Delegated {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::router::{assert_not_validating, assert_whitelisted, set_operator, OPERATOR};
    use crate::{
        error::VaultError,
        router,
        router::{assert_router, set_router, ROUTER},
    };
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{to_json_binary, ContractResult, SystemError, SystemResult, WasmQuery};

    #[test]
    fn test_set_router() {
        let mut deps = mock_dependencies();
        let router = deps.api.addr_make("router/1");
        set_router(deps.as_mut().storage, &router).unwrap();
    }

    #[test]
    fn test_set_operator() {
        let mut deps = mock_dependencies();
        let operator = deps.api.addr_make("operator/1");
        set_operator(deps.as_mut().storage, &operator).unwrap();
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
    fn test_get_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator/2");
        set_operator(deps.as_mut().storage, &operator).unwrap();

        let result = router::get_operator(deps.as_mut().storage).unwrap();

        assert_eq!(result, operator);
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
            VaultError::Unauthorized {
                msg: "not router".to_string()
            }
            .to_string()
        );
    }

    #[test]
    fn test_assert_whitelisted() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let router = deps.api.addr_make("router/5");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        {
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { .. } => {
                    return SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()));
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let sender = deps.api.addr_make("sender");
        assert_whitelisted(&deps.as_ref(), &env).unwrap();
    }

    #[test]
    fn test_assert_whitelisted_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let router = deps.api.addr_make("router/5");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        {
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { .. } => {
                    return SystemResult::Ok(ContractResult::Ok(to_json_binary(&false).unwrap()));
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let sender = deps.api.addr_make("sender");
        let err = assert_whitelisted(&deps.as_ref(), &env).unwrap_err();
        assert_eq!(err.to_string(), VaultError::NotWhitelisted {}.to_string());
    }

    #[test]
    fn test_assert_not_validating() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator/5");
        OPERATOR.save(deps.as_mut().storage, &operator).unwrap();

        let router = deps.api.addr_make("router/5");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        {
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { .. } => {
                    return SystemResult::Ok(ContractResult::Ok(to_json_binary(&false).unwrap()));
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let sender = deps.api.addr_make("sender");
        assert_not_validating(&deps.as_ref()).unwrap();
    }

    #[test]
    fn test_assert_not_validating_fail() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator/5");
        OPERATOR.save(deps.as_mut().storage, &operator).unwrap();

        let router = deps.api.addr_make("router/5");
        ROUTER.save(deps.as_mut().storage, &router).unwrap();

        {
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { .. } => {
                    return SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()));
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let sender = deps.api.addr_make("sender");
        let err = assert_not_validating(&deps.as_ref()).unwrap_err();
        assert_eq!(err.to_string(), VaultError::Delegated {}.to_string());
    }
}
