pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "library")]
pub mod api {
    use crate::msg::{CanExecuteResponse, QueryMsg};
    use cosmwasm_std::{Addr, Deps, Env, MessageInfo, StdError, StdResult, Storage};
    use cw_storage_plus::Item;

    pub const REGISTRY: Item<Addr> = Item::new("_registry");

    pub use strum::Display;

    pub fn set_registry(store: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
        REGISTRY.save(store, addr)
    }

    pub fn validate_can_execute(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        msg: &dyn ToString,
    ) -> StdResult<()> {
        let addr = REGISTRY.load(deps.storage)?;
        let method = msg.to_string();

        let query = QueryMsg::CanExecute {
            contract: env.contract.address.to_string(),
            sender: info.sender.to_string(),
            method,
        };
        let response: CanExecuteResponse = deps.querier.query_wasm_smart(addr, &query)?;
        if response.can_execute() {
            return Ok(());
        }
        Err(StdError::generic_err("CanExecute: false"))
    }
}
