use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, OperatorStatusResponse, SignatureWithSaltAndExpiry},
    state::{OperatorStatus, AVSDirectoryStorage},
    utils::{calculate_digest_hash, verify_signature},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterOperator { operator, signature } => {
            register_operator(deps, env, info, operator, signature)
        }
        ExecuteMsg::DeregisterOperator { operator } => deregister_operator(deps, info, operator),
        ExecuteMsg::UpdateMetadataURI { metadata_uri } => update_metadata_uri(info, metadata_uri),
        ExecuteMsg::CancelSalt { salt } => cancel_salt(deps, info, salt),
    }
}

pub fn register_operator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    signature: SignatureWithSaltAndExpiry,
) -> Result<Response, ContractError> {
    let current_time: Uint64 = env.block.time.seconds().into();

    if signature.expiry < current_time {
        return Err(ContractError::SignatureExpired {});
    }

    let mut storage = AVSDirectoryStorage::default();

    if AVSDirectoryStorage::load(&(*deps.storage), operator.clone()).is_ok() {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }

    if storage.salt_spent.contains(&signature.salt) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    let digest_hash = calculate_digest_hash(&operator, &info.sender, &signature.salt, signature.expiry.into());
    if !verify_signature(&operator, &digest_hash, &signature.signature).map_err(|_| ContractError::InvalidSignature {})? {
        return Err(ContractError::InvalidSignature {});
    }

    storage.save(deps.storage, operator.clone(), OperatorStatus::Registered)?;
    storage.salt_spent.insert(signature.salt.clone());

    Ok(Response::new()
        .add_attribute("method", "register_operator")
        .add_attribute("operator", operator)
        .add_attribute("avs", info.sender.to_string()))
}

pub fn deregister_operator(
    deps: DepsMut,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let storage = AVSDirectoryStorage::default();

    if let Ok(status) = AVSDirectoryStorage::load(&(*deps.storage), operator.clone()) {
        if status == OperatorStatus::Registered {
            storage.save(deps.storage, operator.clone(), OperatorStatus::Unregistered)?;

            return Ok(Response::new()
                .add_attribute("method", "deregister_operator")
                .add_attribute("operator", operator)
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
    info: MessageInfo,
    salt: String,
) -> Result<Response, ContractError> {
    let mut storage = AVSDirectoryStorage::default();

    if storage.salt_spent.contains(&salt) {
        return Err(ContractError::SaltAlreadySpent {});
    }

    storage.salt_spent.insert(salt.clone());
    storage.save(deps.storage, info.sender.to_string(), OperatorStatus::Registered)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_salt")
        .add_attribute("operator", info.sender.to_string())
        .add_attribute("salt", salt))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryOperator { operator } => to_json_binary(&query_operator(deps, operator)?),
    }
}

fn query_operator(deps: Deps, operator: String) -> StdResult<OperatorStatusResponse> {
    let status = AVSDirectoryStorage::load(deps.storage, operator.clone()).unwrap_or(OperatorStatus::Unregistered);
    Ok(OperatorStatusResponse { status })
}
