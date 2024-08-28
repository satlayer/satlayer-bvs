use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

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
    #[returns(bool)]
    IsDelegated { staker: String },

    #[returns(bool)]
    IsOperator { operator: String },
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}
