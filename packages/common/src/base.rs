use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub strategy_manager: String,
    pub underlying_token: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        recipient: String,
        token: String,
        amount_shares: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetStrategyState {},
}

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct StrategyState {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
    pub total_shares: Uint128,
}
