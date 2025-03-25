#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::RewardsError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, RewardsError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, RewardsError> {
    match msg {
        ExecuteMsg::DistributeRewards {
            token,
            amount,
            root,
        } => execute::distribute_rewards(deps, info),
        ExecuteMsg::ClaimRewards {
            token,
            amount,
            proof,
        } => execute::claim_rewards(deps, info),
    }
}

mod execute {
    use crate::error::RewardsError;
    use cosmwasm_std::{DepsMut, MessageInfo, Response};

    pub fn distribute_rewards(
        _deps: DepsMut,
        _info: MessageInfo,
    ) -> Result<Response, RewardsError> {
        todo!()
    }
    pub fn claim_rewards(_deps: DepsMut, _info: MessageInfo) -> Result<Response, RewardsError> {
        todo!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    todo!()
}

mod query {}

#[cfg(test)]
mod tests {}
