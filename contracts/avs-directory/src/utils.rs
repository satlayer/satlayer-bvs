use cosmwasm_std::{Addr, Binary, StdResult, QuerierWrapper, WasmQuery, QueryRequest, to_json_binary, Env};
use crate::msg::{IsOperatorRegisteredQueryMsg, IsOperatorRegisteredResponse};
use tiny_keccak::{Hasher, Keccak};

const OPERATOR_AVS_REGISTRATION_TYPEHASH: &[u8] = b"OperatorAVSRegistration(address operator,address avs,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";

fn keccak256(input: &[u8]) -> Vec<u8> {
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

pub fn verify_signature(
    _operator: &Addr,
    _digest_hash: &[u8],
    _signature: &[u8],
) -> Result<bool, ()> {
    // Implement signature verification logic here.
    // This is a placeholder and needs to be implemented with the appropriate logic.
    Ok(true)
}

pub fn is_operator_registered<Q: cosmwasm_std::CustomQuery>(
    querier: &QuerierWrapper<Q>,
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
