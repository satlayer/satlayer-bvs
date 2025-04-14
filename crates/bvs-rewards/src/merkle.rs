use crate::error::RewardsError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use rs_merkle::Hasher;
use sha3::{Digest, Sha3_256};

/// Implements the SHA3-256 hashing algorithm
#[derive(Clone)]
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
    pub amount: String,
}

pub fn leaf_hash(earner: String, amount: String) -> [u8; 32] {
    let leaf = format!("{}{}", earner, amount);
    Sha3_256Algorithm::hash(leaf.as_bytes())
}

pub fn verify_merkle_proof(
    root: Binary,
    proof: Vec<Binary>,
    leaf: Leaf,
    leaf_index: usize,
    total_leaves_count: usize,
) -> Result<bool, RewardsError> {
    let leaf_hash = leaf_hash(leaf.earner, leaf.amount);

    let proof_bytes: Vec<[u8; 32]> = proof.iter().map(|s| s.to_array().unwrap()).collect();

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
    use cosmwasm_std::HexBinary;
    use rs_merkle::MerkleTree;

    fn generate_merkle_tree(leaves: &Vec<Leaf>) -> MerkleTree<Sha3_256Algorithm> {
        MerkleTree::<Sha3_256Algorithm>::from_leaves(
            leaves
                .iter()
                .map(|leaf| leaf_hash(leaf.earner.clone(), leaf.amount.clone()))
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }

    fn generate_merkle_proof(
        tree: &MerkleTree<Sha3_256Algorithm>,
        leaf_index: usize,
    ) -> Vec<Binary> {
        let proof = tree.proof(&[leaf_index]);
        proof
            .proof_hashes()
            .iter()
            .map(|hash| Binary::from(hash.to_vec()))
            .collect()
    }

    #[test]
    fn test_leaf_hash() {
        let earner = "bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894".to_string();
        let amount = "100000000000000000".to_string();

        let expected_hash = "0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d29";
        assert_eq!(
            HexBinary::from_hex(expected_hash)
                .unwrap()
                .to_array()
                .unwrap(),
            Sha3_256Algorithm::hash(format!("{}{}", earner, amount).as_bytes())
        );
    }

    #[test]
    fn test_verify_merkle_proof() {
        let leaf = Leaf {
            earner: "earner1".to_string(),
            amount: "100".to_string(),
        };
        let leaf_index = 0;

        let total_leaves_count = 1;

        let tree = generate_merkle_tree(&vec![leaf.clone()]);
        let root = Binary::from(tree.root().unwrap());

        let proof = generate_merkle_proof(&tree, leaf_index);

        let res = verify_merkle_proof(root, proof, leaf, leaf_index, total_leaves_count);
        assert_eq!(res.unwrap(), true);
    }

    #[test]
    fn test_verify_complex_merkle_proof() {
        let leaves = vec![
            Leaf {
                earner: "earner1".to_string(),
                amount: "100".to_string(),
            },
            Leaf {
                earner: "earner2".to_string(),
                amount: "300".to_string(),
            },
            Leaf {
                earner: "earner3".to_string(),
                amount: "50".to_string(),
            },
            Leaf {
                earner: "earner4".to_string(),
                amount: "99".to_string(),
            },
            Leaf {
                earner: "earner5".to_string(),
                amount: "1000".to_string(),
            },
            Leaf {
                earner: "earner6".to_string(),
                amount: "1000".to_string(),
            },
            Leaf {
                earner: "earner7".to_string(),
                amount: "1000".to_string(),
            },
        ];

        let total_leaves_count = leaves.len();

        let tree = generate_merkle_tree(&leaves);
        let root = Binary::from(tree.root().unwrap());

        let leaf_index = 6;
        let leaf_to_proof = leaves[leaf_index].clone();
        let proof = generate_merkle_proof(&tree, leaf_index);

        let res = verify_merkle_proof(root, proof, leaf_to_proof, leaf_index, total_leaves_count);
        assert_eq!(res.unwrap(), true);
    }

    #[test]
    fn test_verify_merkle_proof_fail() {
        let leaf = Leaf {
            earner: "earner1".to_string(),
            amount: "100".to_string(),
        };
        let leaf_index = 0;

        let total_leaves_count = 1;

        let tree = generate_merkle_tree(&vec![leaf.clone()]);
        let root = Binary::from(tree.root().unwrap());

        let proof = generate_merkle_proof(&tree, leaf_index);

        let fake_leaf = Leaf {
            earner: "earner1".to_string(),
            amount: "200".to_string(),
        };

        let res = verify_merkle_proof(root, proof, fake_leaf, leaf_index, total_leaves_count);
        assert_eq!(res.unwrap(), false);
    }
}
