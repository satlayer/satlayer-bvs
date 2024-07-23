use cosmwasm_std::{Addr, Binary, StdResult, Env, StdError};
// use crate::msg::{IsOperatorRegisteredQueryMsg, IsOperatorRegisteredResponse};
// use cosmwasm_std::{QuerierWrapper, WasmQuery, QueryRequest, to_json_binary}
use sha2::{Sha256, Digest};
use cosmwasm_crypto::secp256k1_verify;

const OPERATOR_AVS_REGISTRATION_TYPEHASH: &[u8] = b"OperatorAVSRegistration(address operator,address avs,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn sha256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn calculate_digest_hash(
    operator: &Addr,
    avs: &Addr,
    salt: &Binary,
    expiry: u64,
    chain_id: u64,
    env: &Env,
) -> Vec<u8> {
    let struct_hash_input = [
        &sha256(OPERATOR_AVS_REGISTRATION_TYPEHASH)[..],
        operator.as_bytes(),
        avs.as_bytes(),
        salt.as_slice(),
        &expiry.to_le_bytes(),
    ]
    .concat();
    let struct_hash = sha256(&struct_hash_input);

    let domain_separator = sha256(&[
        &sha256(DOMAIN_TYPEHASH)[..],
        &sha256(DOMAIN_NAME)[..],
        &chain_id.to_le_bytes(),
        env.contract.address.as_bytes(),
    ].concat());

    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &struct_hash[..],
    ]
    .concat();

    sha256(&digest_hash_input)
}

pub fn recover(digest_hash: &[u8], signature: &[u8], operator: &Addr) -> StdResult<bool> {
    let public_key_bytes = hex::decode(operator.as_str()).map_err(|_| StdError::generic_err("Invalid operator address"))?;

    match secp256k1_verify(digest_hash, signature, &public_key_bytes) {
        Ok(valid) => Ok(valid),
        Err(_) => Ok(false),
    }
}


// pub fn is_operator_registered<Q: cosmwasm_std::CustomQuery>(
//     querier: &QuerierWrapper<Q>,
//     _env: &Env,
//     delegation_manager_addr: &Addr,
//     operator: &Addr,
// ) -> StdResult<bool> {
//     let query_msg = IsOperatorRegisteredQueryMsg {
//         operator: operator.clone(),
//     };

//     let query_request = QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: delegation_manager_addr.to_string(),
//         msg: to_json_binary(&query_msg)?,
//     });

//     let res: IsOperatorRegisteredResponse = querier.query(&query_request)?;
//     Ok(res.registered)
// }