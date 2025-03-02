#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    auth,
    error::ContractError,
    msg::{
        BvsInfoResponse, CalculateDigestHashResponse, DomainNameResponse, DomainTypeHashResponse,
        ExecuteMsg, InstantiateMsg, IsSaltSpentResponse, OperatorBvsRegistrationTypeHashResponse,
        OperatorStatusResponse, QueryMsg, SignatureWithSaltAndExpiry,
    },
    state::{
        BvsInfo, OperatorBvsRegistrationStatus, BVS_INFO, BVS_OPERATOR_STATUS, OPERATOR_SALT_SPENT,
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
use bvs_library::ownership;

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
    bvs_registry::api::set_registry_addr(deps.storage, &deps.api.addr_validate(&msg.registry)?)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::_set_owner(deps.storage, &owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_registry::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::RegisterBvs { bvs_contract } => register_bvs(deps, bvs_contract),
        ExecuteMsg::RegisterOperatorToBvs {
            operator,
            public_key,
            contract_addr,
            signature_with_salt_and_expiry,
        } => {
            let operator = deps.api.addr_validate(&operator)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;

            register_operator_to_bvs(
                deps,
                env,
                info,
                contract_addr,
                operator,
                public_key,
                signature_with_salt_and_expiry,
            )
        }
        ExecuteMsg::DeregisterOperatorFromBvs { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            deregister_operator_from_bvs(deps, env, info, operator)
        }
        ExecuteMsg::UpdateBvsMetadataUri { metadata_uri } => {
            update_bvs_metadata_uri(info, metadata_uri)
        }
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, env, info, salt),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps, &info, &new_owner).map_err(ContractError::Ownership)
        }
        ExecuteMsg::SetRouting { delegation_manager } => {
            let delegation_manager = deps.api.addr_validate(&delegation_manager)?;
            auth::set_routing(deps, info, delegation_manager)
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

    BVS_INFO.save(deps.storage, &bvs_hash, &bvs_info)?;

    Ok(Response::new()
        .add_attribute("method", "register_bvs")
        .add_attribute("bvs_hash", bvs_hash))
}

/// Called by the BVS to register an operator to the BVS
pub fn register_operator_to_bvs(
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

    let delegation_manager = auth::get_delegation_manager(deps.storage)?;

    let is_operator_response: OperatorResponse = deps.querier.query_wasm_smart(
        delegation_manager.clone(),
        &DelegationManagerQueryMsg::IsOperator {
            operator: operator.to_string(),
        },
    )?;

    if !is_operator_response.is_operator {
        return Err(ContractError::OperatorNotRegisteredFromDelegationManager {});
    }

    let status = BVS_OPERATOR_STATUS.may_load(deps.storage, (&info.sender, &operator))?;
    if status == Some(OperatorBvsRegistrationStatus::Registered) {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_str = operator_signature.salt.to_string();

    let salt_spent = OPERATOR_SALT_SPENT.may_load(deps.storage, (&operator, &salt_str))?;
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
        (&info.sender, &operator),
        &OperatorBvsRegistrationStatus::Registered,
    )?;
    OPERATOR_SALT_SPENT.save(deps.storage, (&operator, &salt_str), &true)?;

    let event = Event::new("OperatorBVSRegistrationStatusUpdated")
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator.to_string())
        .add_attribute("bvs", info.sender.to_string())
        .add_attribute("salt_str", salt_str.clone())
        .add_attribute("operator_bvs_registration_status", "REGISTERED");

    Ok(Response::new().add_event(event))
}

