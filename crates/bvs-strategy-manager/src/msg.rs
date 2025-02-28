use crate::query::{
    DelegationManagerResponse, DepositsResponse, IsTokenBlacklistedResponse,
    StakerStrategyListLengthResponse, StakerStrategyListResponse, StakerStrategySharesResponse,
    StrategyManagerStateResponse, StrategyWhitelistedResponse, StrategyWhitelisterResponse,
    TokenStrategyResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub delegation_manager: String,
    pub slash_manager: String,
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
    SetDelegationManager {
        new_delegation_manager: String,
    },
    SetSlashManager {
        new_slash_manager: String,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
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

    #[returns(StrategyManagerStateResponse)]
    GetStrategyManagerState {},

    #[returns(DelegationManagerResponse)]
    DelegationManager {},

    #[returns(IsTokenBlacklistedResponse)]
    IsTokenBlacklisted { token: String },

    #[returns(TokenStrategyResponse)]
    TokenStrategy { token: String },
}
