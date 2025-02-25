use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdError, StdResult};
use cw_storage_plus::Item;

pub const PAUSED_STATE: Item<u8> = Item::new("paused_state");

pub const PAUSE_ALL: u8 = 0xFF;
pub const UNPAUSE_ALL: u8 = 0x00;

use crate::roles::{check_pauser, check_unpauser};

fn set_bit(value: u8, index: u8) -> StdResult<u8> {
    if index >= 8 {
        return Err(StdError::generic_err("Index out of range for bit shifting"));
    }
    let mask = 1u8 << index;
    Ok(value | mask)
}

fn clear_bit(value: u8, index: u8) -> StdResult<u8> {
    if index >= 8 {
        return Err(StdError::generic_err("Index out of range for bit shifting"));
    }
    let mask = 1u8 << index;
    Ok(value & !mask)
}

pub fn pause_all(deps: DepsMut, info: &MessageInfo) -> StdResult<Response> {
    check_pauser(deps.as_ref(), info.clone())?;
    PAUSED_STATE.save(deps.storage, &PAUSE_ALL)?;
    Ok(Response::new().add_attribute("action", "pause_all"))
}

pub fn unpause_all(deps: DepsMut, info: &MessageInfo) -> StdResult<Response> {
    check_unpauser(deps.as_ref(), info.clone())?;
    PAUSED_STATE.save(deps.storage, &UNPAUSE_ALL)?;
    Ok(Response::new().add_attribute("action", "unpause_all"))
}

pub fn pause_bit(deps: DepsMut, info: &MessageInfo, index: u8) -> StdResult<Response> {
    check_pauser(deps.as_ref(), info.clone())?;
    let mut current = PAUSED_STATE.load(deps.storage)?;
    current = set_bit(current, index)?;
    PAUSED_STATE.save(deps.storage, &current)?;
    Ok(Response::new()
        .add_attribute("action", "pause_bit")
        .add_attribute("index", index.to_string()))
}

pub fn unpause_bit(deps: DepsMut, info: &MessageInfo, index: u8) -> StdResult<Response> {
    check_unpauser(deps.as_ref(), info.clone())?;
    let mut current = PAUSED_STATE.load(deps.storage)?;
    current = clear_bit(current, index)?;
    PAUSED_STATE.save(deps.storage, &current)?;
    Ok(Response::new()
        .add_attribute("action", "unpause_bit")
        .add_attribute("index", index.to_string()))
}

pub fn get_paused_state(deps: Deps) -> StdResult<u8> {
    PAUSED_STATE.load(deps.storage)
}

pub fn only_when_not_paused(deps: Deps, index: u8) -> StdResult<()> {
    let state = PAUSED_STATE.load(deps.storage)?;
    if index >= 8 {
        return Err(StdError::generic_err("Index out of range"));
    }
    let mask = 1u8 << index;
    if (state & mask) == mask {
        return Err(StdError::generic_err("Functionality is paused"));
    }
    Ok(())
}

pub fn only_when_not_paused_all(deps: Deps) -> StdResult<()> {
    let state = PAUSED_STATE.load(deps.storage)?;
    if state != 0 {
        return Err(StdError::generic_err("Contract is globally paused"));
    }
    Ok(())
}
