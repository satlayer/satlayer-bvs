use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    IncreaseDelegatedShares {
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsDelegated { staker: Addr },

    #[returns(bool)]
    IsOperator { operator: Addr },
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}
