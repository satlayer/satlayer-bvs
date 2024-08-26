use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct ValueResponse {
    pub value: i64,
}
