use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry},
    state::{OperatorAVSRegistrationStatus, AVSDirectoryStorage, OWNER},
    utils::{calculate_digest_hash, recover},
};  
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Addr, StdError
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

    let storage = AVSDirectoryStorage::default();
    storage.delegation_manager.save(deps.storage, &delegation_manager)?;

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
        ExecuteMsg::RegisterOperatorToAVS { operator, public_key_hex, signature } => {
            register_operator(deps, env, info, operator, &public_key_hex, signature)
        }
        ExecuteMsg::DeregisterOperatorFromAVS { operator } => deregister_operator(deps, env, info, operator),
        ExecuteMsg::UpdateAVSMetadataURI { metadata_uri } => update_metadata_uri(info, metadata_uri),
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, env, info, &salt),
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: Addr,
    public_key_hex: &str,
    operator_signature: SignatureWithSaltAndExpiry,
) -> Result<Response, ContractError> {
    let current_time: Uint64 = env.block.time.seconds().into();

    if operator_signature.expiry < current_time {
        return Err(ContractError::SignatureExpired {});
    }

    let storage = AVSDirectoryStorage::default();

    if storage.load_status(deps.storage, info.sender.clone(), operator.clone())? == OperatorAVSRegistrationStatus::Registered {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_binary = Binary::from(operator_signature.salt.as_bytes());

    if storage.is_salt_spent(deps.storage, operator.clone(), salt_binary.clone())? {
        return Err(ContractError::SaltAlreadySpent {});
    }

    let chain_id = &env.block.chain_id;
    println!("chain_id = {}", chain_id);

    let public_key_bytes = Binary::from(hex::decode(public_key_hex)
    .map_err(|_| StdError::generic_err("public_key_hex decode failed"))?); 

    let message_bytes = calculate_digest_hash(
        &public_key_bytes,
        &info.sender,
        &salt_binary,
        operator_signature.expiry.u64(),
        chain_id,
        &env,
    );

    let signature_bytes = hex::decode(&operator_signature.signature)
    .map_err(|_| ContractError::InvalidSignature {})?;

    if !recover(&message_bytes, &signature_bytes, public_key_bytes.as_slice())? {
        return Err(ContractError::InvalidSignature {});
    }

    storage.save_status(deps.storage, info.sender.clone(), operator.clone(), OperatorAVSRegistrationStatus::Registered)?;

    storage.save_salt(deps.storage, operator.clone(), salt_binary)?;

    println!("register_operator: operator = {}", operator); 
    Ok(Response::new()
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator.to_string())
        .add_attribute("avs", info.sender.to_string()))
}


pub fn deregister_operator(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: Addr,
) -> Result<Response, ContractError> {
    let storage = AVSDirectoryStorage::default();

    if let Ok(status) = storage.load_status(deps.storage, info.sender.clone(), operator.clone()) {
        if status == OperatorAVSRegistrationStatus::Registered {
            storage.save_status(deps.storage, info.sender.clone(), operator.clone(), OperatorAVSRegistrationStatus::Unregistered)?;

            return Ok(Response::new()
                .add_attribute("method", "deregister_operator")
                .add_attribute("operator", operator.to_string())
                .add_attribute("avs", info.sender.to_string()));
        }
    }

    Err(ContractError::OperatorNotRegistered {})
}

pub fn update_metadata_uri(
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attribute("method", "update_metadata_uri")
        .add_attribute("avs", info.sender.to_string())
        .add_attribute("metadata_uri", metadata_uri))
}

pub fn cancel_salt(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    salt: &str,
) -> Result<Response, ContractError> {
    let storage = AVSDirectoryStorage::default();

    let salt = Binary::from(salt.as_bytes());

    if storage.is_salt_spent(deps.storage, info.sender.clone(), salt.clone())? {
        return Err(ContractError::SaltAlreadySpent {});
    }

    storage.save_salt(deps.storage, info.sender.clone(), salt.clone())?;

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
    // Load current owner from storage
    let current_owner = OWNER.load(deps.storage)?;

    // Ensure only current owner can transfer ownership
    if current_owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Update owner in storage
    OWNER.save(deps.storage, &new_owner)?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryOperator { avs, operator } => {
            to_json_binary(&query_operator(deps, avs, operator)?)
        },
    }
}