pub fn deregister_operator_from_bvs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let status = BVS_OPERATOR_STATUS.may_load(deps.storage, (&info.sender, &operator))?;
    if status == Some(OperatorBvsRegistrationStatus::Registered) {
        BVS_OPERATOR_STATUS.save(
            deps.storage,
            (&info.sender, &operator),
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

pub fn update_bvs_metadata_uri(
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let event = Event::new("BVSMetadataURIUpdated")
        .add_attribute("method", "update_metadata_uri")
        .add_attribute("bvs", info.sender.to_string())
        .add_attribute("metadata_uri", metadata_uri.clone());

    Ok(Response::new().add_event(event))
}

pub fn cancel_salt(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    salt: Binary,
) -> Result<Response, ContractError> {
    let salt_spent =
        OPERATOR_SALT_SPENT.may_load(deps.storage, (&info.sender, &salt.to_base64()))?;
    if salt_spent.unwrap_or(false) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    OPERATOR_SALT_SPENT.save(deps.storage, (&info.sender, &salt.to_base64()), &true)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_salt")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("salt", salt.to_base64()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OperatorStatus { bvs, operator } => {
            let bvs = deps.api.addr_validate(&bvs)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query_operator_status(deps, bvs, operator)?)
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
            let bvs = deps.api.addr_validate(&bvs)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;

            let params = DigestHashParams {
                operator_public_key: public_key_binary,
                bvs,
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
        .may_load(deps.storage, (&user_addr, &operator))?
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
        .may_load(deps.storage, (&operator, &salt))?
        .unwrap_or(false);

    Ok(IsSaltSpentResponse { is_salt_spent })
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
    let bvs_info = BVS_INFO.load(deps.storage, &bvs_hash)?;
    Ok(BvsInfoResponse {
        bvs_hash,
        bvs_contract: bvs_info.bvs_contract,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::set_routing;
    use base64::{engine::general_purpose, Engine as _};
    use bech32::{self, ToBase32, Variant};
    use bvs_library::testing;
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

        let msg = InstantiateMsg {
            owner: owner.clone(),
            registry: deps.api.addr_make("registry").to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.clone());

        let current_owner = ownership::OWNER.load(&deps.storage).unwrap();
        assert_eq!(current_owner, Addr::unchecked(owner));
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner").to_string();
        let owner_info = message_info(&Addr::unchecked(owner.clone()), &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: deps.api.addr_make("registry").to_string(),
        };

        let res = instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.to_string());

        (deps, env, owner_info)
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
        let (mut deps, _, _) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
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

        let bvs_info = BVS_INFO.load(&deps.storage, bvs_hash).unwrap();

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
        let (mut deps, env, info) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager");
        set_routing(deps.as_mut(), info.clone(), delegation_manager.clone()).unwrap();

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
            } if contract_addr == &delegation_manager.to_string() => {
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

        let res = register_operator_to_bvs(
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
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "salt_str");
        assert_eq!(event.attributes[3].value, salt.to_string());
        assert_eq!(event.attributes[4].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[4].value, "REGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (&info.sender, &operator))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (&operator, &salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);
    }

    #[test]
    fn test_deregister_operator() {
        let (mut deps, env, info) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager");
        auth::set_routing(deps.as_mut(), info.clone(), delegation_manager.clone()).unwrap();

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
            } if contract_addr == &delegation_manager.to_string() => {
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

        let res = register_operator_to_bvs(
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

        let res = deregister_operator_from_bvs(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            operator.clone(),
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
        assert_eq!(event.attributes[0].value, "deregister_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[3].value, "UNREGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (&info.sender, &operator))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Unregistered);
    }

    #[test]
    fn test_update_metadata_uri() {
        let (_, _, info) = instantiate_contract();

        let metadata_uri = "http://metadata.uri".to_string();
        let res = update_bvs_metadata_uri(info.clone(), metadata_uri.clone());

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
        let (mut deps, env, info) = instantiate_contract();

        let salt = Binary::from(b"salt");

        let is_salt_spent = OPERATOR_SALT_SPENT
            .may_load(&deps.storage, (&info.sender, &salt.to_string()))
            .unwrap();
        assert!(is_salt_spent.is_none());

        let res = cancel_salt(deps.as_mut(), env.clone(), info.clone(), salt.clone());

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (&info.sender, &salt.to_string()))
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
    fn test_query_operator() {
        let (mut deps, env, info) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager");
        set_routing(deps.as_mut(), info.clone(), delegation_manager.clone()).unwrap();

        let contract_addr: Addr = deps.api.addr_make("contract_addr");

        let expiry = 2722875888;
        let salt = Binary::from(b"salt");
        let operator = testing::Account::default();
        let operator_public_key = operator.public_key.serialize();

        let message_byte = calculate_digest_hash(
            env.clone().block.chain_id,
            &operator_public_key,
            &info.sender,
            &salt,
            expiry,
            &contract_addr,
        );

        let signature = operator.sign(message_byte);
        let signature_bytes = signature.serialize_compact().to_vec();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == &delegation_manager.to_string() => {
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

        let res = register_operator_to_bvs(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract_addr.clone(),
            operator.address.clone(),
            Binary::from_base64(&operator.public_key_base64()).unwrap(),
            SignatureWithSaltAndExpiry {
                signature: Binary::new(signature_bytes),
                salt: salt.clone(),
                expiry,
            },
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorBVSRegistrationStatusUpdated");
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.address.to_string());
        assert_eq!(event.attributes[2].key, "bvs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());
        assert_eq!(event.attributes[3].key, "salt_str");
        assert_eq!(event.attributes[3].value, salt.to_string());
        assert_eq!(event.attributes[4].key, "operator_bvs_registration_status");
        assert_eq!(event.attributes[4].value, "REGISTERED");

        let status = BVS_OPERATOR_STATUS
            .load(&deps.storage, (&info.sender, &operator.address))
            .unwrap();
        assert_eq!(status, OperatorBvsRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (&operator.address, &salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);

        let query_msg = QueryMsg::OperatorStatus {
            bvs: info.sender.to_string(),
            operator: operator.address.to_string(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result: {:?}", query_res);

        assert_eq!(query_res.status, OperatorBvsRegistrationStatus::Registered);
    }

    #[test]
    fn test_query_operator_unregistered() {
        let (deps, env, info) = instantiate_contract();
        let account = testing::Account::new("random_private_key");

        let query_msg = QueryMsg::OperatorStatus {
            bvs: info.sender.to_string(),
            operator: account.address.to_string(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();

        assert_eq!(
            query_res.status,
            OperatorBvsRegistrationStatus::Unregistered
        );
    }

    #[test]
    fn test_query_calculate_digest_hash() {
        let (deps, env, info) = instantiate_contract();
        let account = testing::Account::new("random_private_key");

        let salt = Binary::from(b"salt");
        let contract_addr: Addr = account.address.clone();
        let expiry = 2722875888;

        let query_msg = QueryMsg::CalculateDigestHash {
            operator_public_key: account.public_key_base64().to_string(),
            bvs: info.sender.to_string(),
            salt: salt.to_string(),
            expiry,
            contract_addr: contract_addr.to_string(),
        };

        let response: CalculateDigestHashResponse =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        let expected_digest_hash = calculate_digest_hash(
            env.clone().block.chain_id,
            &account.public_key.serialize(),
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
        let (mut deps, env, info) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager");
        set_routing(deps.as_mut(), info.clone(), delegation_manager.clone()).unwrap();

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
            } if contract_addr == &delegation_manager.to_string() => {
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

        let res = register_operator_to_bvs(
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
    fn test_query_operator_bvs_registration_type_hash() {
        let (deps, env, _info) = instantiate_contract();

        let query_msg = QueryMsg::OperatorBvsRegistrationTypeHash {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let response: OperatorBvsRegistrationTypeHashResponse = from_json(query_res).unwrap();

        let expected_str = String::from_utf8_lossy(OPERATOR_BVS_REGISTRATION_TYPE_HASH).to_string();

        assert_eq!(response.operator_bvs_registration_type_hash, expected_str);
    }
    #[test]
    fn test_query_domain_type_hash() {
        let (deps, env, _info) = instantiate_contract();

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
        let (mut deps, env, info) = instantiate_contract();
        let delegation_manager = deps.api.addr_make("delegation_manager");
        set_routing(deps.as_mut(), info.clone(), delegation_manager.clone()).unwrap();

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
            } if contract_addr == &delegation_manager.to_string() => {
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

        let res = register_operator_to_bvs(
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
        let (mut deps, env, _info) = instantiate_contract();

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
}
