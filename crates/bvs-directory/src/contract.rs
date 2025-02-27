#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{
        BvsInfoResponse, CalculateDigestHashResponse, DelegationManagerResponse,
        DomainNameResponse, DomainTypeHashResponse, ExecuteMsg, InstantiateMsg,
        IsSaltSpentResponse, OperatorBvsRegistrationTypeHashResponse, OperatorStatusResponse,
        OwnerResponse, QueryMsg, SignatureWithSaltAndExpiry,
    },
    state::{
        BvsInfo, OperatorBvsRegistrationStatus, BVS_INFO, BVS_OPERATOR_STATUS, DELEGATION_MANAGER,
        OPERATOR_SALT_SPENT, OWNER,
    },
    utils::{
        calculate_digest_hash, recover, sha256, DigestHashParams, DOMAIN_NAME, DOMAIN_TYPE_HASH,
        OPERATOR_BVS_REGISTRATION_TYPE_HASH,
    },
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use bvs_base::delegation::{OperatorResponse, QueryMsg as DelegationManagerQueryMsg};

const CONTRACT_NAME: &str = "BVS Directory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    bvs_registry::api::set_registry(deps.storage, &deps.api.addr_validate(&msg.registry)?)?;

    let owner = deps.api.addr_validate(&msg.initial_owner)?;
    let delegation_manager = deps.api.addr_validate(&msg.delegation_manager)?;

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
    bvs_registry::api::validate_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::RegisterBvs { bvs_contract } => register_bvs(deps, bvs_contract),
        ExecuteMsg::RegisterOperatorToBvs {
            operator,
            public_key,
            contract_addr,
            signature_with_salt_and_expiry,
        } => {
            let operator_addr = Addr::unchecked(operator);
            let contract_addr = Addr::unchecked(contract_addr);

            register_operator(
                deps,
                env,
                info,
                contract_addr,
                operator_addr,
                public_key,
                signature_with_salt_and_expiry,
            )
        }
        ExecuteMsg::DeregisterOperatorFromBvs { operator } => {
            let operator_addr = Addr::unchecked(operator);
            deregister_operator(deps, env, info, operator_addr)
        }
        ExecuteMsg::UpdateBvsMetadataUri { metadata_uri } => {
            update_metadata_uri(info, metadata_uri)
        }
        ExecuteMsg::SetDelegationManager { delegation_manager } => {
            let delegation_manager_addr = deps.api.addr_validate(&delegation_manager)?;
            set_delegation_manager(deps, info, delegation_manager_addr)
        }
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, env, info, salt),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            transfer_ownership(deps, info, new_owner_addr)
        }
    }
}

pub fn register_bvs(deps: DepsMut, bvs_contract: String) -> Result<Response, ContractError> {
    let hash_result = sha256(bvs_contract.as_bytes());

    let bvs_hash = hex::encode(hash_result);

    let bvs_info = BvsInfo {
        bvs_hash: bvs_hash.clone(),
        bvs_contract: bvs_contract.clone(),
    };

    BVS_INFO.save(deps.storage, bvs_hash.clone(), &bvs_info)?;

    Ok(Response::new()
        .add_attribute("method", "register_bvs")
        .add_attribute("bvs_hash", bvs_hash))
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
    if operator_signature.expiry < env.block.time.seconds() {
        return Err(ContractError::SignatureExpired {});
    }

    let delegation_manager = DELEGATION_MANAGER.load(deps.storage)?;

    let is_operator_response: OperatorResponse = deps.querier.query_wasm_smart(
        delegation_manager.clone(),
        &DelegationManagerQueryMsg::IsOperator {
            operator: operator.to_string(),
        },
    )?;

    if !is_operator_response.is_operator {
        return Err(ContractError::OperatorNotRegisteredFromDelegationManager {});
    }

    let status =
        BVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorBvsRegistrationStatus::Registered) {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_str = operator_signature.salt.to_string();

    let salt_spent =
        OPERATOR_SALT_SPENT.may_load(deps.storage, (operator.clone(), salt_str.clone()))?;
    if salt_spent.unwrap_or(false) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    let message_bytes = calculate_digest_hash(
        env.block.chain_id,
        &public_key,
        &info.sender,
        &operator_signature.salt,
        operator_signature.expiry,
        &contract_addr,
    );

    if !recover(
        &message_bytes,
        &operator_signature.signature,
        public_key.as_slice(),
    )? {
        return Err(ContractError::InvalidSignature {});
    }

    BVS_OPERATOR_STATUS.save(
        deps.storage,
        (info.sender.clone(), operator.clone()),
        &OperatorBvsRegistrationStatus::Registered,
    )?;
    OPERATOR_SALT_SPENT.save(deps.storage, (operator.clone(), salt_str.clone()), &true)?;

    let event = Event::new("OperatorBVSRegistrationStatusUpdated")
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator.to_string())
        .add_attribute("bvs", info.sender.to_string())
        .add_attribute("operator_bvs_registration_status", "REGISTERED");

    Ok(Response::new().add_event(event))
}

