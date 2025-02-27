pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "library")]
pub mod api {
    use crate::msg::{CanExecuteFlag, CanExecuteResponse, QueryMsg};
    use cosmwasm_std::{Addr, Deps, Env, MessageInfo, StdError, StdResult, Storage};
    use cw_storage_plus::Item;

    pub use strum::Display;

    /// Errors associated with the BVS registry.
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
                CanExecuteFlag::CanExecute => Ok(()),
                CanExecuteFlag::IsPaused => Err(RegistryError::IsPaused),
                CanExecuteFlag::Unauthorized => Err(RegistryError::Unauthorized),
            }
        }
    }

    pub const REGISTRY: Item<Addr> = Item::new("_registry");

    pub fn instantiate_registry(store: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        REGISTRY.save(store, addr)
    }

    pub fn assert_can_execute(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        msg: &dyn ToString,
    ) -> Result<(), RegistryError> {
        let addr = REGISTRY.load(deps.storage)?;
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
