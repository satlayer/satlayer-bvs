use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        recipient: Addr,
        amount_shares: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares { user: Addr },

    #[returns(Uint128)]
    SharesToUnderlyingView { amount_shares: Uint128 },

    #[returns(Uint128)]
    UnderlyingToShareView { amount: Uint128 },

    #[returns(StrategyState)]
    GetStrategyState {},
}

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct StrategyState {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
    pub total_shares: Uint128,
}