pub fn deregister_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let status =
        BVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorBvsRegistrationStatus::Registered) {
        BVS_OPERATOR_STATUS.save(
            deps.storage,
            (info.sender.clone(), operator.clone()),
            &OperatorBvsRegistrationStatus::Unregistered,
        )?;

        let event = Event::new("OperatorBVSRegistrationStatusUpdated")
            .add_attribute("method", "deregister_operator")
            .add_attribute("operator", operator.to_string())
            .add_attribute("bvs", info.sender.to_string())
            .add_attribute("operator_bvs_registration_status", "UNREGISTERED");

        return Ok(Response::new().add_event(event));
    }

    Err(ContractError::OperatorNotRegistered {})
}

pub fn update_metadata_uri(
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let event = Event::new("BVSMetadataURIUpdated")
        .add_attribute("method", "update_metadata_uri")
        .add_attribute("bvs", info.sender.to_string())
        .add_attribute("metadata_uri", metadata_uri.clone());

    Ok(Response::new().add_event(event))
}

pub fn set_delegation_manager(
    deps: DepsMut,
    info: MessageInfo,
    delegation_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;

    Ok(Response::new()
        .add_attribute("method", "set_delegation_manager")
        .add_attribute("delegation_manager", delegation_manager.to_string()))
}

pub fn cancel_salt(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    salt: Binary,
) -> Result<Response, ContractError> {
    let salt_spent =
        OPERATOR_SALT_SPENT.may_load(deps.storage, (info.sender.clone(), salt.to_base64()))?;
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
        QueryMsg::OperatorStatus { bvs, operator } => {
            let bvs_addr = Addr::unchecked(bvs);
            let operator_addr = Addr::unchecked(operator);
            to_json_binary(&query_operator_status(deps, bvs_addr, operator_addr)?)
        }
        QueryMsg::CalculateDigestHash {
            operator_public_key,
            bvs,
            salt,
            expiry,
            contract_addr,
        } => {
            let public_key_binary = Binary::from_base64(&operator_public_key)?;
            let salt = Binary::from_base64(&salt)?;
            let bvs_addr = Addr::unchecked(bvs);
            let contract_addr = Addr::unchecked(contract_addr);

            let params = DigestHashParams {
                operator_public_key: public_key_binary,
                bvs: bvs_addr,
                salt,
                expiry,
                contract_addr,
            };
            to_json_binary(&query_calculate_digest_hash(deps, env, params)?)
        }
        QueryMsg::IsSaltSpent { operator, salt } => {
            let operator_addr = Addr::unchecked(operator);
            let is_spent = query_is_salt_spent(deps, operator_addr, salt)?;
            to_json_binary(&is_spent)
        }
        QueryMsg::DelegationManager {} => to_json_binary(&query_delegation_manager(deps)?),
        QueryMsg::Owner {} => to_json_binary(&query_owner(deps)?),
        QueryMsg::OperatorBvsRegistrationTypeHash {} => {
            to_json_binary(&query_operator_bvs_registration_type_hash(deps)?)
        }
        QueryMsg::DomainTypeHash {} => to_json_binary(&query_domain_type_hash(deps)?),
        QueryMsg::DomainName {} => to_json_binary(&query_domain_name(deps)?),
        QueryMsg::BvsInfo { bvs_hash } => to_json_binary(&query_bvs_info(deps, bvs_hash)?),
    }
}

