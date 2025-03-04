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
        shares: Uint128,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares { staker: String },

    #[returns(UserUnderlyingResponse)]
    UserUnderlyingView { user: String },

    #[returns(SharesToUnderlyingResponse)]
    SharesToUnderlyingView { amount_shares: Uint128 },

    #[returns(UnderlyingToSharesResponse)]
    UnderlyingToShares { amount: Uint128 },

    #[returns(StrategyManagerResponse)]
    GetStrategyManager {},

    #[returns(UnderlyingTokenResponse)]
    GetUnderlyingToken {},

    #[returns(TotalSharesResponse)]
    GetTotalShares {},

    #[returns(StrategyState)]
    GetStrategyState {},
}

/// Both Strategy Base & Strategy Manager circularly depend on each other.
/// Since we can't circularly import each other, we put [QueryMsg] which is used by
/// StrategyManager here as well.
pub mod strategy_manager {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::Uint128;

    #[cw_serde]
    pub enum QueryMsg {
        GetStakerStrategyShares { staker: String, strategy: String },
    }

    #[cw_serde]
    pub struct StakerStrategySharesResponse {
        pub shares: Uint128,
    }
}
