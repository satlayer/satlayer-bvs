use crate::{
    error::ContractError,
    delegation_manager,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry, AVSRegisterParams},
    state::{OperatorAVSRegistrationStatus, OWNER, AVS_OPERATOR_STATUS, OPERATOR_SALT_SPENT, DELEGATION_MANAGER, AVSInfo, AVS_INFO},
    utils::{calculate_digest_hash, recover, OPERATOR_AVS_REGISTRATION_TYPEHASH, DOMAIN_TYPEHASH, DOMAIN_NAME, DigestHashParams, sha256},
};
use delegation_manager::QueryMsg as DelegationManagerQueryMsg;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Addr, Event
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg.initial_owner;
    let delegation_manager = msg.delegation_manager;

    OWNER.save(deps.storage, &owner)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner.to_string())
        .add_attribute("delegation_manager", delegation_manager.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAVS {
            avs_contract,
            state_bank,
            avs_driver,
        } => {
            let params = AVSRegisterParams {
                avs_contract,
                state_bank,
                avs_driver,
            };
            register_avs(deps, params)
        }
        ExecuteMsg::RegisterOperatorToAVS {
            operator,
            public_key,
            contract_addr,
            signature_with_salt_and_expiry,
        } => {
            let public_key_binary = Binary::from_base64(&public_key)?;
            let signature = Binary::from_base64(&signature_with_salt_and_expiry.signature)?;
            let salt = Binary::from_base64(&signature_with_salt_and_expiry.salt)?;

            let signature_with_salt_and_expiry = SignatureWithSaltAndExpiry {
                signature,
                salt,
                expiry: signature_with_salt_and_expiry.expiry,
            };

            register_operator(
                deps,
                env,
                info,
                contract_addr,
                operator,
                public_key_binary,
                signature_with_salt_and_expiry,
            )
        }
        ExecuteMsg::DeregisterOperatorFromAVS { operator } => {
            let operator_addr: Addr = Addr::unchecked(operator);
            deregister_operator(deps, env, info, operator_addr)
        }
        ExecuteMsg::UpdateAVSMetadataURI { metadata_uri } => {
            update_metadata_uri(info, metadata_uri)
        }
        ExecuteMsg::CancelSalt { salt } => {
            let salt_binary = Binary::from_base64(&salt)?;
            cancel_salt(deps, env, info, salt_binary)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr: Addr = Addr::unchecked(new_owner);
            transfer_ownership(deps, info, new_owner_addr)
        }
    }
}

pub fn register_avs(
    deps: DepsMut,
    params: AVSRegisterParams,
) -> Result<Response, ContractError> {
    let input = format!(
        "{}{}{}",
        params.avs_contract, params.state_bank, &params.avs_driver
    );

    let hash_result = sha256(input.as_bytes());

    let avs_hash = hex::encode(hash_result);

    let avs_info = AVSInfo {
        avs_hash: avs_hash.clone(),
        avs_contract: params.avs_contract.clone(),
        state_bank: params.state_bank.clone(),
        avs_driver: params.avs_driver.clone(),
    };

    AVS_INFO.save(deps.storage, avs_hash.clone(), &avs_info)?;

    Ok(Response::new()
        .add_attribute("method", "register_avs")
        .add_attribute("avs_hash", avs_hash))
}

pub fn register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_addr: Addr,
    operator: Addr,
    public_key: Binary,
    operator_signature: SignatureWithSaltAndExpiry,
) -> Result<Response, ContractError> {
    let current_time: Uint64 = env.block.time.seconds().into();

    if operator_signature.expiry < current_time {
        return Err(ContractError::SignatureExpired {});
    }

    let delegation_manager = DELEGATION_MANAGER.load(deps.storage)?;

    let is_operator_registered: bool = deps.querier.query_wasm_smart(
        delegation_manager.clone(),
        &DelegationManagerQueryMsg::IsOperator {
            operator: operator.clone(),
        },
    )?;

    if !is_operator_registered {
        return Err(ContractError::OperatorNotRegistered {});
    }

    let status = AVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorAVSRegistrationStatus::Registered) {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_str: String = operator_signature.salt.to_string();

    let salt_spent = OPERATOR_SALT_SPENT.may_load(deps.storage, (operator.clone(), salt_str.clone()))?;
    if salt_spent.unwrap_or(false) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    let chain_id = &env.block.chain_id;
    println!("chain_id = {}", chain_id);

    let message_bytes = calculate_digest_hash(
        &public_key,
        &info.sender,
        &operator_signature.salt,
        operator_signature.expiry.u64(),
        chain_id,
        &contract_addr,
    );

    if !recover(&message_bytes, &operator_signature.signature, public_key.as_slice())? {
        return Err(ContractError::InvalidSignature {});
    }

    AVS_OPERATOR_STATUS.save(deps.storage, (info.sender.clone(), operator.clone()), &OperatorAVSRegistrationStatus::Registered)?;
    OPERATOR_SALT_SPENT.save(deps.storage, (operator.clone(), salt_str.clone()), &true)?;

    println!("register_operator: operator = {}", operator);

    let event = Event::new("RegisterOperatorToAVS")
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator.to_string())
        .add_attribute("avs", info.sender.to_string());

    Ok(Response::new().add_event(event))    
}

