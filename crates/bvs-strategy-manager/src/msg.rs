use crate::query::{
    DepositsResponse, StakerStrategyListLengthResponse, StakerStrategyListResponse,
    StakerStrategySharesResponse, StrategyWhitelistedResponse, TokenStrategyResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    AddNewStrategy {
        new_strategy: String,
        token: String,
    },
    AddStrategiesToWhitelist {
        strategies: Vec<String>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
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
    },
    AddShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    RemoveShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
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

    #[returns(TokenStrategyResponse)]
    TokenStrategy { token: String },
}

/// Both Strategy Manager and Delegation Manager are circularly dependent on each other.
/// Since we can't circularly import each other, we put ExecuteMsg,
/// which is only used by the Strategy Manager here.
/// Delegation Manager must import this module and implement [IncreaseDelegatedShares]
pub mod delegation_manager {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::Uint128;

    #[cw_serde]
    pub enum ExecuteMsg {
        IncreaseDelegatedShares(IncreaseDelegatedShares),
    }

    #[cw_serde]
    pub struct IncreaseDelegatedShares {
        pub staker: String,
        pub strategy: String,
        pub shares: Uint128,
    }
}
