use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    IncreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorResponse)]
    IsOperator { operator: String },
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}
