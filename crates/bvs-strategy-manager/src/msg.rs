use crate::query::{
    DepositsResponse, IsTokenBlacklistedResponse, StakerStrategyListLengthResponse,
    StakerStrategyListResponse, StakerStrategySharesResponse, StrategyWhitelistedResponse,
    StrategyWhitelisterResponse, TokenStrategyResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub initial_strategy_whitelister: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    AddNewStrategy {
        new_strategy: String,
        token: String,
    },
    BlacklistTokens {
        tokens: Vec<String>,
    },
    AddStrategiesToWhitelist {
        strategies: Vec<String>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
    },
    SetStrategyWhitelister {
        new_strategy_whitelister: String,
    },
    DepositIntoStrategy {
        strategy: String,
        token: String,
        amount: Uint128,
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
    RemoveShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
    SetRouting {
        delegation_manager: String,
        slash_manager: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DepositsResponse)]
    GetDeposits { staker: String },

    #[returns(StakerStrategyListLengthResponse)]
    StakerStrategyListLength { staker: String },

    #[returns(StakerStrategySharesResponse)]
    GetStakerStrategyShares { staker: String, strategy: String },

    #[returns(StakerStrategyListResponse)]
    GetStakerStrategyList { staker: String },

    #[returns(StrategyWhitelistedResponse)]
    IsStrategyWhitelisted { strategy: String },

    #[returns(StrategyWhitelisterResponse)]
    GetStrategyWhitelister {},

    #[returns(IsTokenBlacklistedResponse)]
    IsTokenBlacklisted { token: String },

    #[returns(TokenStrategyResponse)]
    TokenStrategy { token: String },
}
