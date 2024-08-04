use crate::{
    delegation_manager,
    error::ContractError,
    msg::{
        ExecuteMsg, InstantiateMsg, OperatorStatusResponse, QueryMsg, SignatureWithSaltAndExpiry,
    },
    state::{
        OperatorAVSRegistrationStatus, AVS_OPERATOR_STATUS, DELEGATION_MANAGER,
        OPERATOR_SALT_SPENT, OWNER,
    },
    utils::{
        calculate_digest_hash, recover, DOMAIN_NAME, DOMAIN_TYPEHASH,
        OPERATOR_AVS_REGISTRATION_TYPEHASH,
    },
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, Uint64,
};
use cw2::set_contract_version;
use delegation_manager::QueryMsg as DelegationManagerQueryMsg;

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
        ExecuteMsg::RegisterOperatorToAVS {
            operator,
            public_key,
            signature,
        } => register_operator(deps, env, info, operator, public_key, signature),
        ExecuteMsg::DeregisterOperatorFromAVS { operator } => {
            deregister_operator(deps, env, info, operator)
        }
        ExecuteMsg::UpdateAVSMetadataURI { metadata_uri } => {
            update_metadata_uri(info, metadata_uri)
        }
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, env, info, salt),
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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

    let status =
        AVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorAVSRegistrationStatus::Registered) {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_str = operator_signature.salt.to_string();

    let salt_spent =
        OPERATOR_SALT_SPENT.may_load(deps.storage, (operator.clone(), salt_str.clone()))?;
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
        &env,
    );

    if !recover(
        &message_bytes,
        &operator_signature.signature,
        public_key.as_slice(),
    )? {
        return Err(ContractError::InvalidSignature {});
    }

    AVS_OPERATOR_STATUS.save(
        deps.storage,
        (info.sender.clone(), operator.clone()),
        &OperatorAVSRegistrationStatus::Registered,
    )?;
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
    let status =
        AVS_OPERATOR_STATUS.may_load(deps.storage, (info.sender.clone(), operator.clone()))?;
    if status == Some(OperatorAVSRegistrationStatus::Registered) {
        AVS_OPERATOR_STATUS.save(
            deps.storage,
            (info.sender.clone(), operator.clone()),
            &OperatorAVSRegistrationStatus::Unregistered,
        )?;

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
        QueryMsg::QueryOperator { avs, operator } => {
            to_json_binary(&_operator(deps, avs, operator)?)
        }
        QueryMsg::CalculateDigestHash {
            operator_public_key,
            avs,
            salt,
            expiry,
            chain_id,
        } => to_json_binary(&_calculate_digest_hash(
            deps,
            env,
            operator_public_key,
            avs,
            salt,
            expiry.u64(),
            chain_id,
        )?),
        QueryMsg::IsSaltSpent { operator, salt } => _is_salt_spent(deps, operator, salt),
        QueryMsg::GetDelegationManager {} => _delegation_manager(deps),
        QueryMsg::GetOwner {} => _owner(deps),
        QueryMsg::GetOperatorAVSRegistrationTypeHash {} => {
            _operator_avs_registration_typehash(deps)
        }
        QueryMsg::GetDomainTypeHash {} => _domain_typehash(deps),
        QueryMsg::GetDomainName {} => _domain_name(deps),
    }
}

fn _operator(deps: Deps, user_addr: Addr, operator: Addr) -> StdResult<OperatorStatusResponse> {
    let status = AVS_OPERATOR_STATUS
        .may_load(deps.storage, (user_addr.clone(), operator.clone()))?
        .unwrap_or(OperatorAVSRegistrationStatus::Unregistered);
    Ok(OperatorStatusResponse { status })
}

fn _calculate_digest_hash(
    _deps: Deps,
    env: Env,
    operator_public_key: Binary,
    avs: Addr,
    salt: Binary,
    expiry: u64,
    chain_id: String,
) -> StdResult<Binary> {
    let digest_hash = calculate_digest_hash(
        operator_public_key.as_slice(),
        &avs,
        &salt,
        expiry,
        &chain_id,
        &env,
    );
    Ok(Binary::new(digest_hash))
}

fn _is_salt_spent(deps: Deps, operator: Addr, salt: String) -> StdResult<Binary> {
    let is_spent = OPERATOR_SALT_SPENT
        .may_load(deps.storage, (operator.clone(), salt.clone()))?
        .unwrap_or(false);

    to_json_binary(&is_spent)
}

fn _delegation_manager(deps: Deps) -> StdResult<Binary> {
    let delegation_manager = DELEGATION_MANAGER.load(deps.storage)?;
    to_json_binary(&delegation_manager)
}

