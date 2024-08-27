use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    RemoveShares {
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: Addr,
        strategy: Addr,
        shares: Uint128,
        token: Addr,
    },
    AddShares {
        staker: Addr,
        token: Addr,
        strategy: Addr,
        shares: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetDeposits { staker: Addr },
    GetStakerStrategyShares { staker: Addr, strategy: Addr },
    IsThirdPartyTransfersForbidden { strategy: Addr },
    IsStrategyWhitelisted { strategy: String },
}

#[cw_serde]
pub struct DepositsResponse {
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct ThirdPartyTransfersForbiddenResponse {
    pub is_forbidden: bool,
}

#[cw_serde]
pub struct StakerStrategySharesResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct StrategyWhitelistedResponse {
    pub is_whitelisted: bool,
}