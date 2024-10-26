use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteBvsOffchain { task_id: u64 },
    AddRegisteredBvsContract { address: String },
}

#[cw_serde]
pub enum QueryMsg {}