fn query_operator(deps: Deps, user_addr: Addr, operator: Addr) -> StdResult<OperatorStatusResponse> {
    let storage = AVSDirectoryStorage::default();
    println!("!!!!User address: {}", user_addr);
    let status = storage.load_status(deps.storage, user_addr.clone(), operator.clone())?;
    println!("Loaded status: {:?}", status);
    Ok(OperatorStatusResponse { status })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{Addr, Binary, Uint64, from_json};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use sha2::{Sha256, Digest};
    use ripemd::Ripemd160;
    use bech32::{self, ToBase32, Variant};

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

        let storage = AVSDirectoryStorage::default();
        let delegation_manager = storage.delegation_manager.load(&deps.storage).unwrap();
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
    
    fn mock_signature_with_message(
        public_key_bytes: &[u8],
        sender: &Addr,
        salt: &str,
        expiry: u64,
        chain_id: &str,
        _contract_addr: &Addr,
        secret_key: &SecretKey,
    ) -> SignatureWithSaltAndExpiry {
        let env = mock_env();
        let message_bytes = calculate_digest_hash(
            &Binary::from(public_key_bytes),
            sender,
            &Binary::from(salt.as_bytes()),
            expiry,
            chain_id,
            &env,
        );
    
        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();
        let signature_hex = hex::encode(signature_bytes);
    
        SignatureWithSaltAndExpiry {
            salt: salt.to_string(),
            expiry: Uint64::from(expiry),
            signature: signature_hex,
        }
    }    
    
    #[test]
    fn test_register_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
    
        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
    
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&public_key_bytes, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);
    
        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key_hex = hex::encode(public_key_bytes);
    
        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key_hex: public_key_hex.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    
        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }
    
        assert!(res.is_ok());
    
        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "register_operator");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, operator.to_string());
        assert_eq!(res.attributes[2].key, "avs");
        assert_eq!(res.attributes[2].value, info.sender.to_string());
    
        let storage = AVSDirectoryStorage::default();
        let status = storage.load_status(&deps.storage, info.sender.clone(), operator.clone()).unwrap();
        assert_eq!(status, OperatorAVSRegistrationStatus::Registered);
    
        let is_salt_spent = storage.is_salt_spent(&deps.storage, operator.clone(), Binary::from(salt.as_bytes())).unwrap();
        assert!(is_salt_spent);
    }
            
    #[test]
    fn test_deregister_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
    
        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, secret_key, public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
    
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&public_key_bytes, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);
    
        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key_hex = hex::encode(public_key_bytes);
    
        let register_msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key_hex: public_key_hex.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), register_msg);
        assert!(res.is_ok());
    
        let deregister_msg = ExecuteMsg::DeregisterOperatorFromAVS {
            operator: operator.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), deregister_msg);
    
        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }
    
        assert!(res.is_ok());
    
        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deregister_operator");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, operator.to_string());
        assert_eq!(res.attributes[2].key, "avs");
        assert_eq!(res.attributes[2].value, info.sender.to_string());
    
        let storage = AVSDirectoryStorage::default();
        let status = storage.load_status(&deps.storage, info.sender.clone(), operator.clone()).unwrap();
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
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "update_metadata_uri");
        assert_eq!(res.attributes[1].key, "avs");
        assert_eq!(res.attributes[1].value, info.sender.to_string());
        assert_eq!(res.attributes[2].key, "metadata_uri");
        assert_eq!(res.attributes[2].value, metadata_uri);
    }

    #[test]
    fn test_cancel_salt() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let salt = "unique_salt".to_string();

        let storage = AVSDirectoryStorage::default();
        let is_salt_spent = storage.is_salt_spent(&deps.storage, info.sender.clone(), Binary::from(salt.as_bytes())).unwrap();
        assert!(!is_salt_spent);

        let msg = ExecuteMsg::CancelSalt { salt: salt.clone() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let is_salt_spent = storage.is_salt_spent(&deps.storage, info.sender.clone(), Binary::from(salt.as_bytes())).unwrap();
        assert!(is_salt_spent);

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "cancel_salt");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, info.sender.to_string());
        assert_eq!(res.attributes[2].key, "salt");
        assert_eq!(res.attributes[2].value, Binary::from(salt.as_bytes()).to_base64());
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
        let (operator, secret_key, public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
    
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&public_key_bytes, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);
    
        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let public_key_hex = hex::encode(public_key_bytes);

        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
            public_key_hex: public_key_hex.clone(),
            signature: signature.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    
        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }
    
        assert!(res.is_ok());
    
        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "register_operator");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, operator.to_string());
        assert_eq!(res.attributes[2].key, "avs");
        assert_eq!(res.attributes[2].value, info.sender.to_string());
    
        let storage = AVSDirectoryStorage::default();
        let status = storage.load_status(&deps.storage, info.sender.clone(), operator.clone()).unwrap();
        assert_eq!(status, OperatorAVSRegistrationStatus::Registered);
    
        let is_salt_spent = storage.is_salt_spent(&deps.storage, operator.clone(), Binary::from(salt.as_bytes())).unwrap();
        assert!(is_salt_spent);
    
        let query_msg = QueryMsg::QueryOperator {
            avs: info.sender.clone(),
            operator: operator.clone(),
        };
        let query_res: OperatorStatusResponse = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result: {:?}", query_res);
    
        assert_eq!(query_res.status, OperatorAVSRegistrationStatus::Registered);
    }    

    #[test]
    fn test_query_operator_unregistered() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
    
        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (operator, _secret_key, _public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);
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
        let query_res: OperatorStatusResponse = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        println!("Query result before registration: {:?}", query_res);
    
        // Check if the status is Unregistered
        assert_eq!(query_res.status, OperatorAVSRegistrationStatus::Unregistered);
    }    
}