use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct SharesToUnderlyingResponse {
    pub amount_to_send: Uint128,
}

#[cw_serde]
pub struct UnderlyingToShareResponse {
    pub share_to_send: Uint128,
}

#[cw_serde]
pub struct UserUnderlyingResponse {
    pub amount_to_send: Uint128,
}

#[cw_serde]
pub struct StrategyManagerResponse {
    pub strate_manager_addr: Addr,
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

#[cw_serde]
pub struct UnderlyingToSharesResponse {
    pub share_to_send: Uint128,
}
