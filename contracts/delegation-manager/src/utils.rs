use cosmwasm_std::{Addr, StdResult, Binary, Uint128, to_json_binary};
use sha2::{Sha256, Digest};
use cosmwasm_crypto::secp256k1_verify;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;


const DELEGATION_APPROVAL_TYPEHASH: &[u8] = b"DelegationApproval(address delegationApprover,address staker,address operator,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const STAKER_DELEGATION_TYPEHASH: &[u8] = b"StakerDelegation(address staker,address operator,uint256 nonce,uint256 expiry)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub struct DelegateParams {
    pub staker: Addr,
    pub operator: Addr,
    pub public_key: Binary,
    pub salt: Binary,
}

pub struct ApproverDigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub delegation_approver: Addr,
    pub approver_public_key: Binary,
    pub approver_salt: Binary,
    pub expiry: u64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_delegation_approval_digest_hash(params: &ApproverDigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(DELEGATION_APPROVAL_TYPEHASH)[..],
        params.delegation_approver.as_bytes(),
        params.approver_public_key.as_slice(),
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        params.approver_salt.as_slice(),
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

    sha256(&digest_hash_input)
}

pub struct StakerDigestHashParams {
    pub staker: Addr,
    pub staker_nonce: u128,
    pub operator: Addr,
    pub staker_public_key: Binary,
    pub expiry: u64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_staker_delegation_digest_hash(params: &StakerDigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(STAKER_DELEGATION_TYPEHASH)[..],
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        params.staker_public_key.as_slice(),
        &params.staker_nonce.to_le_bytes(),
        &params.expiry.to_le_bytes()
    ]
    .concat();
    let staker_struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(&[
        &sha256(DOMAIN_TYPEHASH)[..],
        &sha256(DOMAIN_NAME)[..],
        &sha256(params.chain_id.as_bytes())[..],
        params.contract_addr.as_bytes(),
    ].concat());

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &staker_struct_hash[..],
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Withdrawal {
    pub staker: Addr,
    pub delegated_to: Addr,
    pub withdrawer: Addr,
    pub nonce: u128,
    pub start_block: u64,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

pub fn calculate_withdrawal_root(withdrawal: &Withdrawal) -> StdResult<Binary> {
    let mut hasher = Sha256::new();
    hasher.update(to_json_binary(withdrawal)?.as_slice());
    Ok(Binary::from(hasher.finalize().as_slice()))
}