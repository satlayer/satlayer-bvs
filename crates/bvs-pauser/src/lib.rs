pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

pub mod testing;

#[cfg(feature = "library")]
pub mod api {
    use crate::msg::{
        CanExecuteResponse, QueryMsg, FLAG_CAN_EXECUTE, FLAG_PAUSED, FLAG_UNAUTHORIZED,
    };
    use cosmwasm_std::{Addr, Deps, Env, MessageInfo, StdError, StdResult, Storage};
    use cw_storage_plus::Item;

    pub use strum::Display;

    /// Errors associated with the BVS Pauser.
    #[derive(thiserror::Error, Debug, PartialEq)]
    pub enum RegistryError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("The contract is paused")]
        IsPaused,

        #[error("Not authorized to execute the method")]
        Unauthorized,
    }

    impl CanExecuteResponse {
        pub fn assert(&self) -> Result<(), RegistryError> {
            match self.0 {
                FLAG_CAN_EXECUTE => Ok(()),
                FLAG_PAUSED => Err(RegistryError::IsPaused),
                FLAG_UNAUTHORIZED => Err(RegistryError::Unauthorized),
                _ => Err(RegistryError::Std(StdError::generic_err(
                    "Unknown flag in CanExecuteResponse",
                ))),
            }
        }
    }

    pub const PAUSER: Item<Addr> = Item::new("_pauser");

    /// Set the address of the pauser contract in the storage slot `_pauser`.
    /// [`assert_can_execute`] will query the pauser contract at this address.
    pub fn set_pauser(store: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        PAUSER.save(store, addr)
    }

    /// Assert that the `ExecuteMsg` can be executed without restrictions.
    /// Requires [`set_pauser`] to be set in the `instantiate()` message.
    pub fn assert_can_execute(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        msg: &dyn ToString,
    ) -> Result<(), RegistryError> {
        let addr = PAUSER.load(deps.storage)?;
        let method = msg.to_string();

        let query_msg = QueryMsg::CanExecute {
            contract: env.contract.address.to_string(),
            sender: info.sender.to_string(),
            method,
        };
        let response: CanExecuteResponse = deps.querier.query_wasm_smart(addr, &query_msg)?;
        response.assert()
    }
}