fn query_operator_status(
    deps: Deps,
    user_addr: Addr,
    operator: Addr,
) -> StdResult<OperatorStatusResponse> {
    let status = BVS_OPERATOR_STATUS
        .may_load(deps.storage, (user_addr.clone(), operator.clone()))?
        .unwrap_or(OperatorBvsRegistrationStatus::Unregistered);
    Ok(OperatorStatusResponse { status })
}

fn query_calculate_digest_hash(
    _deps: Deps,
    env: Env,
    params: DigestHashParams,
) -> StdResult<CalculateDigestHashResponse> {
    let digest_hash = calculate_digest_hash(
        env.block.chain_id,
        &params.operator_public_key,
        &params.bvs,
        &params.salt,
        params.expiry,
        &params.contract_addr,
    );

    let digest_hash = Binary::new(digest_hash);
    Ok(CalculateDigestHashResponse { digest_hash })
}

fn query_is_salt_spent(deps: Deps, operator: Addr, salt: String) -> StdResult<IsSaltSpentResponse> {
    let is_salt_spent = OPERATOR_SALT_SPENT
        .may_load(deps.storage, (operator.clone(), salt.clone()))?
        .unwrap_or(false);

    Ok(IsSaltSpentResponse { is_salt_spent })
}

fn query_delegation_manager(deps: Deps) -> StdResult<DelegationManagerResponse> {
    let delegation_addr = DELEGATION_MANAGER.load(deps.storage)?;
    Ok(DelegationManagerResponse { delegation_addr })
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let owner_addr = OWNER.load(deps.storage)?;
    Ok(OwnerResponse { owner_addr })
}

fn query_operator_bvs_registration_type_hash(
    _deps: Deps,
) -> StdResult<OperatorBvsRegistrationTypeHashResponse> {
    let operator_bvs_registration_type_hash =
        String::from_utf8_lossy(OPERATOR_BVS_REGISTRATION_TYPE_HASH).to_string();
    Ok(OperatorBvsRegistrationTypeHashResponse {
        operator_bvs_registration_type_hash,
    })
}

fn query_domain_type_hash(_deps: Deps) -> StdResult<DomainTypeHashResponse> {
    let domain_type_hash = String::from_utf8_lossy(DOMAIN_TYPE_HASH).to_string();
    Ok(DomainTypeHashResponse { domain_type_hash })
}

fn query_domain_name(_deps: Deps) -> StdResult<DomainNameResponse> {
    let domain_name = String::from_utf8_lossy(DOMAIN_NAME).to_string();
    Ok(DomainNameResponse { domain_name })
}

