use cosmwasm_std::{Addr, Uint128, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SlashDetails {
    pub slasher: Addr,
    pub operator: Addr,
    pub share: Uint128,
    pub slash_signature: u64,
    pub slash_validator: Vec<Addr>,
    pub reason: String,
    pub start_time: u64,
    pub end_time: u64,
    pub status: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteSlashDetails {
    pub slasher: String,
    pub operator: String,
    pub share: Uint128,
    pub slash_signature: u64,
    pub slash_validator: Vec<String>,
    pub reason: String,
    pub start_time: u64,
    pub end_time: u64,
    pub status: bool,
}

pub fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn calculate_slash_hash(
    sender: &Addr,
    slash_details: &SlashDetails,
) -> Vec<u8> {
    let sender_bytes = sender.as_bytes();

    let slash_details_bytes = serde_json::to_vec(slash_details).expect("Failed to serialize submission");

    let mut hasher = Sha256::new();
    hasher.update(sender_bytes);
    hasher.update(slash_details_bytes);

    hasher.finalize().to_vec()
}

pub fn validate_addresses(api: &dyn Api, validators: &[String]) -> StdResult<Vec<Addr>> {
    validators
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}