fn _owner(deps: Deps) -> StdResult<Binary> {
    let owner = OWNER.load(deps.storage)?;
    to_json_binary(&owner)
}

fn _operator_avs_registration_typehash(_deps: Deps) -> StdResult<Binary> {
    to_json_binary(&OPERATOR_AVS_REGISTRATION_TYPEHASH.to_vec())
}

fn _domain_typehash(_deps: Deps) -> StdResult<Binary> {
    to_json_binary(&DOMAIN_TYPEHASH.to_vec())
}

fn _domain_name(_deps: Deps) -> StdResult<Binary> {
    to_json_binary(&DOMAIN_NAME.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bech32::{self, ToBase32, Variant};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        from_json, Addr, Binary, ContractResult, SystemError, SystemResult, Uint64, WasmQuery,
    };
    use ripemd::Ripemd160;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};

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

    fn mock_signature_with_message(
        public_key_bytes: &[u8],
        sender: &Addr,
        salt: Binary,
        expiry: u64,
        chain_id: &str,
        _contract_addr: &Addr,
        secret_key: &SecretKey,
    ) -> SignatureWithSaltAndExpiry {
        let env = mock_env();
        let message_bytes = calculate_digest_hash(
            &Binary::from(public_key_bytes),
            sender,
            &salt,
            expiry,
            chain_id,
            &env,
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        SignatureWithSaltAndExpiry {
            salt,
            expiry: Uint64::from(expiry),
            signature: Binary::from(signature_bytes.as_slice()),
        }
    }

    #[test]
    fn test_register_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = Binary::from(b"salt");
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(
            &public_key_bytes,
            &info.sender,
            salt.clone(),
            expiry,
            chain_id,
            &contract_addr,
            &secret_key,
        );

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key = Binary::from(public_key_bytes.as_slice());

        // Mock the response from the delegation_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == "delegation_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                // Mock operator is registered
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key: public_key.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "RegisterOperatorToAVS");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "avs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());

        let status = AVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorAVSRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (operator.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);
    }

    #[test]
    fn test_deregister_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = Binary::from(b"salt");
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(
            &public_key_bytes,
            &info.sender,
            salt.clone(),
            expiry,
            chain_id,
            &contract_addr,
            &secret_key,
        );

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key = Binary::from(public_key_bytes.as_slice());

        // Mock the response from the delegation_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == "delegation_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                // Mock operator is registered
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        // Register the operator first
        let register_msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key: public_key.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), register_msg);
        assert!(res.is_ok());

        // Deregister the operator
        let deregister_msg = ExecuteMsg::DeregisterOperatorFromAVS {
            operator: operator.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), deregister_msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "DeregisterOperatorFromAVS");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "deregister_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "avs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());

        let status = AVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorAVSRegistrationStatus::Unregistered);
    }

    #[test]
    fn test_update_metadata_uri() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let metadata_uri = "http://metadata.uri".to_string();

        let msg = ExecuteMsg::UpdateAVSMetadataURI {
            metadata_uri: metadata_uri.clone(),
        };
        let res = execute(deps.as_mut(), env, info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "UpdateAVSMetadataURI");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "update_metadata_uri");
        assert_eq!(event.attributes[1].key, "avs");
        assert_eq!(event.attributes[1].value, info.sender.to_string());
        assert_eq!(event.attributes[2].key, "metadata_uri");
        assert_eq!(event.attributes[2].value, metadata_uri);
    }

    #[test]
    fn test_cancel_salt() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let salt = Binary::from(b"salt");

        let is_salt_spent = OPERATOR_SALT_SPENT
            .may_load(&deps.storage, (info.sender.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent.is_none());

        let msg = ExecuteMsg::CancelSalt { salt: salt.clone() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

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
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = Addr::unchecked("owner");
        let info = message_info(&owner, &[]);
        let new_owner = Addr::unchecked("new_owner");

        let instantiate_msg = InstantiateMsg {
            initial_owner: owner.clone(),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

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

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, new_owner);
    }

    #[test]
    fn test_query_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = Binary::from(b"salt");
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(
            &public_key_bytes,
            &info.sender,
            salt.clone(),
            expiry,
            chain_id,
            &contract_addr,
            &secret_key,
        );

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key = Binary::from(public_key_bytes.as_slice());

        // Mock the response from the delegation_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == "delegation_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                // Mock operator is registered
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key: public_key.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.attributes.len(), 0);
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "RegisterOperatorToAVS");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "register_operator");
        assert_eq!(event.attributes[1].key, "operator");
        assert_eq!(event.attributes[1].value, operator.to_string());
        assert_eq!(event.attributes[2].key, "avs");
        assert_eq!(event.attributes[2].value, info.sender.to_string());

        let status = AVS_OPERATOR_STATUS
            .load(&deps.storage, (info.sender.clone(), operator.clone()))
            .unwrap();
        assert_eq!(status, OperatorAVSRegistrationStatus::Registered);

        let is_salt_spent = OPERATOR_SALT_SPENT
            .load(&deps.storage, (operator.clone(), salt.to_string()))
            .unwrap();
        assert!(is_salt_spent);

        let query_msg = QueryMsg::QueryOperator {
            avs: info.sender.clone(),
            operator: operator.clone(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result: {:?}", query_res);

        assert_eq!(query_res.status, OperatorAVSRegistrationStatus::Registered);
    }

    #[test]
    fn test_query_operator_unregistered() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, _secret_key, _public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        // Before RegisterOperatorToAVS, the operator should be unregistered
        let query_msg = QueryMsg::QueryOperator {
            avs: info.sender.clone(),
            operator: operator.clone(),
        };
        let query_res: OperatorStatusResponse =
            from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result before registration: {:?}", query_res);

        // Check if the status is Unregistered
        assert_eq!(
            query_res.status,
            OperatorAVSRegistrationStatus::Unregistered
        );
    }

    #[test]
    fn test_query_calculate_digest_hash() {
        let deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";

        // Create a CalculateDigestHash query message
        let query_msg = QueryMsg::CalculateDigestHash {
            operator_public_key: Binary::from(public_key_bytes.as_slice()),
            avs: info.sender.clone(),
            salt: Binary::from(salt.as_bytes()),
            expiry: Uint64::from(expiry),
            chain_id: chain_id.to_string(),
        };

        // Execute the query
        let query_res: Binary =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        let expected_digest_hash = calculate_digest_hash(
            public_key_bytes.as_slice(),
            &info.sender,
            &Binary::from(salt.as_bytes()),
            expiry,
            chain_id,
            &env,
        );

        assert_eq!(query_res.as_slice(), expected_digest_hash.as_slice());

        println!("Digest hash: {:?}", query_res);
    }

    #[test]
    fn test_query_is_salt_spent() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = Binary::from(b"salt");
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(
            &public_key_bytes,
            &info.sender,
            salt.clone(),
            expiry,
            chain_id,
            &contract_addr,
            &secret_key,
        );

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key = Binary::from(public_key_bytes.as_slice());

        // Mock the response from the delegation_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if contract_addr == "delegation_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                // Mock operator is registered
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key: public_key.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let query_msg = QueryMsg::IsSaltSpent {
            operator: operator.clone(),
            salt: salt.to_string(),
        };

        let query_res: bool =
            from_json(query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap()).unwrap();
        assert!(query_res);

        // Query again to check the updated status
        let query_res: bool = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        assert!(query_res);
    }

    #[test]
    fn test_query_delegation_manager() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        // Instantiate the contract with a delegation manager
        let delegation_manager_address = Addr::unchecked("delegation_manager");
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: delegation_manager_address.clone(),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let query_msg = QueryMsg::GetDelegationManager {};
        let query_res: Addr = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();

        assert_eq!(query_res, delegation_manager_address);
    }

    #[test]
    fn test_query_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner_address = Addr::unchecked("owner");
        let instantiate_msg = InstantiateMsg {
            initial_owner: owner_address.clone(),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let query_msg = QueryMsg::GetOwner {};
        let query_res: Addr = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();

        assert_eq!(query_res, owner_address);
    }

    #[test]
    fn test_query_operator_avs_registration_typehash() {
        let deps = mock_dependencies();
        let env = mock_env();

        let query_msg = QueryMsg::GetOperatorAVSRegistrationTypeHash {};
        let query_res: Vec<u8> =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert_eq!(query_res, OPERATOR_AVS_REGISTRATION_TYPEHASH.to_vec());
    }

    #[test]
    fn test_query_domain_typehash() {
        let deps = mock_dependencies();
        let env = mock_env();

        let query_msg = QueryMsg::GetDomainTypeHash {};
        let query_res: Vec<u8> =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert_eq!(query_res, DOMAIN_TYPEHASH.to_vec());
    }

    #[test]
    fn test_query_domain_name() {
        let deps = mock_dependencies();
        let env = mock_env();

        let query_msg = QueryMsg::GetDomainName {};
        let query_res: Vec<u8> =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert_eq!(query_res, DOMAIN_NAME.to_vec());
    }
}
