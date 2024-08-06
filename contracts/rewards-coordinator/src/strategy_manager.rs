use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64, Binary};

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: Addr,
    pub slasher: Addr,
    pub initial_strategy_whitelister: Addr,
    pub initial_owner: Addr,
}

