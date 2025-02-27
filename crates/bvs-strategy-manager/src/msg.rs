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
    pub delegation_manager: String,
    pub slash_manager: String,
    pub initial_strategy_whitelister: String,
    pub initial_owner: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
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
    /// Transfer ownership of the contract to a new owner.
    /// Contract admin (set for all BVS contracts, a cosmwasm feature)
    /// has the omni-ability to override by migration;
    /// this logic is app-level.
    /// > 2-step ownership transfer is mostly redundant for CosmWasm contracts with the admin set.
    /// > You can override ownership with using CosmWasm migrate `entry_point`.
    TransferOwnership {
        new_owner: String,
    },
    Pause {},
    Unpause {},
    SetPauser {
        new_pauser: String,
    },
    SetUnpauser {
        new_unpauser: String,
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
