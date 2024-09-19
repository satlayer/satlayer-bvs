use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{MinimalSlashSignatureResponse, SlashDetailsResponse, ValidatorResponse},
    state::{DELEGATION_MANAGER, OWNER, SLASHER, VALIDATOR, MINIMAL_SLASH_SIGNATURE, SLASH_DETAILS},
    utils::{validate_addresses, SlashDetails, calculate_slash_hash},
};

use common::delegation::{QueryMsg as DelegationManagerQueryMsg, ExecuteMsg as DelegationManagerExecuteMsg};
use common::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use common::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.initial_owner)?;
    let delegation_manager = deps.api.addr_validate(&msg.delegation_manager)?;

    OWNER.save(deps.storage, &owner)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    PAUSED_STATE.save(deps.storage, &msg.initial_paused_status)?;

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
        ExecuteMsg::SubmitSlashRequest { slash_details } => {
            let slasher_addr = deps.api.addr_validate(&slash_details.slasher)?;
            let operator_addr = deps.api.addr_validate(&slash_details.operator)?;
            let slash_validator = validate_addresses(deps.api, &slash_details.slash_validator)?;

            let params = SlashDetails {
                slasher: slasher_addr,
                operator: operator_addr,
                share: slash_details.share,
                slash_signature: slash_details.slash_signature,
                slash_validator,
                reason: slash_details.reason,
                start_time: slash_details.start_time,
                end_time: slash_details.end_time,
                status: slash_details.status
            };

            submit_slash_request(deps, info, env, params)
        }
        ExecuteMsg::ExecuteSlashRequest {
            slash_hash,
            signatures,
        } => execute_slash_request(deps, env, info, slash_hash, signatures),
        ExecuteMsg::CancelSlashRequest { slash_hash } => {
            cancel_slash_request(deps, env, info, slash_hash)
        }
        ExecuteMsg::SetMinimalSlashSignature { minimal_signature } => {
            set_minimal_slash_signature(deps, info, minimal_signature)
        }
        ExecuteMsg::SetSlasher { slasher, value } => {
            let slasher_addr = deps.api.addr_validate(&slasher)?;
            set_slasher(deps, info, slasher_addr, value)
        }
        ExecuteMsg::SetSlasherValidator { validators, values } => {
            let validators = validate_addresses(deps.api, &validators)?;
            set_slash_validator(deps, info, validators, values)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            transfer_ownership(deps, info, new_owner_addr)
        }
        ExecuteMsg::Pause {} => {
            check_pauser(deps.as_ref(), info.clone())?;
            pause(deps, &info).map_err(ContractError::Std)
        }
        ExecuteMsg::Unpause {} => {
            check_unpauser(deps.as_ref(), info.clone())?;
            unpause(deps, &info).map_err(ContractError::Std)
        }
        ExecuteMsg::SetPauser { new_pauser } => {
            only_owner(deps.as_ref(), &info.clone())?;
            let new_pauser_addr = deps.api.addr_validate(&new_pauser)?;
            set_pauser(deps, new_pauser_addr).map_err(ContractError::Std)
        }
        ExecuteMsg::SetUnpauser { new_unpauser } => {
            only_owner(deps.as_ref(), &info.clone())?;
            let new_unpauser_addr = deps.api.addr_validate(&new_unpauser)?;
            set_unpauser(deps, new_unpauser_addr).map_err(ContractError::Std)
        }
    }
}

