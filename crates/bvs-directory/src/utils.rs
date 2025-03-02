use cosmwasm_std::{Addr, Binary};
use sha2::{Digest, Sha256};

pub const OPERATOR_BVS_REGISTRATION_TYPE_HASH: &[u8] =
    b"OperatorBVSRegistration(address operator,address bvs,bytes32 salt,uint256 expiry)";
pub const DOMAIN_TYPE_HASH: &[u8] =
    b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
pub const DOMAIN_NAME: &[u8] = b"SatLayer";

pub fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn calculate_digest_hash(
    chain_id: String,
    operator_public_key: &[u8],
    bvs: &Addr,
    salt: &Binary,
    expiry: u64,
    contract_addr: &Addr,
) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(OPERATOR_BVS_REGISTRATION_TYPE_HASH)[..],
        operator_public_key,
        bvs.as_bytes(),
        salt.as_slice(),
        &expiry.to_le_bytes(),
    ]
    .concat();
    let struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(
        &[
            &sha256(DOMAIN_TYPE_HASH)[..],
            &sha256(DOMAIN_NAME)[..],
            chain_id.as_bytes(),
            contract_addr.as_bytes(),
        ]
        .concat(),
    );

    let digest_hash_input = [b"\x19\x01", &domain_separator[..], &struct_hash[..]].concat();

    sha256(&digest_hash_input)
}
