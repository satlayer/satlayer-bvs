use crate::query::{
    ExplanationResponse, SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse,
    TotalSharesResponse, UnderlyingToShareResponse, UnderlyingToSharesResponse,
    UnderlyingTokenResponse, UserUnderlyingResponse,
};
use crate::state::StrategyState;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub strategy_manager: String,
    pub underlying_token: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        recipient: String,
        token: String,
        amount_shares: Uint128,
    },
    SetStrategyManager {
        new_strategy_manager: String,
    },
    TransferOwnership {
        /// Transfer ownership of the contract to a new owner.
        /// Contract admin (set for all BVS contracts, a cosmwasm feature)
        /// has the omni-ability to override by migration;
        /// this logic is app-level.
        /// > 2-step ownership transfer is mostly redundant for CosmWasm contracts with the admin set.
        /// > You can override ownership with using CosmWasm migrate `entry_point`.
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

    #[returns(ExplanationResponse)]
    Explanation {},

    #[returns(UnderlyingToSharesResponse)]
    UnderlyingToShares { amount_underlying: Uint128 },

    #[returns(StrategyState)]
    GetStrategyState {},
}