pub fn submit_slash_request(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    slash_details: SlashDetails,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    if slash_details.share.is_zero() {
        return Err(ContractError::InvalidShare {});
    }

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
    }

    let current_minimal_signature = MINIMAL_SLASH_SIGNATURE.load(deps.storage)?;

    if slash_details.slash_signature < current_minimal_signature {
        return Err(ContractError::InvalidSlashSignature {});
    }

    for validator in slash_details.slash_validator.iter() {
        if !VALIDATOR.load(deps.storage, validator.clone())? {
            return Err(ContractError::Unauthorized {});
        }
    }

    if slash_details.start_time.is_zero() || slash_details.start_time < env.block.time.seconds().into() {
        return Err(ContractError::InvalidStartTime {});
    }
    
    if slash_details.end_time.is_zero() || slash_details.end_time < env.block.time.seconds().into() {
        return Err(ContractError::InvalidEndTime {});
    }

    let slash_hash = calculate_slash_hash(&info.sender, &slash_details);
    let slash_hash_hex = hex::encode(slash_hash.clone());
    SLASH_DETAILS.save(deps.storage, slash_hash_hex.clone(), &slash_details)?;

    let event = Event::new("slash_request_submitted")
        .add_attribute("slash_hash", slash_hash_hex.clone())
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("operator", slash_details.operator.to_string())
        .add_attribute("share", slash_details.share.to_string())
        .add_attribute("start_time", slash_details.start_time.to_string())
        .add_attribute("end_time", slash_details.end_time.to_string())
        .add_attribute("status", slash_details.status.to_string());

    Ok(Response::new().add_event(event))
}

pub fn execute_slash_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    slash_hash: Binary,
    signatures: Vec<Binary>,
) -> Result<Response, ContractError> {
}

pub fn cancel_slash_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    slash_hash: Binary,
) -> Result<Response, ContractError> {
}

pub fn set_minimal_slash_signature(
    deps: DepsMut,
    info: MessageInfo,
    minimal_signature: u64,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    let old_minimal_signature = MINIMAL_SLASH_SIGNATURE.load(deps.storage)?;

    MINIMAL_SLASH_SIGNATURE.save(deps.storage, &minimal_signature)?;

    let event = Event::new("minimal_slash_signature_set")
        .add_attribute("method", "set_minimal_slash_signature")
        .add_attribute("old_minimal_signature", old_minimal_signature.to_string())
        .add_attribute("minimal_signature", minimal_signature.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_slasher(
    deps: DepsMut,
    info: MessageInfo,
    slasher: Addr,
    value: bool,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    SLASHER.save(deps.storage, slasher.clone(), &value)?;

    let event = Event::new("slasher_set")
        .add_attribute("method", "set_slasher")
        .add_attribute("slasher", slasher.to_string())
        .add_attribute("value", value.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_slash_validator(
    deps: DepsMut,
    info: MessageInfo,
    validators: Vec<Addr>,
    values: Vec<bool>,
) -> Result<Response, ContractError> {
    if validators.len() != values.len() {
        return Err(ContractError::InvalidInputLength {});
    }

    only_slasher(deps.as_ref(), &info)?;

    let mut response = Response::new();

    for (validator, value) in validators.iter().zip(values.iter()) {
        VALIDATOR.save(deps.storage, validator.clone(), value)?;

        let event = Event::new("slash_validator_set")
            .add_attribute("method", "set_slash_validator")
            .add_attribute("validator", validator.to_string())
            .add_attribute("value", value.to_string())
            .add_attribute("sender", info.sender.to_string());

        response = response.add_event(event);
    }

    Ok(response)
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
        QueryMsg::GetSlashDetails { slash_hash } => {
            to_json_binary(&query_slash_details(deps, slash_hash)?)
        }
        QueryMsg::IsValidator { validator } => {
            to_json_binary(&query_is_validator(deps, validator)?)
        }
        QueryMsg::GetMinimalSlashSignature {} => {
            to_json_binary(&query_minimal_slash_signature(deps)?)
        }
    }
}

fn query_slash_details(deps: Deps, slash_hash: Binary) -> StdResult<SlashDetailsResponse> {}

fn query_is_validator(deps: Deps, validator: Addr) -> StdResult<ValidatorResponse> {}

fn query_minimal_slash_signature(deps: Deps) -> StdResult<MinimalSlashSignatureResponse> {}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn only_slasher(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let is_slasher = SLASHER.load(deps.storage, info.sender.clone())?;
    if !is_slasher {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "migrate"))
}