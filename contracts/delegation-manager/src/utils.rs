use cosmwasm_std::{Addr, StdResult, StdError, Uint128, Uint64, Binary};
use sha2::{Sha256, Digest};
use cosmwasm_crypto::secp256k1_verify;
use bech32::{self, FromBase32};

const DELEGATION_TYPEHASH: &[u8] = b"DelegateTo(address staker,address operator,uint256 nonce,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}


pub struct DigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub nonce: u64,
    pub expiry: u64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_digest_hash(params: &DigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(DELEGATION_TYPEHASH)[..],
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        &params.nonce.to_le_bytes(),
        &params.expiry.to_le_bytes(),
    ]
    .concat();
    let struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(&[
        &sha256(DOMAIN_TYPEHASH)[..],
        &sha256(DOMAIN_NAME)[..],
        &sha256(params.chain_id.as_bytes())[..],
        params.contract_addr.as_bytes(),
    ].concat());

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &struct_hash[..],
    ]
    .concat();

    sha256(&digest_hash_input)
}

pub fn recover(digest_hash: &[u8], signature: &[u8], staker: &Addr) -> StdResult<bool> {
    let (_hrp, data, _variant) = bech32::decode(staker.as_str()).map_err(|_| StdError::generic_err("Invalid staker address"))?;
    let public_key_bytes = Vec::<u8>::from_base32(&data).map_err(|_| StdError::generic_err("Invalid staker address"))?;

    match secp256k1_verify(digest_hash, signature, &public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}
