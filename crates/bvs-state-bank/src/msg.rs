use crate::query::ValueResponse;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub bvs_directory: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Set { key: String, value: String },
    AddRegisteredBvsContract { address: String },
    TwoStepTransferOwnership { new_owner: String },
    AcceptOwnership {},
    CancelOwnershipTransfer {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResponse)]
    Get { key: String },
}

#[cw_serde]
pub struct MigrateMsg {}
