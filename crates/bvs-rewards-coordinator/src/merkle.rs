use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Binary, HexBinary, StdResult, Timestamp, Uint128};
use sha2::{Digest, Sha256};

pub const EARNER_LEAF_SALT: u8 = 0;
pub const TOKEN_LEAF_SALT: u8 = 1;

pub fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

#[cw_serde]
pub struct RewardsSubmission {
    pub strategies_and_multipliers: Vec<StrategyAndMultiplier>,
    /// token contract address
    pub token: Addr, // TODO: validate Addr string
    /// token amount to be transferred to the contract as rewards to be distributed
    pub amount: Uint128,
    /// start_timestamp must be multiple of calculation_interval_seconds
    pub start_timestamp: Timestamp,
    /// duration must be multiple of calculation_interval_seconds
    pub duration: u64,
}

#[cw_serde]
pub struct StrategyAndMultiplier {
    /// strategy contract address
    pub strategy: Addr, // TODO: validate Addr string
    /// TODO: add desc/usage
    pub multiplier: u64,
}

#[cw_serde]
pub struct TokenTreeMerkleLeaf {
    pub token: Addr,
    pub cumulative_earnings: Uint128,
}

#[cw_serde]
pub struct EarnerTreeMerkleLeaf {
    pub earner: Addr,
    pub earner_token_root: HexBinary,
}

#[cw_serde]
pub struct RewardsMerkleClaim {
    pub root_index: u32,
    pub earner_index: u32,
    pub earner_tree_proof: Vec<u8>,
    pub earner_leaf: EarnerTreeMerkleLeaf,
    pub token_indices: Vec<u32>,
    pub token_tree_proofs: Vec<Vec<u8>>,
    pub token_leaves: Vec<TokenTreeMerkleLeaf>,
}

impl RewardsMerkleClaim {
    pub fn validate(&self, api: &dyn Api) -> StdResult<()> {
        api.addr_validate(self.earner_leaf.earner.as_str())?;
        for leaf in self.token_leaves.iter() {
            api.addr_validate(leaf.token.as_str())?;
        }
        Ok(())
    }
}

pub fn calculate_rewards_submission_hash(
    sender: &Addr,
    nonce: u64,
    submission: &RewardsSubmission,
) -> Binary {
    let sender_bytes = sender.as_bytes();
    let nonce_bytes = nonce.to_be_bytes();

    let submission_bytes = serde_json::to_vec(submission).expect("Failed to serialize submission");

    let mut hasher = Sha256::new();
    hasher.update(sender_bytes);
    hasher.update(nonce_bytes);
    hasher.update(submission_bytes);

    Binary::new(hasher.finalize().to_vec())
}

/// Calculates the hash of an earner leaf
pub fn calculate_earner_leaf_hash(leaf: &EarnerTreeMerkleLeaf) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update([EARNER_LEAF_SALT]);
    hasher.update(leaf.earner.as_bytes());
    hasher.update(leaf.earner_token_root.as_slice());
    hasher.finalize().to_vec()
}

/// Calculates the hash of a token leaf
pub fn calculate_token_leaf_hash(leaf: &TokenTreeMerkleLeaf) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update([TOKEN_LEAF_SALT]);
    hasher.update(leaf.token.as_bytes());
    hasher.update(leaf.cumulative_earnings.to_be_bytes());
    hasher.finalize().to_vec()
}

