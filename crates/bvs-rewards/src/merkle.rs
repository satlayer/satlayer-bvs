use crate::error::RewardsError;
use cosmwasm_std::Binary;
use rs_merkle::algorithms::Sha256;
use rs_merkle::Hasher;

pub struct Leaf {
    pub earner: String,
    pub amount: String,
}

pub fn leaf_hash(earner: String, amount: String) -> [u8; 32] {
    let leaf = format!("{}{}", earner, amount);
    Sha256::hash(leaf.as_bytes())
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

    Ok(rs_merkle::MerkleProof::<Sha256>::new(proof_bytes).verify(
        root.to_array()?,
        &[leaf_index],
        &[leaf_hash],
        total_leaves_count,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::HexBinary;

    #[test]
    fn test_leaf_hash() {
        let earner = "bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894".to_string();
        let amount = "100000000000000000".to_string();

        let expected_hash = "32e1d1f090750f2a02f515dfa3e6ab69c2364c1ac498c52134c6151a414abea4";
        assert_eq!(
            HexBinary::from_hex(expected_hash)
                .unwrap()
                .to_array()
                .unwrap(),
            Sha256::hash(format!("{}{}", earner, amount).as_bytes())
        );
    }

    // #[test]
    // fn test_verify_merkle_proof() {
    //     let root = Binary::from(vec![0; 32]);
    //     let proof = vec![Binary::from(vec![0; 32])];
    //     let leaf = Leaf {
    //         earner: "earner1".to_string(),
    //         amount: "100".to_string(),
    //     };
    //     let leaf_index = 0;
    //
    //     let total_leaves_count = 1;
    //
    //     assert!(verify_merkle_proof(root, proof, leaf, leaf_index, total_leaves_count).is_ok());
    // }
}
