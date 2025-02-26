pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "library")]
pub mod api {
    use crate::msg::{IsPausedResponse, QueryMsg};
    use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage};
    use cw_storage_plus::Item;

    pub const REGISTRY: Item<Addr> = Item::new("_registry");

    pub use strum::Display;

    pub fn set_registry(store: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        REGISTRY.save(store, addr)
    }

    pub fn is_paused(deps: Deps, env: &Env, msg: &dyn ToString) -> StdResult<()> {
        let addr = REGISTRY.load(deps.storage)?;
        let method = msg.to_string();

        let response: IsPausedResponse = deps.querier.query_wasm_smart(
            addr,
            &QueryMsg::IsPaused {
                contract: env.contract.address.to_string(),
                method,
            },
        )?;

        if response.paused {
            return Err(StdError::generic_err("Paused"));
        }
        Ok(())
    }
}