pub fn merkleize_sha256(mut leaves: Vec<Vec<u8>>) -> Vec<u8> {
    assert!(
        leaves.len().is_power_of_two(),
        "the number of leaf nodes must be a power of two"
    );

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

/// verify_inclusion_sha256 verifies that a `leaf` is included in a Merkle tree given the `proof` and `root`
/// if the `leaf` is included in the tree, the generated root will match the `root`
pub fn verify_inclusion_sha256(proof: &[u8], root: &[u8], leaf: &[u8], index: u64) -> bool {
    process_inclusion_proof_sha256(proof, leaf, index) == root
}

/// process_inclusion_proof_sha256 will regenerate the merkle root from the `leaf` and `proof`
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

pub fn get_merkle_proof() {}

mod tests {
    use crate::merkle::{
        calculate_earner_leaf_hash, calculate_token_leaf_hash, merkleize_sha256, sha256,
        verify_inclusion_sha256, EarnerTreeMerkleLeaf, TokenTreeMerkleLeaf,
    };
    use cosmwasm_std::{Addr, HexBinary, Uint128};

    #[test]
    fn test_token_leaf_merkle_tree_construction() {
        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        // calculate root hash
        let leaves_ab = vec![hash_a.clone(), hash_b.clone()];
        let parent_ab = merkleize_sha256(leaves_ab.clone());

        let leaves_cd = vec![hash_c.clone(), hash_d.clone()];
        let parent_cd = merkleize_sha256(leaves_cd.clone());

        let parent_hash = vec![parent_ab.clone(), parent_cd.clone()];
        let root_hash = merkleize_sha256(parent_hash.clone());

        assert_eq!(
            HexBinary::from(root_hash.clone()).to_hex(),
            "9c587713ca04be4102d530e6b0a7ad44a62c451fd19338810802fe6e6273c991"
        );

        assert_eq!(
            parent_ab,
            sha256(&[hash_a.as_slice(), hash_b.as_slice()].concat()),
            "Parent AB hash is incorrect"
        );
        assert_eq!(
            parent_cd,
            sha256(&[hash_c.as_slice(), hash_d.as_slice()].concat()),
            "Parent CD hash is incorrect"
        );
    }

    #[test]
    fn test_earner_leaf_merkle_tree_construction() {
        let token_leaves_sets = vec![
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a1"),
                    cumulative_earnings: Uint128::new(100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a2"),
                    cumulative_earnings: Uint128::new(200),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a3"),
                    cumulative_earnings: Uint128::new(300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a4"),
                    cumulative_earnings: Uint128::new(400),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b1"),
                    cumulative_earnings: Uint128::new(500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b2"),
                    cumulative_earnings: Uint128::new(600),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b3"),
                    cumulative_earnings: Uint128::new(700),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b4"),
                    cumulative_earnings: Uint128::new(800),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c1"),
                    cumulative_earnings: Uint128::new(900),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c2"),
                    cumulative_earnings: Uint128::new(1000),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c3"),
                    cumulative_earnings: Uint128::new(1100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c4"),
                    cumulative_earnings: Uint128::new(1200),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d1"),
                    cumulative_earnings: Uint128::new(1300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d2"),
                    cumulative_earnings: Uint128::new(1400),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d3"),
                    cumulative_earnings: Uint128::new(1500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d4"),
                    cumulative_earnings: Uint128::new(1600),
                },
            ],
        ];

        // Calculate Merkle roots for each set of token leaves
        let mut merkle_roots = Vec::new();

        for leaves in token_leaves_sets {
            let mut leaf_hashes = Vec::new();
            for leaf in leaves {
                leaf_hashes.push(calculate_token_leaf_hash(&leaf));
            }
            let merkle_root = merkleize_sha256(leaf_hashes);
            merkle_roots.push(merkle_root.clone());
        }

        // Assertions & Print root hash for calculate_earner_leaf_hash
        for (i, merkle_root) in merkle_roots.iter().enumerate() {
            assert!(
                !merkle_root.is_empty(),
                "Merkle root for tree {} should not be empty",
                i + 1
            );
        }

        let tree1_root_hash = [
            48, 187, 24, 98, 230, 203, 235, 218, 90, 43, 190, 153, 209, 248, 126, 128, 198, 194,
            113, 131, 32, 46, 106, 102, 115, 45, 214, 230, 122, 67, 222, 244,
        ];
        let tree2_root_hash = [
            31, 173, 229, 179, 199, 27, 21, 153, 215, 61, 227, 184, 156, 136, 11, 226, 144, 224,
            214, 117, 192, 110, 116, 32, 123, 117, 254, 131, 59, 205, 178, 221,
        ];
        let tree3_root_hash = [
            241, 77, 172, 5, 228, 0, 249, 31, 159, 211, 176, 37, 20, 123, 30, 159, 62, 148, 250,
            97, 101, 206, 14, 35, 211, 217, 181, 123, 237, 149, 14, 220,
        ];
        let tree4_root_hash = [
            114, 34, 142, 99, 115, 93, 244, 227, 187, 171, 41, 53, 218, 109, 87, 55, 75, 87, 46,
            220, 50, 151, 15, 77, 78, 255, 183, 253, 198, 47, 244, 132,
        ];

        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: HexBinary::from(tree1_root_hash.to_vec()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: HexBinary::from(tree2_root_hash.to_vec()),
        };
        let earner3 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner3"),
            earner_token_root: HexBinary::from(tree3_root_hash.to_vec()),
        };
        let earner4 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner4"),
            earner_token_root: HexBinary::from(tree4_root_hash.to_vec()),
        };

        // Calculate earner leaf hashes
        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);
        let earner_leaf_hash3 = calculate_earner_leaf_hash(&earner3);
        let earner_leaf_hash4 = calculate_earner_leaf_hash(&earner4);

        let leaves = vec![
            earner_leaf_hash1.clone(),
            earner_leaf_hash2.clone(),
            earner_leaf_hash3.clone(),
            earner_leaf_hash4.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        // Expected parent hash & Expected root hash
        let leaves_1_2 = vec![earner_leaf_hash1.clone(), earner_leaf_hash2.clone()];
        let parent_1_2 = merkleize_sha256(leaves_1_2.clone());

        let leaves_3_4 = vec![earner_leaf_hash3.clone(), earner_leaf_hash4.clone()];
        let parent_3_4 = merkleize_sha256(leaves_3_4.clone());

        let parent_hash = vec![parent_1_2.clone(), parent_3_4.clone()];
        let expected_root_hash = merkleize_sha256(parent_hash.clone());

        assert!(!merkle_root.is_empty(), "Merkle root should not be empty");
        assert_eq!(merkle_root, expected_root_hash);

        assert_eq!(
            parent_1_2,
            sha256(&[earner_leaf_hash1.as_slice(), earner_leaf_hash2.as_slice()].concat()),
            "Parent 1 2 hash is incorrect"
        );
        assert_eq!(
            parent_3_4,
            sha256(&[earner_leaf_hash3.as_slice(), earner_leaf_hash4.as_slice()].concat()),
            "Parent 3 4 hash is incorrect"
        );
    }

    #[test]
    fn test_verify_inclusion_proof() {
        let token_leaves_sets = vec![
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a1"),
                    cumulative_earnings: Uint128::new(100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a2"),
                    cumulative_earnings: Uint128::new(200),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a3"),
                    cumulative_earnings: Uint128::new(300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a4"),
                    cumulative_earnings: Uint128::new(400),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b1"),
                    cumulative_earnings: Uint128::new(500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b2"),
                    cumulative_earnings: Uint128::new(600),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b3"),
                    cumulative_earnings: Uint128::new(700),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b4"),
                    cumulative_earnings: Uint128::new(800),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c1"),
                    cumulative_earnings: Uint128::new(900),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c2"),
                    cumulative_earnings: Uint128::new(1000),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c3"),
                    cumulative_earnings: Uint128::new(1100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c4"),
                    cumulative_earnings: Uint128::new(1200),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d1"),
                    cumulative_earnings: Uint128::new(1300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d2"),
                    cumulative_earnings: Uint128::new(1400),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d3"),
                    cumulative_earnings: Uint128::new(1500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d4"),
                    cumulative_earnings: Uint128::new(1600),
                },
            ],
        ];

        let mut merkle_roots = Vec::new();

        for leaves in &token_leaves_sets {
            let mut leaf_hashes = Vec::new();
            for leaf in leaves {
                leaf_hashes.push(calculate_token_leaf_hash(leaf));
            }
            let merkle_root = merkleize_sha256(leaf_hashes.clone());
            merkle_roots.push(merkle_root.clone());

            println!("Merkle Root: {:?}", merkle_root);
        }

        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: HexBinary::from(merkle_roots[0].clone()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: HexBinary::from(merkle_roots[1].clone()),
        };
        let earner3 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner3"),
            earner_token_root: HexBinary::from(merkle_roots[2].clone()),
        };
        let earner4 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner4"),
            earner_token_root: HexBinary::from(merkle_roots[3].clone()),
        };

        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);
        let earner_leaf_hash3 = calculate_earner_leaf_hash(&earner3);
        let earner_leaf_hash4 = calculate_earner_leaf_hash(&earner4);

        let leaves = vec![
            earner_leaf_hash1.clone(),
            earner_leaf_hash2.clone(),
            earner_leaf_hash3.clone(),
            earner_leaf_hash4.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        let leaves_3_4 = vec![earner_leaf_hash3.clone(), earner_leaf_hash4.clone()];
        let parent_3_4 = merkleize_sha256(leaves_3_4.clone());

        let leaves_1_2 = vec![earner_leaf_hash1.clone(), earner_leaf_hash2.clone()];
        let parent_1_2 = merkleize_sha256(leaves_1_2.clone());

        // Generate proof for earner1 leaf
        let proof1 = [earner_leaf_hash2.clone(), parent_3_4.clone()];
        let proof2 = [earner_leaf_hash1.clone(), parent_3_4.clone()];
        let proof3 = [earner_leaf_hash4.clone(), parent_1_2.clone()];
        let proof4 = [earner_leaf_hash3.clone(), parent_1_2.clone()];

        assert!(verify_inclusion_sha256(
            &proof1.concat(),
            &merkle_root,
            &earner_leaf_hash1,
            0
        ));

        assert!(verify_inclusion_sha256(
            &proof2.concat(),
            &merkle_root,
            &earner_leaf_hash2,
            1
        ));

        assert!(verify_inclusion_sha256(
            &proof3.concat(),
            &merkle_root,
            &earner_leaf_hash3,
            2
        ));

        assert!(verify_inclusion_sha256(
            &proof4.concat(),
            &merkle_root,
            &earner_leaf_hash4,
            3
        ));
    }
}