pub fn deregister_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let status = AVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorAVSRegistrationStatus::Registered) {
        AVS_OPERATOR_STATUS.save(deps.storage, (info.sender.clone(), operator.clone()), &OperatorAVSRegistrationStatus::Unregistered)?;

        let event = Event::new("DeregisterOperatorFromAVS")
            .add_attribute("method", "deregister_operator")
            .add_attribute("operator", operator.to_string())
            .add_attribute("avs", info.sender.to_string());

        return Ok(Response::new().add_event(event));
    }

    Err(ContractError::OperatorNotRegistered {})
}

pub fn update_metadata_uri(
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let event = Event::new("UpdateAVSMetadataURI")
        .add_attribute("method", "update_metadata_uri")
        .add_attribute("avs", info.sender.to_string())
        .add_attribute("metadata_uri", metadata_uri.clone());

    Ok(Response::new().add_event(event))
}

pub fn cancel_salt(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    salt: Binary,
) -> Result<Response, ContractError> {
    let salt_spent = OPERATOR_SALT_SPENT.may_load(deps.storage, (info.sender.clone(), salt.to_base64()))?;
    if salt_spent.unwrap_or(false) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    OPERATOR_SALT_SPENT.save(deps.storage, (info.sender.clone(), salt.to_base64()), &true)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_salt")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("salt", salt.to_base64()))
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    let current_owner = OWNER.load(deps.storage)?;

    if current_owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    OWNER.save(deps.storage, &new_owner)?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryOperatorStatus { avs, operator } => {
            to_json_binary(&query_operator_status(deps, avs, operator)?)
        },
        QueryMsg::CalculateDigestHash {
            operator_public_key,
            avs,
            salt,
            expiry,
            chain_id,
            contract_addr,
        } => {
            let public_key_binary = Binary::from_base64(&operator_public_key)?;
            let salt = Binary::from_base64(&salt)?;

            let params = DigestHashParams {
                operator_public_key: public_key_binary,
                avs,
                salt,
                expiry,
                chain_id,
                contract_addr,
            };
            to_json_binary(&query_calculate_digest_hash(deps, env, params)?)
        },
        QueryMsg::IsSaltSpent { operator, salt } => {
            let is_spent = query_is_salt_spent(deps, operator, salt)?;
            to_json_binary(&is_spent)
        },        
        QueryMsg::GetDelegationManager {} => {
            let delegation_manager_addr = query_delegation_manager(deps)?;
            to_json_binary(&delegation_manager_addr.to_string())
        },        
        QueryMsg::GetOwner {} => {
            let owner_addr = query_owner(deps)?;
            to_json_binary(&owner_addr.to_string())
        },
        QueryMsg::GetOperatorAVSRegistrationTypeHash {} => {
            let hash_str = query_operator_avs_registration_typehash(deps)?;
            to_json_binary(&hash_str) 
        },        
        QueryMsg::GetDomainTypeHash {} => {
            let hash_str = query_domain_typehash(deps)?;
            to_json_binary(&hash_str)
        },
        QueryMsg::GetDomainName {} => {
            let name_str = query_domain_name(deps)?;
            to_json_binary(&name_str)
        },
        QueryMsg::GetAVSInfo { avs_hash } => {
            let avs_info = query_avs_info(deps, avs_hash)?;
            to_json_binary(&avs_info)
        }
    }
}

