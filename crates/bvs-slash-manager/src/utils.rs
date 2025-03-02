use cosmwasm_crypto::secp256k1_verify;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Uint128};
use sha2::{Digest, Sha256};

#[cw_serde]
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

#[cw_serde]
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

pub fn calculate_slash_hash(
    sender: &Addr,
    slash_details: &SlashDetails,
    contract_addr: &Addr,
    validator_public_keys: &[Vec<u8>],
) -> Vec<u8> {
    let sender_bytes = sender.as_bytes();
    let contract_addr_bytes = contract_addr.as_bytes();
    let slash_details_bytes = serde_json::to_vec(slash_details).expect("Serialization failed");

    let mut hasher = Sha256::new();
    hasher.update(sender_bytes);
    hasher.update(slash_details_bytes);
    hasher.update(contract_addr_bytes);

    for public_key in validator_public_keys {
        hasher.update(public_key);
    }

    hasher.finalize().to_vec()
}

pub fn recover(digest_hash: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> StdResult<bool> {
    match secp256k1_verify(digest_hash, signature, public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}
