use cosmwasm_std::{Addr, StdResult};
use sha2::{Sha256, Digest};
use cosmwasm_crypto::secp256k1_verify;

const DELEGATION_APPROVAL_TYPEHASH: &[u8] = b"DelegationApproval(address delegationApprover,address staker,address operator,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const STAKER_DELEGATION_TYPEHASH: &[u8] = b"StakerDelegation(address staker,address operator,uint256 nonce,uint256 expiry)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub struct DigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub delegationApprover: Addr,
    pub approver_public_key_hex: String,
    pub nonce: u64,
    pub expiry: u64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_digest_hash(params: &DigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(DELEGATION_APPROVAL_TYPEHASH)[..],
        params.delegationApprover.as_bytes(),
        params.approver_public_key_hex.as_bytes(),
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        &params.nonce.to_le_bytes(),
        &params.expiry.to_le_bytes(),
    ]
    .concat();
    let approver_struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(&[
        &sha256(DOMAIN_TYPEHASH)[..],
        &sha256(DOMAIN_NAME)[..],
        &sha256(params.chain_id.as_bytes())[..],
        params.contract_addr.as_bytes(),
    ].concat());

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &approver_struct_hash[..],
    ]
    .concat();

    let approver_digest_hash = sha256(&digest_hash_input);
    approver_digest_hash
}

pub fn recover(approver_digest_hash: &[u8], signature: &[u8], approver_public_key_bytes: &[u8]) -> StdResult<bool> {
    match secp256k1_verify(approver_digest_hash, signature, approver_public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}
