use cosmwasm_std::{Addr, Uint128, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SlashDetails {
    pub slasher: Addr,
    pub operator: Addr,
    pub share: Uint128,
    pub slash_signature: u64,
    pub slash_validator: Vec<Addr>,
    pub reason: String,
    pub start_time: Uint128,
    pub end_time: Uint128,
    pub status: bool,
}

pub fn validate_addresses(api: &dyn Api, validators: &[String]) -> StdResult<Vec<Addr>> {
    validators
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}
