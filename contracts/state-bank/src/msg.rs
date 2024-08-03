use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Set { key: String, value: i32 },
}

#[cw_serde]
pub enum QueryMsg {}