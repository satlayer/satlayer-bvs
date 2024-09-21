use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteAvsOffchain { task_id: u64 },
    AddRegisteredAvsContract { address: String },
}

#[cw_serde]
pub enum QueryMsg {}
