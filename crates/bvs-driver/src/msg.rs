use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteBvsOffchain { task_id: String },
    AddRegisteredBvsContract { address: String },
    TransferOwnership { new_owner: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
