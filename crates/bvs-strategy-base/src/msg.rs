use crate::state::StrategyState;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

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
    Shares { staker: String },

    #[returns(UnderlyingResponse)]
    Underlying { staker: String },

    #[returns(SharesToUnderlyingResponse)]
    SharesToUnderlying { shares: Uint128 },

    #[returns(UnderlyingToSharesResponse)]
    UnderlyingToShares { amount: Uint128 },

    #[returns(StrategyManagerResponse)]
    StrategyManager {},

    #[returns(UnderlyingTokenResponse)]
    UnderlyingToken {},

    #[returns(TotalSharesResponse)]
    TotalShares {},

    #[returns(StrategyState)]
    GetStrategyState {},
}

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct UnderlyingResponse {
    pub amount_to_send: Uint128,
}

#[cw_serde]
pub struct SharesToUnderlyingResponse {
    pub amount_to_send: Uint128,
}

#[cw_serde]
pub struct UnderlyingToSharesResponse {
    pub share_to_send: Uint128,
}

#[cw_serde]
pub struct StrategyManagerResponse {
    pub strategy_manager_addr: Addr,
}

#[cw_serde]
pub struct UnderlyingTokenResponse {
    pub underlying_token_addr: Addr,
}

#[cw_serde]
pub struct TotalSharesResponse {
    pub total_shares: Uint128,
}

/// Both Strategy Base & Strategy Manager circularly depend on each other.
/// Since we can't circularly import each other, we put [QueryMsg] which is used by
/// StrategyManager here as well.
pub mod strategy_manager {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_binary, Deps, QueryRequest, StdResult, Uint128, WasmQuery};

    #[cw_serde]
    pub enum QueryMsg {
        GetStakerStrategyShares { staker: String, strategy: String },
    }

    #[cw_serde]
    pub struct StakerStrategySharesResponse {
        pub shares: Uint128,
    }

    /// Get the shares of a staker in a strategy
    /// This information is stored in the Strategy Manager
    pub fn get_staker_strategy_shares(
        deps: Deps,
        strategy_manager: String,
        strategy: String,
        staker: String,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::GetStakerStrategyShares { staker, strategy };

        let response: StakerStrategySharesResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: strategy_manager,
                msg: to_json_binary(&msg)?,
            }))?;

        Ok(response.shares)
    }
}
