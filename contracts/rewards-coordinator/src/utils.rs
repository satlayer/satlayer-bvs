use cosmwasm_std::{Addr, Uint128, Timestamp, Binary};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

pub const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
pub const DOMAIN_NAME: &[u8] = b"EigenLayer";

pub const EARNER_LEAF_SALT: u8 = 0;
pub const TOKEN_LEAF_SALT: u8 = 1;

pub fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn calculate_domain_separator(chain_id: &str, contract_addr: &Addr) -> Vec<u8> {
    let encoded_data = [
        DOMAIN_TYPEHASH,
        &sha256(DOMAIN_NAME),
        chain_id.as_bytes(),
        contract_addr.as_bytes(),
    ]
    .concat();

    sha256(&encoded_data)
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTreeMerkleLeaf {
    pub token: Addr,
    pub cumulative_earnings: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EarnerTreeMerkleLeaf {
    pub earner: Addr,
    pub earner_token_root: Binary,
}

pub fn calculate_earner_leaf_hash(leaf: &EarnerTreeMerkleLeaf) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update([EARNER_LEAF_SALT]);
    hasher.update(leaf.earner.as_bytes());
    hasher.update(leaf.earner_token_root.as_slice());
    hasher.finalize().to_vec()
}

pub fn calculate_token_leaf_hash(leaf: &TokenTreeMerkleLeaf) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update([TOKEN_LEAF_SALT]);
    hasher.update(leaf.token.as_bytes());
    hasher.update(leaf.cumulative_earnings.to_be_bytes());
    hasher.finalize().to_vec()
}

pub fn merkleize_sha256(mut leaves: Vec<Vec<u8>>) -> Vec<u8> {
    assert!(leaves.len().is_power_of_two(), "the number of leaf nodes must be a power of two");

    while leaves.len() > 1 {
        let mut next_layer = Vec::with_capacity(leaves.len() / 2);

        for i in (0..leaves.len()).step_by(2) {
            let combined = [leaves[i].as_slice(), leaves[i + 1].as_slice()].concat();
            let parent_hash = sha256(&combined);
            next_layer.push(parent_hash);
        }

        leaves = next_layer;
    }

    leaves[0].clone()
}

pub fn verify_inclusion_sha256(proof: &[u8], root: &[u8], leaf: &[u8], index: u64) -> bool {
    process_inclusion_proof_sha256(proof, leaf, index) == root
}

fn process_inclusion_proof_sha256(proof: &[u8], leaf: &[u8], index: u64) -> Vec<u8> {
    if proof.len() % 32 != 0 {
        panic!("Proof length should be a multiple of 32");
    }

    let mut computed_hash = leaf.to_vec();
    let mut index = index;

    for i in (0..proof.len()).step_by(32) {
        let proof_element = &proof[i..i + 32];

        if index % 2 == 0 {
            computed_hash = sha256(&[&computed_hash, proof_element].concat());
        } else {
            computed_hash = sha256(&[proof_element, &computed_hash].concat());
        }

        index /= 2;
    }

    computed_hash
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsMerkleClaim {
    pub root_index: u32,
    pub earner_index: u32,
    pub earner_tree_proof: Vec<u8>,
    pub earner_leaf: EarnerTreeMerkleLeaf,
    pub token_indices: Vec<u32>,
    pub token_tree_proofs: Vec<Vec<u8>>,
    pub token_leaves: Vec<TokenTreeMerkleLeaf>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExeuteRewardsMerkleClaim {
    pub root_index: u32,
    pub earner_index: u32,
    pub earner_tree_proof: Vec<u8>,
    pub earner_leaf: ExecuteEarnerTreeMerkleLeaf,
    pub token_indices: Vec<u32>,
    pub token_tree_proofs: Vec<Vec<u8>>,
    pub token_leaves: Vec<TokenTreeMerkleLeaf>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteEarnerTreeMerkleLeaf {
    pub earner: Addr,
    pub earner_token_root: String,
}