use cosmwasm_std::{Addr, Binary, StdResult, Uint64};
use sha2::{Sha256, Digest};
use cosmwasm_crypto::secp256k1_verify;

pub const OPERATOR_AVS_REGISTRATION_TYPEHASH: &[u8] = b"OperatorAVSRegistration(address operator,address avs,bytes32 salt,uint256 expiry)";
pub const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
pub const DOMAIN_NAME: &[u8] = b"EigenLayer";

pub fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub struct DigestHashParams {
    pub operator_public_key: Binary,
    pub avs: Addr,
    pub salt: Binary,
    pub expiry: Uint64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_digest_hash(
    operator_public_key: &[u8],
    avs: &Addr,
    salt: &Binary,
    expiry: u64,
    chain_id: &str,
    contract_addr: &Addr,
) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(OPERATOR_AVS_REGISTRATION_TYPEHASH)[..],
        operator_public_key,
        avs.as_bytes(),
        salt.as_slice(),
        &expiry.to_le_bytes(),
    ]
    .concat();
    let struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(&[
        &sha256(DOMAIN_TYPEHASH)[..],
        &sha256(DOMAIN_NAME)[..],
        chain_id.as_bytes(),
        contract_addr.as_bytes(),
    ].concat());

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &struct_hash[..],
    ]
    .concat();

    sha256(&digest_hash_input)
}

pub fn recover(digest_hash: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> StdResult<bool> {
    match secp256k1_verify(digest_hash, signature, public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}