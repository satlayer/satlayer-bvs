use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo, Response, StdError, StdResult};
use cw_storage_plus::Item;

pub const PAUSER: Item<Addr> = Item::new("pauser");
pub const UNPAUSER: Item<Addr> = Item::new("unpauser");

pub fn set_pauser(deps: DepsMut, new_pauser: Addr) -> StdResult<Response> {
    PAUSER.save(deps.storage, &new_pauser)?;

    Ok(Response::new()
        .add_attribute("method", "set_pauser")
        .add_attribute("new_pauser", new_pauser.to_string()))
}

pub fn set_unpauser(deps: DepsMut, new_unpauser: Addr) -> StdResult<Response> {
    UNPAUSER.save(deps.storage, &new_unpauser)?;

    Ok(Response::new()
        .add_attribute("method", "set_unpauser")
        .add_attribute("new_unpauser", new_unpauser.to_string()))
}

pub fn check_pauser(deps: Deps, info: MessageInfo) -> StdResult<()> {
    let pauser = PAUSER.load(deps.storage)?;
    if info.sender != pauser {
        return Err(StdError::generic_err("Unauthorized: Not pauser"));
    }
    Ok(())
}

pub fn check_unpauser(deps: Deps, info: MessageInfo) -> StdResult<()> {
    let unpauser = UNPAUSER.load(deps.storage)?;
    if info.sender != unpauser {
        return Err(StdError::generic_err("Unauthorized: Not unpauser"));
    }
    Ok(())
}
