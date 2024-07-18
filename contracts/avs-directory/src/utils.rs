use cosmwasm_std::{Addr, Binary, StdResult, QuerierWrapper, WasmQuery, QueryRequest, to_json_binary, Env};
use crate::msg::{IsOperatorRegisteredQueryMsg, IsOperatorRegisteredResponse, VerifySignatureMsg};
use tiny_keccak::{Hasher, Keccak};
use secp256k1::{Message, Secp256k1, PublicKey};
use secp256k1::ecdsa::Signature;

const OPERATOR_AVS_REGISTRATION_TYPEHASH: &[u8] = b"OperatorAVSRegistration(address operator,address avs,bytes32 salt,uint256 expiry)";
const DOMAIN_TYPEHASH: &[u8] = b"EIP712Domain(string name,uint256 chainId,address verifyingContract)";
const DOMAIN_NAME: &[u8] = b"EigenLayer";
const EIP1271_MAGICVALUE: [u8; 4] = [0x16, 0x26, 0xba, 0x7e];

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

pub fn verify_signature<Q: cosmwasm_std::CustomQuery>(
    querier: &QuerierWrapper<Q>,
    operator: &Addr,
    digest_hash: &[u8],
    signature: &[u8],
) -> Result<bool, cosmwasm_std::StdError> {
    let secp = Secp256k1::verification_only();

    // Check if operator is a contract
    let is_contract = is_contract_address(querier, operator)?;

    if is_contract {
        // Implement EIP-1271 verification
        let verify_msg = VerifySignatureMsg {
            hash: digest_hash.to_vec().into(),
            signature: signature.to_vec().into(),
        };

        let res: StdResult<Binary> = querier.query(&WasmQuery::Smart {
            contract_addr: operator.to_string(),
            msg: to_json_binary(&verify_msg)?,
        }.into());

        match res {
            Ok(binary) => {
                let magic_value: [u8; 4] = binary.as_slice().try_into().map_err(|_| cosmwasm_std::StdError::generic_err("Invalid magic value"))?;
                Ok(magic_value == EIP1271_MAGICVALUE)
            }
            Err(_) => Err(cosmwasm_std::StdError::generic_err("EIP-1271 signature verification failed")),
        }
    } else {
        // Implement ECDSA verification
        let message = Message::from_slice(digest_hash).map_err(|_| cosmwasm_std::StdError::generic_err("Invalid message"))?;
        let signature = Signature::from_compact(signature).map_err(|_| cosmwasm_std::StdError::generic_err("Invalid signature"))?;
        let public_key = PublicKey::from_slice(operator.as_bytes()).map_err(|_| cosmwasm_std::StdError::generic_err("Invalid public key"))?;

        match secp.verify_ecdsa(&message, &signature, &public_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
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

fn is_contract_address<Q: cosmwasm_std::CustomQuery>(
    querier: &QuerierWrapper<Q>,
    address: &Addr,
) -> StdResult<bool> {
    let query = WasmQuery::ContractInfo {
        contract_addr: address.to_string(),
    };

    let res: StdResult<cosmwasm_std::ContractInfoResponse> = querier.query(&QueryRequest::Wasm(query));
    match res {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
