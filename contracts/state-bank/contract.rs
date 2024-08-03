use crate::{error::ContractError, msg::ExecuteMsg, msg::InstantiateMsg, msg::QueryMsg};

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