fn query_operator_status(deps: Deps, user_addr: Addr, operator: Addr) -> StdResult<OperatorStatusResponse> {
    let status = AVS_OPERATOR_STATUS.may_load(deps.storage, (user_addr.clone(), operator.clone()))?
        .unwrap_or(OperatorAVSRegistrationStatus::Unregistered);
    Ok(OperatorStatusResponse { status })
}

fn query_calculate_digest_hash(
    _deps: Deps,
    _env: Env,
    params: DigestHashParams,
) -> StdResult<Binary> {
    let digest_hash = calculate_digest_hash(
        &params.operator_public_key,
        &params.avs,
        &params.salt,
        params.expiry.u64(),
        &params.chain_id,
        &params.contract_addr,
    );
    Ok(Binary::new(digest_hash))
}

fn query_is_salt_spent(deps: Deps, operator: Addr, salt: String) -> StdResult<bool> {
    let is_spent = OPERATOR_SALT_SPENT
        .may_load(deps.storage, (operator.clone(), salt.clone()))?
        .unwrap_or(false);

    Ok(is_spent)
}

fn query_delegation_manager(deps: Deps) -> StdResult<Addr> {
    let delegation_manager = DELEGATION_MANAGER.load(deps.storage)?;
    Ok(delegation_manager)
}

fn query_owner(deps: Deps) -> StdResult<Addr> {
    let owner = OWNER.load(deps.storage)?;
    Ok(owner)
}

fn query_operator_avs_registration_typehash(_deps: Deps) -> StdResult<String> {
    let typehash_str = String::from_utf8_lossy(OPERATOR_AVS_REGISTRATION_TYPEHASH).to_string();
    Ok(typehash_str)
}

fn query_domain_typehash(_deps: Deps) -> StdResult<String> {
    let typehash_str = String::from_utf8_lossy(DOMAIN_TYPEHASH).to_string();
    Ok(typehash_str)
}

fn query_domain_name(_deps: Deps) -> StdResult<String> {
    let domain_name_str = String::from_utf8_lossy(DOMAIN_NAME).to_string();
    Ok(domain_name_str)
}

fn query_avs_info(deps: Deps, avs_hash: String) -> StdResult<AVSInfo> {
    let avs_info = AVS_INFO.load(deps.storage, avs_hash.to_string())?;
    Ok(avs_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{Addr, Binary, Uint64, from_json, WasmQuery, SystemResult, ContractResult, SystemError};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use sha2::{Sha256, Digest};
    use ripemd::Ripemd160;
    use bech32::{self, ToBase32, Variant};
    use base64::{engine::general_purpose, Engine as _};
    use crate::msg::ExecuteSignatureWithSaltAndExpiry;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, "owner");

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, Addr::unchecked("owner"));

        let delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(delegation_manager, Addr::unchecked("delegation_manager"));
    }

    fn generate_osmosis_public_key_from_private_key(private_key_hex: &str) -> (Addr, SecretKey, Vec<u8>) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(private_key_hex).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize();
        let sha256_result = Sha256::digest(public_key_bytes);
        let ripemd160_result = Ripemd160::digest(sha256_result);
        let address = bech32::encode("osmo", ripemd160_result.to_base32(), Variant::Bech32).unwrap();
        (Addr::unchecked(address), secret_key, public_key_bytes.to_vec())
    }

    #[test]
    fn test_register_avs() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg: _ } if contract_addr == "delegation_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap())) // Mock operator is registered
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });
    
        let msg = ExecuteMsg::RegisterAVS {
            avs_contract: "avs_contract".to_string(),
            state_bank: "state_bank".to_string(),
            avs_driver: "avs_driver".to_string(),
        };
    
        let result = execute(deps.as_mut(), env, info, msg).unwrap();
    
        let avs_hash = &result.attributes.iter().find(|a| a.key == "avs_hash").unwrap().value;
    
        let avs_info = AVS_INFO.load(&deps.storage, avs_hash.clone()).unwrap();
    
        assert_eq!(result.attributes.len(), 2);
        assert_eq!(result.attributes[0].key, "method");
        assert_eq!(result.attributes[0].value, "register_avs");
        assert_eq!(result.attributes[1].key, "avs_hash");
        assert_eq!(result.attributes[1].value, *avs_hash);
    
        assert_eq!(avs_info.avs_hash, *avs_hash);
        assert_eq!(avs_info.avs_contract, "avs_contract");
        assert_eq!(avs_info.state_bank, "state_bank");
        assert_eq!(avs_info.avs_driver, "avs_driver");
    }    


}