fn query_bvs_info(deps: Deps, bvs_hash: String) -> StdResult<BvsInfoResponse> {
    let bvs_info = BVS_INFO.load(deps.storage, bvs_hash.to_string())?;
    Ok(BvsInfoResponse {
        bvs_hash,
        bvs_contract: bvs_info.bvs_contract,
    })
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use bech32::{self, ToBase32, Variant};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, Addr, Binary, ContractResult, OwnedDeps, SystemError, SystemResult, WasmQuery,
    };
    use ripemd::Ripemd160;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner = deps.api.addr_make("owner").to_string();
        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner.clone(),
            delegation_manager: delegation_manager.clone(),
            registry: deps.api.addr_make("registry").to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.clone());

        let current_owner = OWNER.load(&deps.storage).unwrap();

        assert_eq!(current_owner, Addr::unchecked(owner));

        let current_delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();

        assert_eq!(
            current_delegation_manager,
            Addr::unchecked(delegation_manager)
        );
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        String,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner").to_string();
        let owner_info = message_info(&Addr::unchecked(owner.clone()), &[]);

        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner.to_string(),
            delegation_manager: delegation_manager.to_string(),
            registry: deps.api.addr_make("registry").to_string(),
        };

        let res = instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.to_string());

        (deps, env, owner_info, delegation_manager)
    }

    fn generate_osmosis_public_key_from_private_key(
        private_key_hex: &str,
    ) -> (Addr, SecretKey, Vec<u8>) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(private_key_hex).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize();
        let sha256_result = Sha256::digest(public_key_bytes);
        let ripemd160_result = Ripemd160::digest(sha256_result);
        let address =
            bech32::encode("osmo", ripemd160_result.to_base32(), Variant::Bech32).unwrap();
        (
            Addr::unchecked(address),
            secret_key,
            public_key_bytes.to_vec(),
        )
    }

    #[test]
    fn test_register_bvs() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let result = register_bvs(deps.as_mut(), "bvs_contract".to_string()).unwrap();

        let bvs_hash = &result
            .attributes
            .iter()
            .find(|a| a.key == "bvs_hash")
            .unwrap()
            .value;

        let bvs_info = BVS_INFO.load(&deps.storage, bvs_hash.clone()).unwrap();

        assert_eq!(result.attributes.len(), 2);
        assert_eq!(result.attributes[0].key, "method");
        assert_eq!(result.attributes[0].value, "register_bvs");
        assert_eq!(result.attributes[1].key, "bvs_hash");
        assert_eq!(result.attributes[1].value, *bvs_hash);

        assert_eq!(bvs_info.bvs_hash, *bvs_hash);
        assert_eq!(bvs_info.bvs_contract, "bvs_contract")
    }

    #[test]
    fn test_register_operator() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 2722875888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1wsjhxj3nl8kmrudsxlf7c40yw6crv4pcrk0twvvsp9jmyr675wjqc8t6an");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

        // Update the mock to return the OperatorResponse struct instead of a boolean
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                // Simulate a successful IsOperator response
                let operator_response = OperatorResponse { is_operator: true };
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&operator_response).unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = register_operator(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.clone(),
            Binary::from_base64(public_key_hex).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        );

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorBVSRegistrationStatusUpdated");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[3].value, "REGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (operator.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);
    }

    #[test]
    fn test_deregister_operator() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 2722875888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1wsjhxj3nl8kmrudsxlf7c40yw6crv4pcrk0twvvsp9jmyr675wjqc8t6an");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                let operator_response = OperatorResponse { is_operator: true };
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&operator_response).unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = register_operator(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.clone(),
            Binary::from_base64(public_key_hex).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        );

        assert!(res.is_ok());

        let res = deregister_operator(deps.as_mut(), env.clone(), info.clone(), operator.clone());

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorBVSRegistrationStatusUpdated");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "deregister_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[3].value, "UNREGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Unregistered);
    }

    #[test]
    fn test_update_metadata_uri() {
        let (deps, env, info, _delegation_manager) = instantiate_contract();

        let metadata_uri = "http://metadata.uri".to_string();
        let res = update_metadata_uri(info.clone(), metadata_uri.clone());

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "BVSMetadataURIUpdated");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "update_metadata_uri");
        assert_eq!(event.attributes[1].key, "bvs");
        assert_eq!(event.attributes[1].value, info.sender.to_string());
        assert_eq!(event.attributes[2].key, "metadata_uri");
        assert_eq!(event.attributes[2].value, metadata_uri);
    }

    #[test]
    fn test_cancel_salt() {
        let (mut deps, env, info, _delegation_manager) = instantiate_contract();

        let salt = Binary::from(b"salt");

        let is_salt_spent = OPERATOR_SALT_SPENT
            .may_load(&deps.storage, (info.sender.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent.is_none());

        let res = cancel_salt(deps.as_mut(), env.clone(), info.clone(), salt.clone());

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (info.sender.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "cancel_salt");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, info.sender.to_string());
        assert_eq!(res.attributes[2].key, "salt");
        assert_eq!(res.attributes[2].value, salt.to_string());
    }

    #[test]
    fn test_transfer_ownership() {
        let (mut deps, env, info, _delegation_manager) = instantiate_contract();

        let new_owner = deps.api.addr_make("new_owner");
        let res = transfer_ownership(deps.as_mut(), info.clone(), new_owner.clone());

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "transfer_ownership");
        assert_eq!(res.attributes[1].key, "new_owner");
        assert_eq!(res.attributes[1].value, new_owner.to_string());

        let current_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(current_owner, new_owner);
    }

    #[test]
    fn test_query_operator() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 2722875888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1wsjhxj3nl8kmrudsxlf7c40yw6crv4pcrk0twvvsp9jmyr675wjqc8t6an");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                let operator_response = OperatorResponse { is_operator: true };
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&operator_response).unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = register_operator(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.clone(),
            Binary::from_base64(public_key_hex).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        );

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorBVSRegistrationStatusUpdated");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[3].value, "REGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (operator.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);

        let query_msg = QueryMsg::OperatorStatus {
            bvs: info.sender.to_string(),
            operator: operator.to_string(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result: {:?}", query_res);

        assert_eq!(query_res.status, OperatorBvsRegistrationStatus::Registered);
    }

    #[test]
    fn test_query_operator_unregistered() {
        let (deps, env, info, _delegation_manager) = instantiate_contract();

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, _secret_key, _public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);

        let query_msg = QueryMsg::OperatorStatus {
            bvs: info.sender.to_string(),
            operator: operator.to_string(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result before registration: {:?}", query_res);

        assert_eq!(
            query_res.status,
            OperatorBvsRegistrationStatus::Unregistered
        );
    }

    #[test]
    fn test_query_calculate_digest_hash() {
        let (deps, env, info, _delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (_operator, _secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1wsjhxj3nl8kmrudsxlf7c40yw6crv4pcrk0twvvsp9jmyr675wjqc8t6an");
        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";
        let expiry = 2722875888;

        let query_msg = QueryMsg::CalculateDigestHash {
            operator_public_key: public_key_hex.to_string(),
            bvs: info.sender.to_string(),
            salt: salt.to_string(),
            expiry,
            contract_addr: contract_addr.to_string(),
        };

        let response: CalculateDigestHashResponse =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        let expected_digest_hash = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        assert_eq!(
            response.digest_hash.as_slice(),
            expected_digest_hash.as_slice()
        );

        println!("Digest hash: {:?}", response.digest_hash);
    }

    #[test]
    fn test_query_is_salt_spent() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 2722875888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1wsjhxj3nl8kmrudsxlf7c40yw6crv4pcrk0twvvsp9jmyr675wjqc8t6an");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                let operator_response = OperatorResponse { is_operator: true };
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&operator_response).unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = register_operator(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.clone(),
            Binary::from_base64(public_key_hex).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        );

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let query_msg = QueryMsg::IsSaltSpent {
            operator: operator.to_string(),
            salt: salt.to_string(),
        };

        let response: IsSaltSpentResponse =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert!(response.is_salt_spent);
    }

    #[test]
    fn test_query_delegation_manager() {
        let (deps, env, _info, delegation_manager) = instantiate_contract();

        let query_msg = QueryMsg::DelegationManager {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: DelegationManagerResponse = from_json(query_res).unwrap();

        assert_eq!(
            response.delegation_addr,
            Addr::unchecked(delegation_manager)
        );
    }

    #[test]
    fn test_query_owner() {
        let (deps, env, info, _delegation_manager) = instantiate_contract();

        let query_msg = QueryMsg::Owner {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: OwnerResponse = from_json(query_res).unwrap();

        assert_eq!(response.owner_addr, info.sender);
    }

    #[test]
    fn test_query_operator_bvs_registration_type_hash() {
        let (deps, env, _info, _delegation_manager) = instantiate_contract();

        let query_msg = QueryMsg::OperatorBvsRegistrationTypeHash {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: OperatorBvsRegistrationTypeHashResponse = from_json(query_res).unwrap();

        let expected_str = String::from_utf8_lossy(OPERATOR_BVS_REGISTRATION_TYPE_HASH).to_string();

        assert_eq!(response.operator_bvs_registration_type_hash, expected_str);
    }
    #[test]
    fn test_query_domain_type_hash() {
        let (deps, env, _info, _delegation_manager) = instantiate_contract();

        let query_msg = QueryMsg::DomainTypeHash {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: DomainTypeHashResponse = from_json(query_res).unwrap();

        let expected_str = String::from_utf8_lossy(DOMAIN_TYPE_HASH).to_string();

        assert_eq!(response.domain_type_hash, expected_str);
    }

    #[test]
    fn test_query_domain_name() {
        let deps = mock_dependencies();
        let env = mock_env();

        let query_msg = QueryMsg::DomainName {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: DomainNameResponse = from_json(query_res).unwrap();

        let expected_str = String::from_utf8_lossy(DOMAIN_NAME).to_string();
        assert_eq!(response.domain_name, expected_str);
    }

    #[test]
    fn test_register_operator_to_bvs() {
        let (mut deps, env, info, delegation_manager) = instantiate_contract();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 1722965888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1dhpupjecw7ltsckrckd4saraaf2266aq2dratwyjtwz5p7476yxspgc6td");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager => {
                let operator_response = OperatorResponse { is_operator: true };
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&operator_response).unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = register_operator(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.clone(),
            Binary::from_base64(public_key_hex).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        );

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());
    }

    #[test]
    fn test_recover() {
        let env = mock_env();
        let info = message_info(
            &Addr::unchecked("osmo1l3u09t2x6ey8xcrhc5e48ygauqlxy3facyz34p"),
            &[],
        );

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (_operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let expiry = 1722965888;
        let salt = Binary::from(b"salt");
        let contract_addr: Addr =
            Addr::unchecked("osmo1dhpupjecw7ltsckrckd4saraaf2266aq2dratwyjtwz5p7476yxspgc6td");

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &Binary::from(public_key_bytes.clone()),
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let signature_base64 = general_purpose::STANDARD.encode(&signature_bytes);

        println!("signature5_base64: {:?}", signature_base64);

        let result: Result<bool, cosmwasm_std::StdError> =
            recover(&message_byte, &signature_bytes, &public_key_bytes.clone());

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_query_bvs_info() {
        let (mut deps, env, _info, _delegation_manager) = instantiate_contract();

        let bvs_contract = "bvs_contract".to_string();

        let result = register_bvs(deps.as_mut(), bvs_contract.clone());
        assert!(result.is_ok());

        let hash_result = sha256(bvs_contract.as_bytes());

        let bvs_hash = hex::encode(hash_result);

        let query_msg = QueryMsg::BvsInfo {
            bvs_hash: bvs_hash.clone(),
        };
        let query_response = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let bvs_info: BvsInfo = from_json(query_response).unwrap();

        assert_eq!(bvs_info.bvs_hash, bvs_hash);
        assert_eq!(bvs_info.bvs_contract, bvs_contract.clone())
    }

    #[test]
    fn test_set_delegation_manager() {
        let (mut deps, env, info, _delegation_manager) = instantiate_contract();

        let delegation_manager = deps.api.addr_make("delegation_manager");

        let res = set_delegation_manager(deps.as_mut(), info.clone(), delegation_manager.clone())
            .unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "method" && a.value == "set_delegation_manager"));

        let delegation_manager_addr = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(delegation_manager_addr, delegation_manager);
    }
}
