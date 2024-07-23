pub mod contract;
mod error;
pub mod msg;
mod state;
mod utils;

pub use crate::error::ContractError;

#[cfg(not(feature = "library"))]
mod entry_points {
    use super::*;
    use cosmwasm_std::entry_point;
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};

    #[entry_point]
    pub fn instantiate(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<cosmwasm_std::Response, ContractError> {
        contract::instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<cosmwasm_std::Response, ContractError> {
        contract::execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(
        deps: cosmwasm_std::Deps,
        env: cosmwasm_std::Env,
        msg: QueryMsg,
    ) -> Result<cosmwasm_std::Binary, cosmwasm_std::StdError> {
        contract::query(deps, env, msg)
    }
}
