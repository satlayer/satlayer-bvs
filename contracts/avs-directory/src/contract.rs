use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry},
    state::{OperatorAVSRegistrationStatus, AVSDirectoryStorage},
    utils::{calculate_digest_hash, recover},
    state::OWNER,
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

    if storage.load_status(deps.storage, info.sender.clone(), operator.clone())? == OperatorAVSRegistrationStatus::Registered {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    let salt_binary = Binary::from(operator_signature.salt.as_bytes());

    if storage.is_salt_spent(deps.storage, operator.clone(), salt_binary.clone())? {
        return Err(ContractError::SaltAlreadySpent {});
    }

    let chain_id = &env.block.chain_id;
    println!("chain_id = {}", chain_id); 

    let message_bytes = calculate_digest_hash(
        &operator,
        &info.sender,
        &salt_binary,
        operator_signature.expiry.u64(),
        chain_id,
        &env,
    );

    let signature_bytes = hex::decode(&operator_signature.signature)
    .map_err(|_| ContractError::InvalidSignature {})?;

    if !recover(&message_bytes, &signature_bytes, &operator)? {
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

    fn generate_operator() -> (Addr, SecretKey, Vec<u8>) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let operator_bytes = public_key.serialize();

        let operator_bech32 = bech32::encode("osmo", operator_bytes.to_base32(), Variant::Bech32).unwrap();
        let operator = Addr::unchecked(operator_bech32);
        println!("Operator Address: {:?}", operator);

        (operator, secret_key, operator_bytes.to_vec())
    }
    
    fn mock_signature_with_message(
        operator: &Addr,
        sender: &Addr,
        salt: &str,
        expiry: u64,
        chain_id: &str,
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
    
        let (operator, secret_key, public_key_bytes) = generate_operator();
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
        println!("Public Key Bytes: {:?}", public_key_bytes);
    
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&operator, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);
    
        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
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
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let (operator, secret_key, public_key_bytes) = generate_operator();
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);
        println!("Public Key Bytes: {:?}", public_key_bytes);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&operator, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
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
        let salt = Binary::from("unique_salt".as_bytes());

        let storage = AVSDirectoryStorage::default();
        let is_salt_spent = storage.is_salt_spent(&deps.storage, info.sender.clone(), salt.clone()).unwrap();
        assert!(!is_salt_spent);

        let msg = ExecuteMsg::CancelSalt {
            salt: salt.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        if let Err(ref err) = res {
            println!("Error: {:?}", err);
        }

        assert!(res.is_ok());

        let is_salt_spent = storage.is_salt_spent(&deps.storage, info.sender.clone(), salt.clone()).unwrap();
        assert!(is_salt_spent);

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "cancel_salt");
        assert_eq!(res.attributes[1].key, "operator");
        assert_eq!(res.attributes[1].value, info.sender.to_string());
        assert_eq!(res.attributes[2].key, "salt");
        assert_eq!(res.attributes[2].value, salt.to_base64());
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

        let (operator, secret_key, _public_key_bytes) = generate_operator();
        println!("Operator Address: {:?}", operator);
        println!("Secret Key: {:?}", secret_key);

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let salt = "salt";
        let chain_id = "cosmos-testnet-14002";
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&operator, &info.sender, salt, expiry, chain_id, &contract_addr, &secret_key);

        println!("Operator: {:?}", operator);
        println!("Signature: {:?}", signature);

        let instantiate_msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
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
    
        let (operator, _secret_key, _public_key_bytes) = generate_operator();
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

    fn generate_operator() -> (Addr, SecretKey, Vec<u8>) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let operator_bytes = public_key.serialize();
    
        let operator_bech32 = bech32::encode("osmo", operator_bytes.to_base32(), Variant::Bech32).unwrap();
        let operator = Addr::unchecked(operator_bech32);
        println!("Operator Address: {:?}", operator);
    
        (operator, secret_key, operator_bytes.to_vec())
    }
    
    fn mock_signature_with_message(
        staker: &Addr,
        strategy: &Addr,
        token: &Addr,
        amount: Uint128,
        nonce: u64,
        expiry: u64,
        chain_id: &str,
        contract_addr: &Addr,
        secret_key: &SecretKey,
    ) -> String {
        let params = DigestHashParams {
            staker: staker.clone(),
            strategy: strategy.clone(),
            token: token.clone(),
            amount: amount.u128(),
            nonce,
            expiry,
            chain_id: chain_id.to_string(),
            contract_addr: contract_addr.clone(),
        };
    
        let message_bytes = calculate_digest_hash(&params);
    
        let secp = Secp256k1::new();
        let message = Message::from_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();
        hex::encode(signature_bytes)
    }
    
    #[test]
    fn test_deposit_into_strategy_with_signature() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
        let info_staker = message_info(&Addr::unchecked("staker"), &[]);
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Whitelist a strategy
        let strategy = Addr::unchecked("strategy1");
        let token = Addr::unchecked("token1");
        let amount = Uint128::new(100);
    
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        // Generate operator (staker) and create signature
        let (staker, secret_key, _public_key_bytes) = generate_operator();
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let nonce = 0;
        let chain_id = env.block.chain_id.clone();
        let contract_addr = env.contract.address.clone();
        let signature = mock_signature_with_message(&staker, &strategy, &token, amount, nonce, expiry, &chain_id, &contract_addr, &secret_key);
    
        // Test deposit into strategy with signature
        let msg = ExecuteMsg::DepositIntoStrategyWithSignature {
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
            staker: staker.clone(),
            expiry: Uint64::from(expiry),
            signature,
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit_into_strategy_with_signature");
        assert_eq!(res.attributes[1].key, "strategy");
        assert_eq!(res.attributes[1].value, strategy.to_string());
        assert_eq!(res.attributes[2].key, "amount");
        assert_eq!(res.attributes[2].value, amount.to_string());
        assert_eq!(res.attributes[3].key, "new_shares");
        assert_eq!(res.attributes[3].value, "50"); // Mock value used in the function
    
        // Verify the transfer and deposit messages
        assert_eq!(res.messages.len(), 3);
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = &res.messages[0].msg {
            assert_eq!(contract_addr, &token.to_string());
            let expected_msg = Cw20ExecuteMsg::TransferFrom {
                owner: staker.to_string(),
                recipient: strategy.to_string(),
                amount,
            };
            let actual_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = &res.messages[1].msg {
            assert_eq!(contract_addr, &strategy.to_string());
            let expected_msg = StrategyExecuteMsg::Deposit { amount };
            let actual_msg: StrategyExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        // Verify nonce was incremented
        let stored_nonce = NONCES.load(&deps.storage, &staker).unwrap();
        println!("Stored nonce after deposit: {}", stored_nonce);
        assert_eq!(stored_nonce, 1);
    }
}