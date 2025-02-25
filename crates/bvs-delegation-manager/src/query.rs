use crate::msg::OperatorDetails;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};

#[cw_serde]
pub struct DelegatedResponse {
    pub is_delegated: bool,
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}

#[cw_serde]
pub struct OperatorDetailsResponse {
    pub details: OperatorDetails,
}

#[cw_serde]
pub struct StakerOptOutWindowBlocksResponse {
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct OperatorSharesResponse {
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct DelegatableSharesResponse {
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct WithdrawalDelayResponse {
    pub withdrawal_delays: Vec<u64>,
}

#[cw_serde]
pub struct CalculateWithdrawalRootResponse {
    pub withdrawal_root: Binary,
}

#[cw_serde]
pub struct OperatorStakersResponse {
    pub stakers_and_shares: Vec<StakerShares>,
}

#[cw_serde]
pub struct CumulativeWithdrawalsQueuedResponse {
    pub cumulative_withdrawals: Uint128,
}

#[cw_serde]
pub struct StakerShares {
    pub staker: Addr,
    pub shares_per_strategy: Vec<(Addr, Uint128)>,
}
