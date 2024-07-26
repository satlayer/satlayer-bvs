use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: Addr,
    pub eigen_pod_manager: Addr,
    pub slasher: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        strategy: Addr,
        amount: Uint128,
    },
    Withdraw {
        strategy: Addr,
        amount_shares: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares {
        user: Addr,
    },
}

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint128,
}
