use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct StrategyManagerResponse {
    pub strategy_manager_addr: Addr,
}

#[cw_serde]
pub struct UnderlyingTokenResponse {
    pub underlying_token_addr: Addr,
}

#[cw_serde]
pub struct TotalSharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct ExplanationResponse {
    pub explanation: String,
}
