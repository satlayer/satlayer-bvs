use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    RemoveShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: String,
        strategy: String,
        shares: Uint128,
        token: String,
    },
    AddShares {
        staker: String,
        token: String,
        strategy: String,
        shares: Uint128,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
    },
    SetThirdPartyTransfersForbidden {
        strategy: String,
        value: bool,
    },
    AddStrategiesToWhitelist {
        strategies: Vec<String>,
        third_party_transfers_forbidden_values: Vec<bool>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetDeposits { staker: String },
    GetStakerStrategyShares { staker: String, strategy: String },
    IsThirdPartyTransfersForbidden { strategy: String },
    IsStrategyWhitelisted { strategy: String },
    GetStakerStrategyList { staker: String },
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

#[cw_serde]
pub struct StakerStrategyListResponse {
    pub strategies: Vec<Addr>,
}
