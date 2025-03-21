#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{self, ExecuteMsg, InstantiateMsg},
    state::{REGISTRY, ROUTER, SLASHERS},
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

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SlashRequest {
            accused,
            start,
            end,
            vault,
            share_amount,
        } => {
            let accused = deps.api.addr_validate(&accused)?;
            let vault = deps.api.addr_validate(&vault)?;
            execute::slash_request(deps, env, info, vault, share_amount, start, end, accused)
        }
        ExecuteMsg::SlashVote {} => {
            // execute::slash_vote();
            Ok(Response::default())
        }
        ExecuteMsg::SlashExecute {} => {
            // execute::slash_execute();
            Ok(Response::default())
        }
        ExecuteMsg::AddSlasher { slasher } => {
            let slasher = deps.api.addr_validate(&slasher)?;
            execute::add_slasher(deps, info, slasher)
        }
    }
}

mod execute {
    use cosmwasm_std::Addr;

    use crate::auth;

    use super::*;

    /// request a slash by the middleware event watchers
    pub fn slash_request(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        vault: Addr,
        amount: u64,
        start: u64,
        end: u64,
        accused: Addr,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;
        todo!();
    }

    /// is this even needed? here?
    /// Shouldn't whether to slash or not be decided by the downstream slashing strategies?
    pub fn slash_vote() -> Result<Response, ContractError> {
        todo!();
    }

    /// not sure who will call this yet.
    /// this will execute the slash request
    /// Mostly like execute msg to the underlying vault
    /// cw20 or bank vault atm
    /// to burn the shares.
    pub fn slash_execute(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        auth::assert_router(deps.as_ref(), &info)?;
        todo!();
    }

    pub fn add_slasher(
        deps: DepsMut,
        info: MessageInfo,
        slasher: Addr,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        SLASHERS.save(deps.storage, &slasher, &info.sender)?;

        Ok(Response::default())
    }
}
