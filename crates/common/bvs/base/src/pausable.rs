#[allow(deprecated)]
use crate::roles::{check_pauser, check_unpauser};
use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdError, StdResult};
use cw_storage_plus::Item;

pub const PAUSED: u8 = 1;
pub const UNPAUSED: u8 = 0;

pub const PAUSED_STATE: Item<u8> = Item::new("paused_state");

pub fn pause(deps: DepsMut, info: &MessageInfo) -> StdResult<Response> {
    check_pauser(deps.as_ref(), info.clone())?;

    PAUSED_STATE.save(deps.storage, &PAUSED)?;

    Ok(Response::new().add_attribute("action", "PAUSED"))
}

pub fn unpause(deps: DepsMut, info: &MessageInfo) -> StdResult<Response> {
    check_unpauser(deps.as_ref(), info.clone())?;

    PAUSED_STATE.save(deps.storage, &UNPAUSED)?;

    Ok(Response::new().add_attribute("action", "UNPAUSED"))
}

pub fn is_paused(deps: Deps) -> StdResult<bool> {
    let state = PAUSED_STATE.load(deps.storage)?;
    Ok(state == PAUSED)
}

pub fn only_when_not_paused(deps: Deps, index: u8) -> StdResult<()> {
    let paused_state = PAUSED_STATE.load(deps.storage)?;

    if paused_state & (1 << index) != 0 {
        return Err(StdError::generic_err("Paused: Functionality paused"));
    }

    Ok(())
}
