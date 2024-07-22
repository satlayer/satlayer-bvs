use cosmwasm_std::{Addr, Binary, StdResult, QuerierWrapper, WasmQuery, QueryRequest, Env, StdError, to_json_binary};
use crate::msg::{IsOperatorRegisteredQueryMsg, IsOperatorRegisteredResponse};
use tiny_keccak::{Hasher, Keccak};
use secp256k1::{Message, Secp256k1, PublicKey};
use secp256k1::ecdsa::Signature;
use bech32::FromBase32;

const OPERATOR_AVS_REGISTRATION_TYPEHASH: &[u8] = b"OperatorAVSRegistration(address operator,address avs,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

pub fn keccak256(input: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(input);
    hasher.finalize(&mut output);
    output.to_vec()
}

pub fn calculate_digest_hash(
    operator: &Addr,
    avs: &Addr,
    salt: &Binary,
    expiry: u64,
    chain_id: u64,
    env: &Env,
) -> Vec<u8> {
    // Get the current contract address
    let this_contract_address = &env.contract.address;

    // Calculate struct hash
    let struct_hash_input = [
        &keccak256(OPERATOR_AVS_REGISTRATION_TYPEHASH)[..],
        operator.as_bytes(),
        avs.as_bytes(),
        salt.as_slice(),
        &expiry.to_le_bytes(),
    ]
    .concat();
    let struct_hash = keccak256(&struct_hash_input);

    // Calculate domain separator
    let domain_separator = keccak256(&[
        &keccak256(DOMAIN_TYPEHASH)[..],
        &keccak256(DOMAIN_NAME)[..],
        &chain_id.to_le_bytes(),
        this_contract_address.as_bytes(),
    ].concat());

    // Calculate digest hash
    let digest_hash_input = [
        b"\x19\x01",
        &domain_separator[..],
        &struct_hash[..],
    ]
    .concat();

    keccak256(&digest_hash_input)
}

pub fn recover(digest_hash: &[u8], signature: &[u8], operator: &Addr) -> StdResult<bool> {
    let secp = Secp256k1::verification_only();
    let message = Message::from_slice(digest_hash).map_err(|_| StdError::generic_err("Invalid message"))?;
    let signature = Signature::from_compact(signature).map_err(|_| StdError::generic_err("Invalid signature"))?;

    let (_, operator_bytes, _) = bech32::decode(operator.as_str()).map_err(|_| StdError::generic_err("Invalid operator address"))?;
    let operator_bytes = Vec::<u8>::from_base32(&operator_bytes).map_err(|_| StdError::generic_err("Invalid operator address"))?;
    let public_key = PublicKey::from_slice(&operator_bytes).map_err(|_| StdError::generic_err("Invalid public key"))?;

    match secp.verify_ecdsa(&message, &signature, &public_key) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn is_operator_registered<Q: cosmwasm_std::CustomQuery>(
    querier: &QuerierWrapper<Q>,
    _env: &Env,
    delegation_manager_addr: &Addr,
    operator: &Addr,
) -> StdResult<bool> {
    let query_msg = IsOperatorRegisteredQueryMsg {
        operator: operator.clone(),
    };

    let query_request = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: delegation_manager_addr.to_string(),
        msg: to_json_binary(&query_msg)?,
    });

    let res: IsOperatorRegisteredResponse = querier.query(&query_request)?;
    Ok(res.registered)
}
