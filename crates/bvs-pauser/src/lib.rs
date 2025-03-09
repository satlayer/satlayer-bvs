pub mod contract;
pub mod msg;
pub mod testing;

mod error;
mod state;

pub use crate::error::PauserError;

#[cfg(feature = "library")]
pub mod api {
    use crate::msg::{CanExecuteFlag, CanExecuteResponse, QueryMsg};
    use cosmwasm_std::{Addr, Deps, Env, MessageInfo, StdError, StdResult, Storage};
    use cw_storage_plus::Item;

    pub use strum::Display;

    /// Errors associated with the BVS Pauser.
    #[derive(thiserror::Error, Debug, PartialEq)]
    pub enum PauserError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("The contract is paused")]
        IsPaused,

        #[error("Not authorized to execute the method")]
        Unauthorized,
    }

    impl From<CanExecuteResponse> for Result<(), PauserError> {
        fn from(value: CanExecuteResponse) -> Self {
            let status: CanExecuteFlag = value.into();
            match status {
                CanExecuteFlag::CanExecute => Ok(()),
                CanExecuteFlag::Paused => Err(PauserError::IsPaused),
                CanExecuteFlag::Unauthorized => Err(PauserError::Unauthorized),
            }
        }
    }

    const PAUSER: Item<Addr> = Item::new("_pauser");

    /// Set the address of the pauser contract in the storage slot `_pauser`.
    /// [`assert_can_execute`] will query the pauser contract at this address.
    pub fn set_pauser(store: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        PAUSER.save(store, addr)
    }

    /// Get the address of the pauser contract from the storage slot `_pauser`.
    /// If [`set_pauser`] has not been called, it will return an [StdError::NotFound].
    pub fn get_pauser(store: &dyn Storage) -> StdResult<Addr> {
        PAUSER.may_load(store)?.ok_or(StdError::not_found("pauser"))
    }

    /// Assert that the `ExecuteMsg` can be executed without restrictions.
    /// Requires [`set_pauser`] to be set in the `instantiate()` message.
    pub fn assert_can_execute(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        msg: &dyn ToString,
    ) -> Result<(), PauserError> {
        let addr = PAUSER.load(deps.storage)?;
        let method = msg.to_string();

        let query_msg = QueryMsg::CanExecute {
            contract: env.contract.address.to_string(),
            sender: info.sender.to_string(),
            method,
        };
        let response: CanExecuteResponse = deps.querier.query_wasm_smart(addr, &query_msg)?;
        response.into()
    }
}
