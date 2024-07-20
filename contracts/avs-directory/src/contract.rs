use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry},
    state::{OperatorAVSRegistrationStatus, AVSDirectoryStorage},
    utils::{calculate_digest_hash, recover, is_operator_registered},
};  
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Addr, 
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
    deps.storage.set(b"owner", owner.as_bytes());

    let storage = AVSDirectoryStorage::default();
    storage.delegation_manager.save(deps.storage, &delegation_manager)?; 

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterOperatorToAVS { operator, signature } => {
            register_operator(deps, env, info, operator, signature)
        }
        ExecuteMsg::DeregisterOperatorFromAVS { operator } => deregister_operator(deps, env, info, operator),
        ExecuteMsg::UpdateAVSMetadataURI { metadata_uri } => update_metadata_uri(info, metadata_uri),
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, env, info, salt),
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: Addr,
    operator_signature: SignatureWithSaltAndExpiry,
) -> Result<Response, ContractError> {
    let current_time: Uint64 = env.block.time.seconds().into();

    if operator_signature.expiry < current_time {
        return Err(ContractError::SignatureExpired {});
    }

    let storage = AVSDirectoryStorage::default();

    if storage.load_status(deps.storage, info.sender.clone(), operator.clone()).is_ok() {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    if storage.is_salt_spent(deps.storage, operator.clone(), operator_signature.salt.clone())? {
        return Err(ContractError::SaltAlreadySpent {});
    }

    // let delegation_manager_addr = storage.delegation_manager.load(deps.storage)?; 
    
    // if !is_operator_registered(&deps.querier, &env, &delegation_manager_addr, &operator)? {
    //     return Err(ContractError::OperatorNotRegistered {});
    // }

    let chain_id: u64 = env.block.chain_id.parse().unwrap_or(0);
    println!("Current chain_id: {}", chain_id); 
    println!("Current contract address: {}", env.contract.address);

    let message_bytes = calculate_digest_hash(
        &operator,
        &info.sender,
        &operator_signature.salt,
        operator_signature.expiry.u64(),
        chain_id,
        &env,
    );
    
    if !recover(&message_bytes, &operator_signature.signature, &operator)? {
        return Err(ContractError::InvalidSignature {});
    }

    storage.save_status(deps.storage, info.sender.clone(), operator.clone(), OperatorAVSRegistrationStatus::Registered)?;

    storage.save_salt(deps.storage, operator.clone(), operator_signature.salt.clone())?;

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
    salt: Binary,
) -> Result<Response, ContractError> {
    let storage = AVSDirectoryStorage::default();

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
    let current_owner = deps.storage.get(b"owner").unwrap();
    
    // Ensure only current owner can transfer ownership
    if current_owner != info.sender.as_bytes() {
        return Err(ContractError::Unauthorized {});
    }

    // Update owner in storage
    deps.storage.set(b"owner", new_owner.as_bytes());

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryOperator { operator } => to_json_binary(&query_operator(deps, operator)?),
    }
}

fn query_operator(deps: Deps, operator: Addr) -> StdResult<OperatorStatusResponse> {
    let storage = AVSDirectoryStorage::default();
    let status = storage.load_status(deps.storage, deps.api.addr_validate("avs")?, operator)?;
    Ok(OperatorStatusResponse { status })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, Storage, Binary, Uint64};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use bech32::{ToBase32, Variant, encode};

    #[test]
    fn test_instantiate() {
        // Arrange
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            chain_id: 1,
            delegation_manager: Addr::unchecked("delegation_manager"),
        };

        // Act
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // Assert
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, "owner");

        let owner = deps.storage.get(b"owner").unwrap();
        assert_eq!(owner, b"owner");

        let storage = AVSDirectoryStorage::default();
        let delegation_manager = storage.delegation_manager.load(&deps.storage).unwrap();
        assert_eq!(delegation_manager, Addr::unchecked("delegation_manager"));
    }

    fn generate_operator() -> (Addr, SecretKey, PublicKey) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let operator_bytes = public_key.serialize();  // Use compressed public key

        // 使用 Bech32 编码生成地址，确保前缀和 CLI 一致
        let bech32_address = encode("osmo", operator_bytes.to_base32(), Variant::Bech32).unwrap();

        let operator = Addr::unchecked(bech32_address);
        (operator, secret_key, public_key)
    } 

    fn mock_signature_with_message(
        operator: &Addr,
        sender: &Addr,
        salt: &str,
        expiry: u64,
        chain_id: u64,
        _contract_addr: &Addr,
        secret_key: &SecretKey,
    ) -> SignatureWithSaltAndExpiry {        
        let env = mock_env();
        let message_bytes = calculate_digest_hash(
            operator,
            sender,
            &Binary::from(salt.as_bytes()),
            expiry,
            chain_id,
            &env,
        );

        println!("Message Hash: {:?}", message_bytes);

        let secp = Secp256k1::new();
        let message = Message::from_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        println!("Signature: {:?}", signature_bytes);

        SignatureWithSaltAndExpiry {
            salt: Binary::from(salt.as_bytes()),
            expiry: Uint64::from(expiry),
            signature: Binary::from(signature_bytes),
        }
    }

    #[test]
    fn test_register_operator() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let (operator, secret_key, _public_key) = generate_operator();
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
        println!("Public Key: {:?}", _public_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id: u64 = 0;
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&operator, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            chain_id: 1,
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
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
        let info = mock_info("creator", &[]);

        let (operator, secret_key, _public_key) = generate_operator();
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
        println!("Public Key: {:?}", _public_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id: u64 = 0;
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&operator, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            chain_id: 1,
            delegation_manager: Addr::unchecked("delegation_manager"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let register_msg = ExecuteMsg::RegisterOperatorToAVS {
            operator: operator.clone(),
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
}