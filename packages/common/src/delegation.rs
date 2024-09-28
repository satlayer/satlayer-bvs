use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    IncreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    DecreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsDelegated { staker: String },

    #[returns(bool)]
    IsOperator { operator: String },

    #[returns(OperatorStakersResponse)]
    GetOperatorStakers { operator: String },
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}

#[cw_serde]
pub struct OperatorStakersResponse {
    pub stakers_and_shares: Vec<StakerShares>,
}

#[cw_serde]
pub struct StakerShares {
    pub staker: Addr,
    pub shares_per_strategy: Vec<(Addr, Uint128)>,
}
