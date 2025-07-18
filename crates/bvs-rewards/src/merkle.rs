use crate::error::RewardsError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, Uint128};
use rs_merkle::Hasher;
use sha3::{Digest, Keccak256};

/// Implements the Keccak256 hashing algorithm
#[derive(Clone)]
#[allow(dead_code)]
pub struct Keccak256Algorithm {}

impl Hasher for Keccak256Algorithm {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();

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
        Keccak256Algorithm::hash(Keccak256Algorithm::hash(leaf.as_bytes()).as_ref())
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
        rs_merkle::MerkleProof::<Keccak256Algorithm>::new(proof_bytes).verify(
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
    fn test_keccak256() {
        // to make sure that the keccak256 algorithm is compatible with solidity keccak256
        let res = Keccak256Algorithm::hash("".as_bytes());
        assert_eq!(
            res,
            HexBinary::from_hex("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
                .unwrap()
                .to_array()
                .unwrap()
        )
    }

    #[test]
    fn test_leaf_hash() {
        let earner = "bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894".to_string();
        let amount = Uint128::new(100_000_000_000_000_000);

        let expected_hash = "bee6cd3b687cd34ea235eb51432375dc7be5a5d6d1d26d23beff099dbf88eeb5";
        assert_eq!(
            HexBinary::from_hex(expected_hash)
                .unwrap()
                .to_array()
                .unwrap(),
            Keccak256Algorithm::hash(format!("{earner}{amount}").as_bytes())
        );
    }

    #[test]
    fn test_verify_merkle_proof_hardcoded() {
        let root =
            HexBinary::from_hex("4b83dc8ecaa7a9d69ac8a7c12718eed8639e1ba1a1b30a51741ccfd020255cec")
                .unwrap();
        let leaf = Leaf {
            earner: "bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894".to_string(),
            amount: Uint128::new(100000000000000000),
        };
        let leaf_index = 1u32;
        let total_leaf_count = 2u32;

        let proof = vec![HexBinary::from_hex(
            "614a2406c3e74dca5a75c5429158a486c9d0d9eb5efbea928cc309beb6b3fce6",
        )
        .unwrap()];

        assert!(verify_merkle_proof(&root, proof, leaf, leaf_index, total_leaf_count).unwrap());
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
    fn test_verify_complex_merkle_proof_hardcoded() {
        let root =
            HexBinary::from_hex("1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4")
                .unwrap();
        let leaf = Leaf {
            earner: "bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b".to_string(),
            amount: Uint128::new(600000000000000000),
        };
        let leaf_index = 7u32;
        let total_leaf_count = 9u32;

        let proof = vec![
            HexBinary::from_hex("c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5")
                .unwrap(),
            HexBinary::from_hex("662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa")
                .unwrap(),
            HexBinary::from_hex("8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256")
                .unwrap(),
            HexBinary::from_hex("afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe")
                .unwrap(),
        ];

        assert!(verify_merkle_proof(&root, proof, leaf, leaf_index, total_leaf_count).unwrap());
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
