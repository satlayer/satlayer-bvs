use cosmwasm_crypto::secp256k1_verify;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Api, Binary, Env, StdResult, Uint128};
use sha2::{Digest, Sha256};

const DELEGATION_APPROVAL_TYPEHASH: &[u8] = b"DelegationApproval(address delegationApprover,address staker,address operator,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] =
    b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"SatLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

#[cw_serde]
pub struct DelegateParams {
    pub staker: Addr,
    pub operator: Addr,
    pub public_key: Binary,
    pub salt: Binary,
}

#[cw_serde]
pub struct ExecuteDelegateParams {
    pub staker: String,
    pub operator: String,
    pub public_key: String,
    pub salt: String,
}

#[cw_serde]
pub struct ApproverDigestHashParams {
    pub staker: Addr,
    pub operator: Addr,
    pub approver: Addr,
    pub approver_public_key: Binary,
    pub approver_salt: Binary,
    pub expiry: u64,
    pub contract_addr: Addr,
}

#[cw_serde]
pub struct QueryApproverDigestHashParams {
    pub staker: String,
    pub operator: String,
    pub approver: String,
    pub approver_public_key: String,
    pub approver_salt: String,
    pub expiry: u64,
    pub contract_addr: String,
}

pub fn calculate_delegation_approval_digest_hash(
    env: Env,
    params: ApproverDigestHashParams,
) -> Vec<u8> {
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
            env.block.chain_id.as_bytes(),
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

pub fn recover(digest_hash: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> StdResult<bool> {
    match secp256k1_verify(digest_hash, signature, public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}

#[cw_serde]
pub struct Withdrawal {
    pub staker: Addr,
    pub delegated_to: Addr,
    pub withdrawer: Addr,
    pub nonce: Uint128,
    pub start_block: u64,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

pub fn calculate_withdrawal_root(withdrawal: &Withdrawal) -> StdResult<Binary> {
    let mut hasher = Sha256::new();
    hasher.update(to_json_binary(withdrawal)?.as_slice());
    Ok(Binary::from(hasher.finalize().as_slice()))
}

pub fn validate_addresses(api: &dyn Api, strategies: &[String]) -> StdResult<Vec<Addr>> {
    strategies
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}
