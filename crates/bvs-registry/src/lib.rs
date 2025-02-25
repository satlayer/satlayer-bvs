pub mod contract;
pub mod msg;
pub mod state;

mod error;

pub use crate::error::ContractError;

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

/// This is an integration testing module to allow easy testing of the contract in cw_multi_test
#[cfg(feature = "testing")]
pub mod testing {
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        msg: Option<crate::msg::InstantiateMsg>,
    ) -> (Addr, crate::msg::InstantiateMsg) {
        let msg = msg.unwrap_or(crate::msg::InstantiateMsg {
            owner: app.api().addr_make("registry:owner").to_string(),
            initial_paused: false,
        });

        let addr = app
            .instantiate_contract(
                code_id,
                app.api().addr_make("sender"),
                &msg,
                &[],
                "BVS Registry",
                None,
            )
            .unwrap();

        (addr, msg)
    }
}
