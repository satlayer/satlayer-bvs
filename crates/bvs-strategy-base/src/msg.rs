use crate::query::{
    SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse, TotalSharesResponse,
    UnderlyingToShareResponse, UnderlyingToSharesResponse, UnderlyingTokenResponse,
    UserUnderlyingResponse,
};
use crate::state::StrategyState;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub strategy_manager: String,
    pub underlying_token: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        recipient: String,
        token: String,
        amount_shares: Uint128,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares { staker: String, strategy: String },

    #[returns(SharesToUnderlyingResponse)]
    SharesToUnderlyingView { amount_shares: Uint128 },

    #[returns(UnderlyingToShareResponse)]
    UnderlyingToShareView { amount: Uint128 },

    #[returns(UserUnderlyingResponse)]
    UserUnderlyingView { user: String },

    #[returns(StrategyManagerResponse)]
    GetStrategyManager {},

    #[returns(UnderlyingTokenResponse)]
    GetUnderlyingToken {},

    #[returns(TotalSharesResponse)]
    GetTotalShares {},

    #[returns(UnderlyingToSharesResponse)]
    UnderlyingToShares { amount_underlying: Uint128 },

    #[returns(StrategyState)]
    GetStrategyState {},
}
