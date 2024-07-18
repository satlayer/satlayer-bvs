use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry},
    state::{OperatorAVSRegistrationStatus, AVSDirectoryStorage},
    utils::{calculate_digest_hash, verify_signature, is_operator_registered},
};  
use babylon_bindings::BabylonQuery;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Addr, 
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<BabylonQuery>,
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
    deps: DepsMut<BabylonQuery>,
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
    deps: DepsMut<BabylonQuery>,
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

    let delegation_manager_addr = storage.delegation_manager.load(deps.storage)?; 
    
    if !is_operator_registered(&deps.querier, &env, &delegation_manager_addr, &operator)? {
        return Err(ContractError::OperatorNotRegistered {});
    }

    // Calculate the digest hash
    let chain_id = env.block.chain_id.parse::<u64>().unwrap_or_default();
    let digest_hash = calculate_digest_hash(
        &operator,
        &info.sender,
        &operator_signature.salt,
        operator_signature.expiry.into(),
        chain_id,
        &env,
    );

    // Check that the signature is valid
    if !verify_signature(&deps.querier, &operator, &digest_hash, &operator_signature.signature).map_err(|_| ContractError::InvalidSignature {})? {
        return Err(ContractError::InvalidSignature {});
    }

    // Set the operator as registered
    storage.save_status(deps.storage, info.sender.clone(), operator.clone(), OperatorAVSRegistrationStatus::Registered)?;

    // Mark the salt as spent
    storage.save_salt(deps.storage, operator.clone(), operator_signature.salt.clone())?;

    Ok(Response::new()
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator.to_string())
        .add_attribute("avs", info.sender.to_string()))
}

pub fn deregister_operator(
    deps: DepsMut<BabylonQuery>,
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
    deps: DepsMut<BabylonQuery>,
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
    deps: DepsMut<BabylonQuery>,
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
