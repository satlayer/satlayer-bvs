use cosmwasm_std::{Addr, Uint128, Timestamp, Binary};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

pub const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
pub const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsSubmission {
    pub strategies_and_multipliers: Vec<StrategyAndMultiplier>,
    pub token: Addr,    
    pub amount: Uint128,
    pub start_timestamp: Timestamp,
    pub duration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StrategyAndMultiplier {
    pub strategy: Addr, 
    pub multiplier: u64,
}

pub fn calculate_rewards_submission_hash(sender: &Addr, nonce: u64, submission: &RewardsSubmission) -> Binary {
    let sender_bytes = sender.as_bytes();
    let nonce_bytes = nonce.to_be_bytes();

    let submission_bytes = serde_json::to_vec(submission).expect("Failed to serialize submission");

    let mut hasher = Sha256::new();
    hasher.update(sender_bytes);
    hasher.update(nonce_bytes);
    hasher.update(submission_bytes);

    Binary::new(hasher.finalize().to_vec())
}
