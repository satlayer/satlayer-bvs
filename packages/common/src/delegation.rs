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

    #[returns(u64)]
    StakerOptOutWindowBlocks { operator: Addr },

    #[returns(Vec<Uint128>)]
    GetOperatorShares {
        operator: Addr,
        strategies: Vec<Addr>,
    },

    #[returns((Vec<Addr>, Vec<Uint128>))]
    GetDelegatableShares { staker: Addr },

    #[returns(Vec<u64>)]
    GetWithdrawalDelay { strategies: Vec<Addr> },
}

#[cw_serde]
pub struct OperatorResponse {
    pub is_operator: bool,
}
