use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub bvs_directory: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteBvsOffchain { task_id: String },
    AddRegisteredBvsContract { address: String },
    SetBVSDirectory { new_directory: String },
    TwoStepTransferOwnership { new_owner: String },
    AcceptOwnership {},
    CancelOwnershipTransfer {},
}

#[cw_serde]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}
