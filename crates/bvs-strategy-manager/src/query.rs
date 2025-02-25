use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::state::StrategyManagerState;

#[cw_serde]
pub struct DepositsResponse {
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct StakerStrategyListLengthResponse {
    pub strategies_len: Uint128,
}

#[cw_serde]
pub struct StakerStrategySharesResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct StakerStrategyListResponse {
    pub strategies: Vec<Addr>,
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner_addr: Addr,
}

#[cw_serde]
pub struct StrategyWhitelistedResponse {
    pub is_whitelisted: bool,
}

#[cw_serde]
pub struct StrategyWhitelisterResponse {
    pub whitelister: Addr,
}

#[cw_serde]
pub struct StrategyManagerStateResponse {
    pub state: StrategyManagerState,
}

#[cw_serde]
pub struct DelegationManagerResponse {
    pub delegation_manager: Addr,
}

#[cw_serde]
pub struct TokenStrategyResponse {
    pub strategy: Addr,
}

#[cw_serde]
pub struct IsTokenBlacklistedResponse {
    pub token: Addr,
    pub is_blacklisted: bool,
}
