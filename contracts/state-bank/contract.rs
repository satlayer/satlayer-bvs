use crate::{error::ContractError, msg::ExecuteMsg, msg::InstantiateMsg, msg::QueryMsg};

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let response = Response::new().add_attribute("method", "instantiate");

    Ok(response)
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Set { key, value } => execute_set(info, key, value),
    }
}

pub fn execute_set(info: MessageInfo, key: String, value: i32) -> StdResult<Response> {
    let event = Event::new("UpdateState")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("key", key.clone())
        .add_attribute("value", value.to_string());

    Ok(Response::new()
        .add_attribute("method", "set")
        .add_attribute("key", key)
        .add_attribute("value", value.to_string())
        .add_event(event))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

