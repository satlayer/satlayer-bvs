use crate::query::{
    ExplanationResponse, SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse,
    TotalSharesResponse, TvlLimitsResponse, UnderlyingToSharesResponse, UnderlyingTokenResponse,
    UserUnderlyingResponse,
};
use crate::state::StrategyState;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub strategy_manager: String,
    pub underlying_token: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
    pub max_per_deposit: Uint128,
    pub max_total_deposits: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        recipient: String,
        amount_shares: Uint128,
    },
    SetStrategyManager {
        new_strategy_manager: String,
    },
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
    SetTvlLimits {
        max_per_deposit: Uint128,
        max_total_deposits: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares { staker: String },

    #[returns(SharesToUnderlyingResponse)]
    SharesToUnderlying { amount_shares: Uint128 },

    #[returns(UnderlyingToSharesResponse)]
    UnderlyingToShares { amount: Uint128 },

    #[returns(UserUnderlyingResponse)]
    UserUnderlying { user: String },

    #[returns(StrategyManagerResponse)]
    GetStrategyManager {},

    #[returns(UnderlyingTokenResponse)]
    GetUnderlyingToken {},

    #[returns(TotalSharesResponse)]
    GetTotalShares {},

    #[returns(ExplanationResponse)]
    Explanation {},

    #[returns(StrategyState)]
    GetStrategyState {},

    #[returns(TvlLimitsResponse)]
    GetTvlLimits {},
}
