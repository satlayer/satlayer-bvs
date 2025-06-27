use crate::error::RewardsError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, Uint128};
use rs_merkle::Hasher;
use sha3::{Digest, Sha3_256};

/// Implements the SHA3-256 hashing algorithm
#[derive(Clone)]
#[allow(dead_code)]
pub struct Sha3_256Algorithm {}

impl Hasher for Sha3_256Algorithm {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha3_256::new();

        hasher.update(data);
        <[u8; 32]>::from(hasher.finalize())
    }
}

#[cw_serde]
pub struct Leaf {
    pub earner: String,
    pub amount: Uint128,
}

impl Leaf {
    // double hash the leaf
    pub fn hash(&self) -> [u8; 32] {
        let leaf = format!("{}{}", self.earner, self.amount);
        Sha3_256Algorithm::hash(Sha3_256Algorithm::hash(leaf.as_bytes()).as_ref())
    }
}

pub fn verify_merkle_proof(
    root: &HexBinary,
    proof: Vec<HexBinary>,
    leaf: Leaf,
    leaf_index: u32,
    total_leaves_count: u32,
) -> Result<bool, RewardsError> {
    let leaf_hash = leaf.hash();

    let proof_bytes: Vec<[u8; 32]> = proof.iter().map(|s| s.to_array().unwrap()).collect();

    // convert leaf index into usize
    let leaf_index: usize = leaf_index
        .try_into()
        .map_err(|_| RewardsError::InvalidProof {
            msg: "Leaf index is too large".to_string(),
        })?;

    // convert total leaves count into usize
    let total_leaves_count: usize =
        total_leaves_count
            .try_into()
            .map_err(|_| RewardsError::InvalidProof {
                msg: "Total leaves count is too large".to_string(),
            })?;

    Ok(
        rs_merkle::MerkleProof::<Sha3_256Algorithm>::new(proof_bytes).verify(
            root.to_array()?,
            &[leaf_index],
            &[leaf_hash],
            total_leaves_count,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{generate_merkle_proof, generate_merkle_tree};
    use cosmwasm_std::HexBinary;

    #[test]
    fn test_leaf_hash() {
        let earner = "bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894".to_string();
        let amount = Uint128::new(100_000_000_000_000_000);

        let expected_hash = "0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d29";
        assert_eq!(
            HexBinary::from_hex(expected_hash)
                .unwrap()
                .to_array()
                .unwrap(),
            Sha3_256Algorithm::hash(format!("{earner}{amount}").as_bytes())
        );
    }

    #[test]
    fn test_verify_merkle_proof() {
        let leaf = Leaf {
            earner: "earner1".to_string(),
            amount: Uint128::new(100),
        };
        let leaf_index = 0u32;

        let total_leaves_count = 1u32;

        let tree = generate_merkle_tree(&[leaf.clone()]);
        let root = HexBinary::from(tree.root().unwrap());

        let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

        let res = verify_merkle_proof(&root, proof, leaf, leaf_index, total_leaves_count);
        assert!(res.unwrap());
    }

    #[test]
    fn test_verify_complex_merkle_proof() {
        let leaves = vec![
            Leaf {
                earner: "earner1".to_string(),
                amount: Uint128::new(100),
            },
            Leaf {
                earner: "earner2".to_string(),
                amount: Uint128::new(300),
            },
            Leaf {
                earner: "earner3".to_string(),
                amount: Uint128::new(50),
            },
            Leaf {
                earner: "earner4".to_string(),
                amount: Uint128::new(99),
            },
            Leaf {
                earner: "earner5".to_string(),
                amount: Uint128::new(1000),
            },
            Leaf {
                earner: "earner6".to_string(),
                amount: Uint128::new(1000),
            },
            Leaf {
                earner: "earner7".to_string(),
                amount: Uint128::new(1000),
            },
        ];

        let total_leaves_count = leaves.len() as u32;

        let tree = generate_merkle_tree(&leaves);
        let root = HexBinary::from(tree.root().unwrap());

        {
            // Verify Success
            let leaf_index = 6u32;
            let leaf_to_proof = leaves[leaf_index as usize].clone();
            let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

            let res =
                verify_merkle_proof(&root, proof, leaf_to_proof, leaf_index, total_leaves_count);
            assert!(res.unwrap());
        }
        {
            // Verify Failure - different amount
            let leaf_index = 6u32;
            let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

            let fake_leaf = Leaf {
                earner: "earner7".to_string(),
                amount: Uint128::new(999),
            };

            let res = verify_merkle_proof(&root, proof, fake_leaf, leaf_index, total_leaves_count);
            assert!(!res.unwrap());
        }
        {
            // Verify Failure - different earner
            let leaf_index = 6u32;
            let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

            let fake_leaf = Leaf {
                earner: "earner8".to_string(),
                amount: Uint128::new(1000),
            };

            let res = verify_merkle_proof(&root, proof, fake_leaf, leaf_index, total_leaves_count);
            assert!(!res.unwrap());
        }
        {
            // Verify Failure - different leaf index
            let leaf_index = 6u32;
            let leaf_to_proof = leaves[leaf_index as usize].clone();
            let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

            let fake_leaf_index = 5u32;

            let res = verify_merkle_proof(
                &root,
                proof,
                leaf_to_proof,
                fake_leaf_index,
                total_leaves_count,
            );
            assert!(!res.unwrap());
        }
        {
            // Verify Failure - different total leaves count
            let leaf_index = 6u32;
            let leaf_to_proof = leaves[leaf_index as usize].clone();
            let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

            let fake_total_leaf_count = 8u32;

            let res = verify_merkle_proof(
                &root,
                proof,
                leaf_to_proof,
                leaf_index,
                fake_total_leaf_count,
            );
            assert!(!res.unwrap());
        }
    }

    #[test]
    fn test_verify_merkle_proof_fail() {
        let leaf = Leaf {
            earner: "earner1".to_string(),
            amount: Uint128::new(100),
        };
        let leaf_index = 0u32;

        let total_leaves_count = 1u32;

        let tree = generate_merkle_tree(&[leaf.clone()]);
        let root = HexBinary::from(tree.root().unwrap());

        let proof = generate_merkle_proof(&tree, leaf_index).expect("Failed to generate proof");

        let fake_leaf = Leaf {
            earner: "earner1".to_string(),
            amount: Uint128::new(200),
        };

        let res = verify_merkle_proof(&root, proof, fake_leaf, leaf_index, total_leaves_count);
        assert!(!res.unwrap());
    }
}
