use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {
        amount: Uint256,
    },
    Withdraw {
        recipient: Addr,
        amount_shares: Uint256,
    },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SharesResponse)]
    GetShares {},
}

#[cw_serde]
pub struct SharesResponse {
    pub total_shares: Uint256,
}
