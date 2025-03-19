#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::InstantiateMsg,
    state::{REGISTRY, ROUTER, SLASHER},
};
use bvs_library::ownership;
use bvs_pauser;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let registry = deps.api.addr_validate(&msg.registry)?;
    REGISTRY.save(deps.storage, &registry)?;

    let router = deps.api.addr_validate(&msg.router)?;
    ROUTER.save(deps.storage, &router)?;

    let slasher = deps.api.addr_validate(&msg.slasher)?;
    SLASHER.save(deps.storage, &slasher)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: (),
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

mod execute {

    /// request a slash by the middleware event watchers
    pub fn slash_request() {
        todo!();
    }

    /// enter a voting period for the slash request
    /// Not clear who will vote,
    /// vote will closed when the peirod is over
    pub fn slash_vote() {
        todo!();
    }

    /// not sure who will call this yet.
    /// this will execute the slash request
    /// Mostly like execute msg to the underlying vault
    /// cw20 or bank vault atm
    /// to burn the shares.
    pub fn slash_execute() {
        todo!();
    }
}
