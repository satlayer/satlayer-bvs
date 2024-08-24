use cosmwasm_crypto::secp256k1_verify;
use cosmwasm_std::{to_json_binary, Addr, Binary, StdResult, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DELEGATION_APPROVAL_TYPEHASH: &[u8] = b"DelegationApproval(address delegationApprover,address staker,address operator,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] =
    b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const STAKER_DELEGATION_TYPEHASH: &[u8] =
    b"StakerDelegation(address staker,address operator,uint256 nonce,uint256 expiry)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

type StrategyShares = Vec<(Addr, Uint128)>;
pub type StakerShares = Vec<(Addr, StrategyShares)>;

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DelegateParams {
    pub staker: Addr,
    pub operator: Addr,
    pub public_key: Binary,
    pub salt: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteDelegateParams {
    pub staker: Addr,
    pub operator: Addr,
    pub public_key: String,
    pub salt: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApproverDigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub approver: Addr,
    pub approver_public_key: Binary,
    pub approver_salt: Binary,
    pub expiry: Uint64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryApproverDigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub approver: Addr,
    pub approver_public_key: String,
    pub approver_salt: String,
    pub expiry: Uint64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryStakerDigestHashParams {
    pub staker: Addr,
    pub staker_nonce: Uint128,
    pub operator: Addr,
    pub staker_public_key: String,
    pub expiry: Uint64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_delegation_approval_digest_hash(params: ApproverDigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(DELEGATION_APPROVAL_TYPEHASH)[..],
        params.approver.as_bytes(),
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        &params.approver_public_key,
        params.approver_salt.as_slice(),
        &params.expiry.to_le_bytes(),
    ]
    .concat();
    let approver_struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(
        &[
            &sha256(DOMAIN_TYPEHASH)[..],
            &sha256(DOMAIN_NAME)[..],
            params.chain_id.as_bytes(),
            params.contract_addr.as_bytes(),
        ]
        .concat(),
    );

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &approver_struct_hash[..],
    ]
    .concat();

    sha256(&digest_hash_input)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerDigestHashParams {
    pub staker: Addr,
    pub staker_nonce: Uint128,
    pub operator: Addr,
    pub staker_public_key: Binary,
    pub expiry: Uint64,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_staker_delegation_digest_hash(params: StakerDigestHashParams) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(STAKER_DELEGATION_TYPEHASH)[..],
        params.staker.as_bytes(),
        params.operator.as_bytes(),
        params.staker_public_key.as_slice(),
        &params.staker_nonce.to_le_bytes(),
        &params.expiry.to_le_bytes(),
    ]
    .concat();
    let staker_struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(
        &[
            &sha256(DOMAIN_TYPEHASH)[..],
            &sha256(DOMAIN_NAME)[..],
            &sha256(params.chain_id.as_bytes())[..],
            params.contract_addr.as_bytes(),
        ]
        .concat(),
    );

    let digest_hash_input = [b"\x19\x01", &domain_separator[..], &staker_struct_hash[..]].concat();

    sha256(&digest_hash_input)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentStakerDigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub staker_public_key: Binary,
    pub expiry: Uint64,
    pub current_nonce: Uint128,
    pub chain_id: String,
    pub contract_addr: Addr,
}

pub fn calculate_current_staker_delegation_digest_hash(
    params: CurrentStakerDigestHashParams,
) -> StdResult<Binary> {
    let params = StakerDigestHashParams {
        staker: params.staker.clone(),
        staker_nonce: params.current_nonce,
        operator: params.operator.clone(),
        staker_public_key: params.staker_public_key.clone(),
        expiry: params.expiry,
        chain_id: params.chain_id.clone(),
        contract_addr: params.contract_addr.clone(),
    };

    let digest_hash = calculate_staker_delegation_digest_hash(params);
    to_json_binary(&digest_hash)
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
    pub nonce: Uint128,
    pub start_block: Uint64,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

pub fn calculate_withdrawal_root(withdrawal: &Withdrawal) -> StdResult<Binary> {
    let mut hasher = Sha256::new();
    hasher.update(to_json_binary(withdrawal)?.as_slice());
    Ok(Binary::from(hasher.finalize().as_slice()))
}
