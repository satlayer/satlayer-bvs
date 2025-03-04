use crate::ContractError;
use cosmwasm_std::{Addr, MessageInfo, StdError, StdResult, Storage};
use cw_storage_plus::Item;

const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");

/// Set the strategy manager
pub fn set_strategy_manager(
    storage: &mut dyn Storage,
    strategy_manager: &Addr,
) -> Result<(), ContractError> {
    STRATEGY_MANAGER.save(storage, strategy_manager)?;
    Ok(())
}

/// Get the Strategy Manager address
/// If [set_strategy_manager] has not been called, it will return an [StdError::NotFound]
pub fn get_strategy_manager(storage: &dyn Storage) -> StdResult<Addr> {
    STRATEGY_MANAGER
        .may_load(storage)?
        .ok_or(StdError::not_found("strategy_manager"))
}

/// Asserts that the sender is the strategy manager
pub fn assert_strategy_manager(
    storage: &dyn Storage,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let strategy_manager = STRATEGY_MANAGER
        .may_load(storage)?
        .ok_or(ContractError::Unauthorized {})?;
    if info.sender != strategy_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::auth::{assert_strategy_manager, set_strategy_manager, STRATEGY_MANAGER};
    use crate::{auth, ContractError};
    use cosmwasm_std::testing::{message_info, mock_dependencies};

    #[test]
    fn test_set_strategy_manager() {
        let mut deps = mock_dependencies();
        let strategy_manager = deps.api.addr_make("strategy_manager/1");
        set_strategy_manager(deps.as_mut().storage, &strategy_manager).unwrap();
    }

    #[test]
    fn test_get_strategy_manager() {
        let mut deps = mock_dependencies();

        let strategy_manager_addr = deps.api.addr_make("strategy_manager/2");
        STRATEGY_MANAGER
            .save(deps.as_mut().storage, &strategy_manager_addr)
            .unwrap();

        let result = auth::get_strategy_manager(deps.as_mut().storage).unwrap();

        assert_eq!(result, strategy_manager_addr);
    }

    #[test]
    fn test_assert_strategy_manager() {
        let mut deps = mock_dependencies();

        let manager_addr = deps.api.addr_make("strategy_manager/3");
        STRATEGY_MANAGER
            .save(deps.as_mut().storage, &manager_addr)
            .unwrap();

        let info = message_info(&manager_addr, &[]);
        assert_strategy_manager(&deps.storage, &info).unwrap();
    }

    #[test]
    fn test_assert_strategy_manager_fail() {
        let mut deps = mock_dependencies();

        let manager_addr = deps.api.addr_make("strategy_manager/4");
        STRATEGY_MANAGER
            .save(&mut deps.storage, &manager_addr)
            .unwrap();

        let not_manager = deps.api.addr_make("strategy_manager/not");
        let sender_info = message_info(&not_manager, &[]);
        let error = assert_strategy_manager(&deps.storage, &sender_info).unwrap_err();

        assert_eq!(
            error.to_string(),
            ContractError::Unauthorized {}.to_string()
        );
    }
}
