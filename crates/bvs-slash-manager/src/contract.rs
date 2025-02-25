use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{
        CalculateSlashHashResponse, MinimalSlashSignatureResponse, SlashDetailsResponse,
        ValidatorResponse,
    },
    state::{
        Config, CONFIG, DELEGATION_MANAGER, MINIMAL_SLASH_SIGNATURE, OWNER, PENDING_OWNER, SLASHER,
        SLASH_DETAILS, STRATEGY_MANAGER, VALIDATOR, VALIDATOR_PUBKEYS,
    },
    utils::{calculate_slash_hash, recover, validate_addresses, SlashDetails},
};

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use std::collections::HashSet;

use bvs_base::delegation::{
    ExecuteMsg as DelegationManagerExecuteMsg, OperatorResponse, OperatorStakersResponse,
    QueryMsg as DelegationManagerQueryMsg,
};
use bvs_base::pausable::{
    only_when_not_paused, pause_all, pause_bit, unpause_all, unpause_bit, PAUSED_STATE,
};
use bvs_base::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};
use bvs_base::strategy::ExecuteMsg as StrategyManagerExecuteMsg;

const CONTRACT_NAME: &str = "BVS Slash Manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAUSED_EXECUTE_SLASH_REQUEST: u8 = 0;

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
    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;

    OWNER.save(deps.storage, &owner)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    if msg.max_time_in_future == 0 {
        return Err(ContractError::InvalidMaxTimeInFuture {});
    }

    let initial_cfg = Config {
        max_time_in_future: msg.max_time_in_future,
    };
    CONFIG.save(deps.storage, &initial_cfg)?;

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
        ExecuteMsg::SubmitSlashRequest {
            slash_details,
            validators_public_keys,
        } => {
            let slasher_addr = deps.api.addr_validate(&slash_details.slasher)?;
            let operator_addr = deps.api.addr_validate(&slash_details.operator)?;
            let slash_validator = validate_addresses(deps.api, &slash_details.slash_validator)?;

            let validators_public_keys_binary: Result<Vec<Binary>, ContractError> =
                validators_public_keys
                    .iter()
                    .map(|val| {
                        Binary::from_base64(val).map_err(|_| ContractError::InvalidValidator {})
                    })
                    .collect();
            let validators_binary = validators_public_keys_binary?;

            let params = SlashDetails {
                slasher: slasher_addr,
                operator: operator_addr,
                share: slash_details.share,
                slash_signature: slash_details.slash_signature,
                slash_validator,
                reason: slash_details.reason,
                start_time: slash_details.start_time,
                end_time: slash_details.end_time,
                status: slash_details.status,
            };

            submit_slash_request(deps, info, env, params, validators_binary)
        }
        ExecuteMsg::ExecuteSlashRequest {
            slash_hash,
            signatures,
            validators_public_keys,
        } => {
            let signatures_binary: Result<Vec<Binary>, ContractError> = signatures
                .iter()
                .map(|sig| Binary::from_base64(sig).map_err(|_| ContractError::InvalidSignature {}))
                .collect();
            let signatures_binary = signatures_binary?;

            let validators_binary: Result<Vec<Binary>, ContractError> = validators_public_keys
                .iter()
                .map(|val| Binary::from_base64(val).map_err(|_| ContractError::InvalidValidator {}))
                .collect();
            let validators_binary = validators_binary.unwrap();

            execute_slash_request(
                deps,
                env,
                info,
                slash_hash,
                signatures_binary,
                validators_binary,
            )
        }
        ExecuteMsg::CancelSlashRequest { slash_hash } => {
            cancel_slash_request(deps, info, slash_hash)
        }
        ExecuteMsg::SetMaxTimeInFuture { new_value } => {
            set_max_time_in_future(deps, info, new_value)
        }
        ExecuteMsg::SetMinimalSlashSignature { minimal_signature } => {
            set_minimal_slash_signature(deps, info, minimal_signature)
        }
        ExecuteMsg::SetSlasher { slasher, value } => {
            let slasher_addr = deps.api.addr_validate(&slasher)?;
            set_slasher(deps, info, slasher_addr, value)
        }
        ExecuteMsg::SetSlasherValidator {
            validators,
            validator_public_keys,
            values,
        } => {
            let validators = validate_addresses(deps.api, &validators)?;

            let pubkeys_binary: Result<Vec<Binary>, ContractError> = validator_public_keys
                .iter()
                .map(|val| Binary::from_base64(val).map_err(|_| ContractError::InvalidValidator {}))
                .collect();
            let pubkeys_binary = pubkeys_binary?;

            set_slash_validator(deps, info, validators, pubkeys_binary, values)
        }
        ExecuteMsg::SetDelegationManager {
            new_delegation_manager,
        } => {
            let new_delegation_manager_addr = deps.api.addr_validate(&new_delegation_manager)?;
            set_delegation_manager(deps, info, new_delegation_manager_addr)
        }
        ExecuteMsg::SetStrategyManager {
            new_strategy_manager,
        } => {
            let new_strategy_manager_addr = deps.api.addr_validate(&new_strategy_manager)?;
            set_strategy_manager(deps, info, new_strategy_manager_addr)
        }
        ExecuteMsg::TwoStepTransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            two_step_transfer_ownership(deps, info, new_owner_addr)
        }
        ExecuteMsg::AcceptOwnership {} => accept_ownership(deps, info),
        ExecuteMsg::CancelOwnershipTransfer {} => cancel_ownership_transfer(deps, info),
        ExecuteMsg::PauseAll {} => {
            check_pauser(deps.as_ref(), info.clone())?;
            pause_all(deps, &info)?;
            Ok(Response::new().add_attribute("method", "pause_all"))
        }
        ExecuteMsg::UnpauseAll {} => {
            check_unpauser(deps.as_ref(), info.clone())?;
            unpause_all(deps, &info)?;
            Ok(Response::new().add_attribute("method", "unpause_all"))
        }
        ExecuteMsg::PauseBit { index } => {
            check_pauser(deps.as_ref(), info.clone())?;
            pause_bit(deps, &info, index)?;
            Ok(Response::new().add_attribute("method", "pause_bit"))
        }
        ExecuteMsg::UnpauseBit { index } => {
            check_unpauser(deps.as_ref(), info.clone())?;
            unpause_bit(deps, &info, index)?;
            Ok(Response::new().add_attribute("method", "unpause_bit"))
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
    validators_public_keys: Vec<Binary>,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    if slash_details.share.is_zero() {
        return Err(ContractError::InvalidShare {});
    }

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
    }

    if slash_details.slash_validator.len() != validators_public_keys.len()
        || slash_details.slash_validator.len() != slash_details.slash_signature as usize
    {
        return Err(ContractError::InvalidInputLength {});
    }

    let mut unique_validators = std::collections::HashSet::new();
    for validator in slash_details.slash_validator.iter() {
        if !unique_validators.insert(validator) {
            return Err(ContractError::DuplicateValidator {});
        }
    }

    let delegation_manager = DELEGATION_MANAGER.load(deps.storage)?;

    let is_operator_response: OperatorResponse = deps.querier.query_wasm_smart(
        delegation_manager.clone(),
        &DelegationManagerQueryMsg::IsOperator {
            operator: slash_details.operator.to_string(),
        },
    )?;

    if !is_operator_response.is_operator {
        return Err(ContractError::OperatorNotRegistered {});
    }

    let current_minimal_signature = MINIMAL_SLASH_SIGNATURE.may_load(deps.storage)?.unwrap_or(0);

    if slash_details.slash_signature < current_minimal_signature {
        return Err(ContractError::InvalidSlashSignature {});
    }

    for validator in slash_details.slash_validator.iter() {
        if !VALIDATOR.load(deps.storage, validator.clone())? {
            return Err(ContractError::Unauthorized {});
        }
    }

    if slash_details.start_time >= slash_details.end_time {
        return Err(ContractError::InvalidTimeRange {});
    }

    if slash_details.start_time == 0 || slash_details.start_time < env.block.time.seconds() {
        return Err(ContractError::InvalidStartTime {});
    }

    if slash_details.end_time == 0 || slash_details.end_time < env.block.time.seconds() {
        return Err(ContractError::InvalidEndTime {});
    }

    let cfg = CONFIG.load(deps.storage)?;

    let max_allowed_end_time = env.block.time.seconds() + cfg.max_time_in_future;
    if slash_details.end_time > max_allowed_end_time {
        return Err(ContractError::EndTimeTooLarge {});
    }

    let slash_hash = calculate_slash_hash(
        &info.sender,
        &slash_details,
        &env.contract.address,
        &validators_public_keys
            .iter()
            .map(|b| b.to_vec())
            .collect::<Vec<_>>(),
    );
    let slash_hash_hex = hex::encode(slash_hash.clone());

    SLASH_DETAILS.save(deps.storage, slash_hash_hex.clone(), &slash_details)?;

    let mut response = Response::new().add_event(
        Event::new("slash_request_submitted")
            .add_attribute("slash_hash", slash_hash_hex.clone())
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("operator", slash_details.operator.to_string())
            .add_attribute("share", slash_details.share.to_string())
            .add_attribute("start_time", slash_details.start_time.to_string())
            .add_attribute("end_time", slash_details.end_time.to_string())
            .add_attribute("status", slash_details.status.to_string()),
    );

    for validator in slash_details.slash_validator.iter() {
        let validator_event =
            Event::new("slash_validator_checked").add_attribute("validator", validator.to_string());

        response = response.add_event(validator_event);
    }

    Ok(response)
}

pub fn execute_slash_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    slash_hash: String,
    signatures: Vec<Binary>,
    validators_public_keys: Vec<Binary>,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_EXECUTE_SLASH_REQUEST)?;

    only_slasher(deps.as_ref(), &info)?;

    let mut slash_details = SLASH_DETAILS
        .may_load(deps.storage, slash_hash.clone())?
        .ok_or(ContractError::SlashDetailsNotFound {})?;

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
    }

    if signatures.is_empty() {
        return Err(ContractError::SignatureEmpty {});
    }

    if signatures.len() != validators_public_keys.len() {
        return Err(ContractError::InvalidInputLength {});
    }

    for (i, validator_addr) in slash_details.slash_validator.iter().enumerate() {
        let pubkey_submitted = &validators_public_keys[i];

        let is_valid = VALIDATOR.may_load(deps.storage, validator_addr.clone())?;
        if is_valid != Some(true) {
            return Err(ContractError::Unauthorized {});
        }

        let stored_pubkey = VALIDATOR_PUBKEYS.may_load(deps.storage, validator_addr.clone())?;
        let stored_pubkey = match stored_pubkey {
            Some(pk) => pk,
            None => {
                return Err(ContractError::ValidatorNotFound {});
            }
        };

        if stored_pubkey != *pubkey_submitted {
            return Err(ContractError::PubkeyMismatch {});
        }
    }

    let message_bytes = calculate_slash_hash(
        &info.sender,
        &slash_details,
        &env.contract.address,
        &validators_public_keys
            .iter()
            .map(|b| b.to_vec())
            .collect::<Vec<_>>(),
    );

    let mut valid_signatures_count = 0u64;
    let mut used_pubkeys: HashSet<Vec<u8>> = HashSet::new();

    for (signature, public_key) in signatures.iter().zip(validators_public_keys.iter()) {
        if recover(&message_bytes, signature.as_slice(), public_key.as_slice())? {
            if !used_pubkeys.contains(&public_key.to_vec()) {
                used_pubkeys.insert(public_key.to_vec());
                valid_signatures_count += 1;
            }
        } else {
            return Err(ContractError::InvalidSignature {});
        }
    }

    if valid_signatures_count < slash_details.slash_signature {
        return Err(ContractError::SignatureNotEnough {});
    }

    let query_msg = DelegationManagerQueryMsg::GetOperatorStakers {
        operator: slash_details.operator.to_string(),
    };
    let stakers_response: OperatorStakersResponse = deps
        .querier
        .query_wasm_smart(DELEGATION_MANAGER.load(deps.storage)?, &query_msg)?;

    if stakers_response.stakers_and_shares.is_empty() {
        return Err(ContractError::NoStakersUnderOperator {});
    }

    let mut sum_of_shares = Uint128::zero();
    for staker_info in &stakers_response.stakers_and_shares {
        let staker_total_share = staker_info
            .shares_per_strategy
            .iter()
            .fold(Uint128::zero(), |acc, (_, s)| acc + *s);
        sum_of_shares += staker_total_share;
    }
    if sum_of_shares.is_zero() {
        return Err(ContractError::NoStakersUnderOperator {});
    }

    let total_slash_share = slash_details.share;
    if total_slash_share.is_zero() {
        return Err(ContractError::SlashShareTooSmall {});
    }

    let mut messages = vec![];

    for staker_info in &stakers_response.stakers_and_shares {
        let staker_total_share = staker_info
            .shares_per_strategy
            .iter()
            .fold(Uint128::zero(), |acc, (_, s)| acc + *s);

        for (strategy_addr, strategy_share) in &staker_info.shares_per_strategy {
            if staker_total_share.is_zero() {
                break;
            }

            // Formula: slash_in_strat = staker_per_strategy_share * (total_slash_share / sum_of_shares)
            let slash_in_strat = strategy_share
                .checked_multiply_ratio(total_slash_share, sum_of_shares)
                .map_err(|_| ContractError::Overflow {})?;

            if slash_in_strat.is_zero() {
                continue;
            }

            if slash_in_strat > *strategy_share {
                return Err(ContractError::InsufficientSharesForStaker {
                    staker: staker_info.staker.to_string(),
                });
            }

            let dec_msg = DelegationManagerExecuteMsg::DecreaseDelegatedShares {
                staker: staker_info.staker.to_string(),
                strategy: strategy_addr.to_string(),
                shares: slash_in_strat,
            };
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: DELEGATION_MANAGER.load(deps.storage)?.to_string(),
                msg: to_json_binary(&dec_msg)?,
                funds: vec![],
            }));

            let remove_msg = StrategyManagerExecuteMsg::RemoveShares {
                staker: staker_info.staker.to_string(),
                strategy: strategy_addr.to_string(),
                shares: slash_in_strat,
            };
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: STRATEGY_MANAGER.load(deps.storage)?.to_string(),
                msg: to_json_binary(&remove_msg)?,
                funds: vec![],
            }));
        }
    }

    slash_details.status = false;
    SLASH_DETAILS.save(deps.storage, slash_hash.clone(), &slash_details)?;

    let slash_event = Event::new("slash_executed_weighted")
        .add_attribute("action", "execute_slash_request")
        .add_attribute("slash_hash", slash_hash)
        .add_attribute("operator", slash_details.operator.to_string())
        .add_attribute("total_slash_share", slash_details.share.to_string());

    Ok(Response::new()
        .add_submessages(messages)
        .add_event(slash_event))
}

pub fn cancel_slash_request(
    deps: DepsMut,
    info: MessageInfo,
    slash_hash: String,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    let mut slash_details = match SLASH_DETAILS.may_load(deps.storage, slash_hash.clone())? {
        Some(details) => details,
        None => return Err(ContractError::SlashDetailsNotFound {}),
    };

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
    }

    slash_details.status = false;
    SLASH_DETAILS.save(deps.storage, slash_hash.clone(), &slash_details)?;

    let event = Event::new("cancel_slash_request")
        .add_attribute("method", "cancel_slash_request")
        .add_attribute("slash_hash", slash_hash)
        .add_attribute("slash_details_status", slash_details.status.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_max_time_in_future(
    deps: DepsMut,
    info: MessageInfo,
    new_value: u64,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    if new_value == 0 {
        return Err(ContractError::InvalidMaxTimeInFuture {});
    }

    CONFIG.update(deps.storage, |mut c| -> Result<_, ContractError> {
        c.max_time_in_future = new_value;
        Ok(c)
    })?;

    let event = Event::new("max_time_in_future_set")
        .add_attribute("method", "set_max_time_in_future")
        .add_attribute("new_value", new_value.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_minimal_slash_signature(
    deps: DepsMut,
    info: MessageInfo,
    minimal_signature: u64,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    let old_minimal_signature = MINIMAL_SLASH_SIGNATURE.may_load(deps.storage)?.unwrap_or(0);

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
    validator_public_keys: Vec<Binary>,
    values: Vec<bool>,
) -> Result<Response, ContractError> {
    if validators.len() != values.len() || validators.len() != validator_public_keys.len() {
        return Err(ContractError::InvalidInputLength {});
    }

    only_slasher(deps.as_ref(), &info)?;

    let mut response = Response::new();

    for ((validator_addr, &value), pubkey) in validators
        .iter()
        .zip(values.iter())
        .zip(validator_public_keys.iter())
    {
        if value {
            VALIDATOR.save(deps.storage, validator_addr.clone(), &true)?;
            VALIDATOR_PUBKEYS.save(deps.storage, validator_addr.clone(), &pubkey.clone())?;
        } else {
            VALIDATOR.remove(deps.storage, validator_addr.clone());
            VALIDATOR_PUBKEYS.remove(deps.storage, validator_addr.clone());
        }

        let event = Event::new("slash_validator_set")
            .add_attribute("method", "set_slash_validator")
            .add_attribute("validator", validator_addr.to_string())
            .add_attribute("value", value.to_string())
            .add_attribute("sender", info.sender.to_string());

        response = response.add_event(event);
    }

    Ok(response)
}

pub fn set_delegation_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_delegation_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    DELEGATION_MANAGER.save(deps.storage, &new_delegation_manager.clone())?;

    let event = Event::new("delegation_manager_set")
        .add_attribute("method", "set_delegation_manager")
        .add_attribute("new_delegation_manager", new_delegation_manager.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_strategy_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    STRATEGY_MANAGER.save(deps.storage, &new_strategy_manager)?;

    let event = Event::new("strategy_manager_set")
        .add_attribute("method", "set_strategy_manager")
        .add_attribute("new_strategy_manager", new_strategy_manager.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(Response::new().add_event(event))
}

pub fn two_step_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    PENDING_OWNER.save(deps.storage, &Some(new_owner.clone()))?;

    let resp = Response::new()
        .add_attribute("action", "two_step_transfer_ownership")
        .add_attribute("old_owner", info.sender.to_string())
        .add_attribute("pending_owner", new_owner.to_string());

    Ok(resp)
}

pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let pending_owner = PENDING_OWNER.load(deps.storage)?;

    let pending_owner_addr = match pending_owner {
        Some(addr) => addr,
        None => return Err(ContractError::NoPendingOwner {}),
    };

    if info.sender != pending_owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    OWNER.save(deps.storage, &info.sender)?;
    PENDING_OWNER.save(deps.storage, &None)?;

    let resp = Response::new()
        .add_attribute("action", "accept_ownership")
        .add_attribute("new_owner", info.sender.to_string());

    Ok(resp)
}

pub fn cancel_ownership_transfer(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    PENDING_OWNER.save(deps.storage, &None)?;

    let resp = Response::new().add_attribute("action", "cancel_ownership_transfer");

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSlashDetails { slash_hash } => {
            to_json_binary(&query_slash_details(deps, slash_hash)?)
        }
        QueryMsg::IsValidator { validator } => {
            let validator_addr = deps.api.addr_validate(&validator)?;
            to_json_binary(&query_is_validator(deps, validator_addr)?)
        }
        QueryMsg::GetMinimalSlashSignature {} => {
            to_json_binary(&query_minimal_slash_signature(deps)?)
        }
        QueryMsg::CalculateSlashHash {
            sender,
            slash_details,
            validators_public_keys,
        } => {
            let sender_addr = deps.api.addr_validate(&sender)?;
            let slasher_addr = deps.api.addr_validate(&slash_details.slasher)?;
            let operator_addr = deps.api.addr_validate(&slash_details.operator)?;
            let slash_validator = validate_addresses(deps.api, &slash_details.slash_validator)?;

            let validators_public_keys_binary: Result<Vec<Binary>, ContractError> =
                validators_public_keys
                    .iter()
                    .map(|val| {
                        Binary::from_base64(val).map_err(|_| ContractError::InvalidValidator {})
                    })
                    .collect();
            let validators_binary = validators_public_keys_binary.unwrap();

            let params = SlashDetails {
                slasher: slasher_addr,
                operator: operator_addr,
                share: slash_details.share,
                slash_signature: slash_details.slash_signature,
                slash_validator,
                reason: slash_details.reason,
                start_time: slash_details.start_time,
                end_time: slash_details.end_time,
                status: slash_details.status,
            };

            let res = query_calculate_slash_hash(env, sender_addr, params, validators_binary)?;

            to_json_binary(&res)
        }
    }
}

fn query_slash_details(deps: Deps, slash_hash: String) -> StdResult<SlashDetailsResponse> {
    let slash_details = SLASH_DETAILS.load(deps.storage, slash_hash)?;
    Ok(SlashDetailsResponse { slash_details })
}

fn query_is_validator(deps: Deps, validator: Addr) -> StdResult<ValidatorResponse> {
    let is_validator = VALIDATOR
        .may_load(deps.storage, validator)?
        .unwrap_or(false);
    Ok(ValidatorResponse { is_validator })
}

fn query_minimal_slash_signature(deps: Deps) -> StdResult<MinimalSlashSignatureResponse> {
    let minimal_slash_signature = MINIMAL_SLASH_SIGNATURE.load(deps.storage)?;
    Ok(MinimalSlashSignatureResponse {
        minimal_slash_signature,
    })
}

fn query_calculate_slash_hash(
    env: Env,
    sender: Addr,
    slash_details: SlashDetails,
    validators_public_keys: Vec<Binary>,
) -> StdResult<CalculateSlashHashResponse> {
    let message_bytes = calculate_slash_hash(
        &sender,
        &slash_details,
        &env.contract.address,
        &validators_public_keys
            .iter()
            .map(|b| b.to_vec())
            .collect::<Vec<_>>(),
    );

    Ok(CalculateSlashHashResponse { message_bytes })
}

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ExecuteSlashDetails;
    use base64::{engine::general_purpose, Engine as _};
    use bech32::{self, ToBase32, Variant};
    use bvs_base::delegation::StakerShares;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, from_json, ContractResult, CosmosMsg, OwnedDeps, SystemError, SystemResult, WasmQuery,
    };
    use ripemd::Ripemd160;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("creator");
        let delegation_manager = deps.api.addr_make("delegation_manager");
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let info = message_info(&initial_owner, &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            delegation_manager: delegation_manager.to_string(),
            strategy_manager: strategy_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            max_time_in_future: 1_000_000,
            initial_paused_status: 0,
        };

        let response = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(
            response.attributes,
            vec![
                attr("method", "instantiate"),
                attr("owner", info.sender.to_string()),
                attr("delegation_manager", delegation_manager.to_string()),
            ]
        );

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, deps.api.addr_make("creator"));

        let delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(delegation_manager, deps.api.addr_make("delegation_manager"));

        let paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_state, 0);

        let pauser_addr = deps.api.addr_make("pauser");
        let unpauser_addr = deps.api.addr_make("unpauser");
        assert!(set_pauser(deps.as_mut(), pauser_addr).is_ok());
        assert!(set_unpauser(deps.as_mut(), unpauser_addr).is_ok());

        let invalid_info = message_info(&deps.api.addr_make("invalid_creator"), &[]);

        let invalid_msg = InstantiateMsg {
            initial_owner: "invalid_address".to_string(),
            delegation_manager: delegation_manager.to_string(),
            strategy_manager: strategy_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            max_time_in_future: 1_000_000,
            initial_paused_status: 0,
        };

        let result = instantiate(
            deps.as_mut(),
            env.clone(),
            invalid_info.clone(),
            invalid_msg,
        );
        assert!(result.is_err());
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        Addr,
        Addr,
        Addr,
        Addr,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("creator");
        let delegation_manager = deps.api.addr_make("delegation_manager");
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let info = message_info(&initial_owner, &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            delegation_manager: delegation_manager.to_string(),
            strategy_manager: strategy_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            max_time_in_future: 1_000_000,
            initial_paused_status: 0,
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        (
            deps,
            env,
            info,
            delegation_manager,
            initial_owner,
            pauser,
            unpauser,
        )
    }

    #[test]
    fn test_set_delegation_manager() {
        let (mut deps, env, info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");

        let execute_msg = ExecuteMsg::SetDelegationManager {
            new_delegation_manager: new_delegation_manager.to_string(),
        };
        let response = execute(deps.as_mut(), env.clone(), info.clone(), execute_msg).unwrap();

        assert_eq!(response.events.len(), 1);
        let event = &response.events[0];

        assert_eq!(event.ty, "delegation_manager_set");

        assert_eq!(event.attributes.len(), 3);

        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_delegation_manager");

        assert_eq!(event.attributes[1].key, "new_delegation_manager");
        assert_eq!(
            event.attributes[1].value,
            new_delegation_manager.to_string()
        );

        let delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(delegation_manager, new_delegation_manager);
    }

    #[test]
    fn test_set_slasher() {
        let (mut deps, env, info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let new_slasher = deps.api.addr_make("new_slasher");

        let execute_msg = ExecuteMsg::SetSlasher {
            slasher: new_slasher.to_string(),
            value: true,
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), execute_msg).unwrap();

        assert_eq!(response.events.len(), 1);
        let event = &response.events[0];

        assert_eq!(event.ty, "slasher_set");

        assert_eq!(event.attributes.len(), 4);

        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_slasher");

        assert_eq!(event.attributes[1].key, "slasher");
        assert_eq!(event.attributes[1].value, new_slasher.to_string());

        assert_eq!(event.attributes[2].key, "value");
        assert_eq!(event.attributes[2].value, "true");

        let is_slasher = SLASHER.load(&deps.storage, new_slasher.clone()).unwrap();
        assert_eq!(is_slasher, true);
    }

    #[test]
    fn test_set_minimal_slash_signature() {
        let (mut deps, env, _info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);
        let new_minimal_signature = 10;

        let execute_msg = ExecuteMsg::SetMinimalSlashSignature {
            minimal_signature: new_minimal_signature,
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        )
        .unwrap();

        assert_eq!(response.events.len(), 1);
        let event = &response.events[0];
        assert_eq!(event.ty, "minimal_slash_signature_set");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_minimal_slash_signature");
        assert_eq!(event.attributes[1].key, "old_minimal_signature");
        assert_eq!(event.attributes[1].value, "0");
        assert_eq!(event.attributes[2].key, "minimal_signature");
        assert_eq!(event.attributes[2].value, new_minimal_signature.to_string());

        let stored_signature = MINIMAL_SLASH_SIGNATURE.load(&deps.storage).unwrap();
        assert_eq!(stored_signature, new_minimal_signature);
    }

    #[test]
    fn test_set_slash_validator() {
        let (mut deps, env, _info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");

        let private_key_hex1 = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator1, _, _) = generate_osmosis_public_key_from_private_key(private_key_hex1);

        let private_key_hex2 = "e5dbc50cb04311a2a5c3c0e0258d396e962f64c6c2f758458ffb677d7f0c0e94";
        let (validator2, _, _) = generate_osmosis_public_key_from_private_key(private_key_hex2);

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        let validators = vec![validator1, validator2];
        let values = vec![true, true];

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "AggozHu/LCQC7T7WATaTNHOm8XTOTKNzVz+s8SKoZm85".to_string(),
        ];

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: validators.iter().map(|v| v.to_string()).collect(),
            validator_public_keys: validators_public_keys,
            values: values.clone(),
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        )
        .unwrap();

        assert_eq!(response.events.len(), 2);

        for (i, validator) in validators.iter().enumerate() {
            let event = &response.events[i];

            assert_eq!(event.ty, "slash_validator_set");
            assert_eq!(event.attributes.len(), 4);

            assert_eq!(event.attributes[0].key, "method");
            assert_eq!(event.attributes[0].value, "set_slash_validator");

            assert_eq!(event.attributes[1].key, "validator");
            assert_eq!(event.attributes[1].value, validator.to_string());

            assert_eq!(event.attributes[2].key, "value");
            assert_eq!(event.attributes[2].value, values[i].to_string());

            assert_eq!(event.attributes[3].key, "sender");
            assert_eq!(event.attributes[3].value, slasher_addr.to_string());
        }

        for (validator, value) in validators.iter().zip(values.iter()) {
            let stored_value = VALIDATOR.load(&deps.storage, validator.clone()).unwrap();
            assert_eq!(stored_value, *value);
        }
    }

    #[test]
    fn test_two_step_transfer_ownership() {
        let (mut deps, env, info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let new_owner_addr = deps.api.addr_make("new_owner");
        let msg = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: new_owner_addr.to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0], ("action", "two_step_transfer_ownership"));
        assert_eq!(res.attributes[1], ("old_owner", info.sender.to_string()));
        assert_eq!(
            res.attributes[2],
            ("pending_owner", new_owner_addr.to_string())
        );

        let cancel_msg = ExecuteMsg::CancelOwnershipTransfer {};
        let cancel_res = execute(deps.as_mut(), env.clone(), info.clone(), cancel_msg).unwrap();

        assert_eq!(cancel_res.attributes.len(), 1);
        assert_eq!(
            cancel_res.attributes[0],
            ("action", "cancel_ownership_transfer")
        );

        let msg2 = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: new_owner_addr.to_string(),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg2).unwrap();

        let new_owner_info = message_info(&new_owner_addr, &[]);

        let accept_msg = ExecuteMsg::AcceptOwnership {};
        let accept_res = execute(
            deps.as_mut(),
            env.clone(),
            new_owner_info.clone(),
            accept_msg,
        )
        .unwrap();

        assert_eq!(accept_res.attributes.len(), 2);
        assert_eq!(accept_res.attributes[0], ("action", "accept_ownership"));
        assert_eq!(
            accept_res.attributes[1],
            ("new_owner", new_owner_addr.to_string())
        );

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner_addr);

        let pending_owner = PENDING_OWNER.load(&deps.storage).unwrap();
        assert_eq!(pending_owner, None);

        let someone_else = deps.api.addr_make("someone_else").to_string();
        let msg3 = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: someone_else,
        };
        let err = execute(deps.as_mut(), env.clone(), info.clone(), msg3).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("Expected Unauthorized error, got: {:?}", e),
        }
    }

    #[test]
    fn test_is_validator() {
        let (mut deps, env, _info, _delegation_manager, _initial_owner, _pauser, _unpauser) =
            instantiate_contract();

        let validator_addr = deps.api.addr_make("validator");

        VALIDATOR
            .save(&mut deps.storage, validator_addr.clone(), &true)
            .unwrap();

        let query_msg = QueryMsg::IsValidator {
            validator: validator_addr.to_string(),
        };

        let response: ValidatorResponse =
            from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert_eq!(response.is_validator, true);

        let non_existent_validator = deps.api.addr_make("non_existent_validator");
        let query_msg = QueryMsg::IsValidator {
            validator: non_existent_validator.to_string(),
        };

        let response: ValidatorResponse =
            from_json(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();

        assert_eq!(response.is_validator, false);
    }

    #[test]
    fn test_query_calculate_slash_hash() {
        let (deps, env, _info, _delegation_manager, _initial_owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");
        let slash_validator = vec![
            deps.api.addr_make("validator1").to_string(),
            deps.api.addr_make("validator2").to_string(),
        ];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(10),
            slash_signature: 1,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys =
            vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()];

        let msg = QueryMsg::CalculateSlashHash {
            sender: slasher_addr.to_string(),
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let res_query = query(deps.as_ref(), env.clone(), msg).unwrap();

        let result: CalculateSlashHashResponse = from_json(&res_query).unwrap();

        assert!(!result.message_bytes.is_empty());
        assert!(!res_query.is_empty());
    }

    #[test]
    fn test_get_minimal_slash_signature() {
        let (mut deps, env, _info, _delegation_manager, _initial_owner, _pauser, _unpauser) =
            instantiate_contract();

        let minimal_signature: u64 = 10;
        MINIMAL_SLASH_SIGNATURE
            .save(&mut deps.storage, &minimal_signature)
            .unwrap();

        let query_msg = QueryMsg::GetMinimalSlashSignature {};

        let response: MinimalSlashSignatureResponse =
            from_json(&query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap()).unwrap();

        assert_eq!(response.minimal_slash_signature, minimal_signature);

        let new_minimal_signature: u64 = 20;
        MINIMAL_SLASH_SIGNATURE
            .save(&mut deps.storage, &new_minimal_signature)
            .unwrap();

        let response: MinimalSlashSignatureResponse =
            from_json(&query(deps.as_ref(), env, query_msg.clone()).unwrap()).unwrap();

        assert_eq!(response.minimal_slash_signature, new_minimal_signature);
    }

    #[test]
    fn test_submit_slash_request() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");
        let slash_validator = vec![
            deps.api.addr_make("validator1").to_string(),
            deps.api.addr_make("validator2").to_string(),
        ];
        let slash_validator_addr = vec![
            deps.api.addr_make("validator1"),
            deps.api.addr_make("validator2"),
        ];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(10),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let expected_slash_details = SlashDetails {
            slasher: slasher_addr.clone(),
            operator: operator_addr.clone(),
            share: Uint128::new(10),
            slash_signature: 2,
            slash_validator: slash_validator_addr.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "AggozHu/LCQC7T7WATaTNHOm8XTOTKNzVz+s8SKoZm85".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == delegation_manager.to_string() => {
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

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        for validator in slash_validator_addr.iter() {
            VALIDATOR
                .save(&mut deps.storage, validator.clone(), &true)
                .unwrap();
        }

        let msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), msg);

        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.events.len(), 3);

        let event = &res.events[0];
        assert_eq!(event.ty, "slash_request_submitted");
        assert_eq!(event.attributes.len(), 7);

        assert_eq!(event.attributes[0].key, "slash_hash");
        assert!(event.attributes[0].value.len() > 0);

        assert_eq!(event.attributes[1].key, "sender");
        assert_eq!(event.attributes[1].value, slasher_addr.to_string());

        assert_eq!(event.attributes[2].key, "operator");
        assert_eq!(event.attributes[2].value, operator_addr.to_string());

        assert_eq!(event.attributes[3].key, "share");
        assert_eq!(event.attributes[3].value, slash_details.share.to_string());

        assert_eq!(event.attributes[4].key, "start_time");
        assert_eq!(
            event.attributes[4].value,
            slash_details.start_time.to_string()
        );

        assert_eq!(event.attributes[5].key, "end_time");
        assert_eq!(
            event.attributes[5].value,
            slash_details.end_time.to_string()
        );

        assert_eq!(event.attributes[6].key, "status");
        assert_eq!(event.attributes[6].value, slash_details.status.to_string());

        for (i, validator) in slash_validator.iter().enumerate() {
            let event = &res.events[i + 1];
            assert_eq!(event.ty, "slash_validator_checked");
            assert_eq!(event.attributes.len(), 1);
            assert_eq!(event.attributes[0].key, "validator");
            assert_eq!(event.attributes[0].value, validator.to_string());
        }

        let slash_hash = event.attributes[0].value.clone();
        let stored_slash_details = SLASH_DETAILS.load(&deps.storage, slash_hash).unwrap();
        assert_eq!(stored_slash_details, expected_slash_details.clone());
    }

    #[test]
    fn test_cancel_slash_request() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");
        let slash_validator = vec![
            deps.api.addr_make("validator1").to_string(),
            deps.api.addr_make("validator2").to_string(),
        ];
        let slash_validator_addr = vec![
            deps.api.addr_make("validator1"),
            deps.api.addr_make("validator2"),
        ];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(10),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "AqYgz77kusvLMmrWWpiW7qYrUCDE3JfhxjgqY6ukj3tM".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == delegation_manager.to_string() => {
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

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        for validator in slash_validator_addr.iter() {
            VALIDATOR
                .save(&mut deps.storage, validator.clone(), &true)
                .unwrap();
        }

        let msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), msg);
        assert!(res.is_ok());

        let res = res.unwrap();
        let slash_hash = res.events[0].attributes[0].value.clone();

        let cancel_msg = ExecuteMsg::CancelSlashRequest {
            slash_hash: slash_hash.clone(),
        };

        let cancel_res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), cancel_msg);
        assert!(cancel_res.is_ok());

        let cancel_res = cancel_res.unwrap();
        assert_eq!(cancel_res.events.len(), 1);

        let event = &cancel_res.events[0];
        assert_eq!(event.ty, "cancel_slash_request");
        assert_eq!(event.attributes.len(), 3);

        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "cancel_slash_request");

        assert_eq!(event.attributes[1].key, "slash_hash");
        assert_eq!(event.attributes[1].value, slash_hash.clone());

        assert_eq!(event.attributes[2].key, "slash_details_status");
        assert_eq!(event.attributes[2].value, "false");

        let updated_slash_details = SLASH_DETAILS.load(&deps.storage, slash_hash).unwrap();
        assert_eq!(updated_slash_details.status, false);
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
            bech32::encode("cosmwasm", ripemd160_result.to_base32(), Variant::Bech32).unwrap();
        (
            Addr::unchecked(address),
            secret_key,
            public_key_bytes.to_vec(),
        )
    }

    #[test]
    fn test_execute_slash_request() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");

        let private_key_hex1 = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator1, secret_key1, public_key_bytes1) =
            generate_osmosis_public_key_from_private_key(private_key_hex1);

        let private_key_hex2 = "e5dbc50cb04311a2a5c3c0e0258d396e962f64c6c2f758458ffb677d7f0c0e94";
        let (validator2, secret_key2, public_key_bytes2) =
            generate_osmosis_public_key_from_private_key(private_key_hex2);

        let slash_validator = vec![validator1.to_string(), validator2.to_string()];
        let slash_validator_addr = vec![validator1, validator2];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(1_000_000),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let expected_slash_details = SlashDetails {
            slasher: slasher_addr.clone(),
            operator: operator_addr.clone(),
            share: Uint128::new(1_000_000),
            slash_signature: 2,
            slash_validator: slash_validator_addr.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "AggozHu/LCQC7T7WATaTNHOm8XTOTKNzVz+s8SKoZm85".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == delegation_manager.to_string() =>
            {
                let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    DelegationManagerQueryMsg::IsOperator { .. } => {
                        let operator_response = OperatorResponse { is_operator: true };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&operator_response).unwrap(),
                        ))
                    }
                    DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                        let stakers_response = OperatorStakersResponse {
                            stakers_and_shares: vec![
                                StakerShares {
                                    staker: deps.api.addr_make("staker1"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(10_000_000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(20_000_000)),
                                    ],
                                },
                                StakerShares {
                                    staker: deps.api.addr_make("staker2"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(15_000_000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(25_000_000)),
                                    ],
                                },
                            ],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&stakers_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query_msg).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: slash_validator_addr.iter().map(|v| v.to_string()).collect(),
            validator_public_keys: validators_public_keys.clone(),
            values: vec![true, true],
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(response.is_ok());

        let submit_msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let submit_res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg);

        assert!(submit_res.is_ok());

        let submit_res = submit_res.unwrap();
        let slash_hash = submit_res.events[0].attributes[0].value.clone();

        let message_byte = calculate_slash_hash(
            &slasher_addr,
            &expected_slash_details,
            &env.contract.address,
            &[public_key_bytes1, public_key_bytes2],
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature1 = secp.sign_ecdsa(&message, &secret_key1);
        let signature2 = secp.sign_ecdsa(&message, &secret_key2);
        let signature_bytes1 = signature1.serialize_compact().to_vec();
        let signature_bytes2 = signature2.serialize_compact().to_vec();

        let signature_base64_1 = general_purpose::STANDARD.encode(signature_bytes1);
        let signature_base64_2 = general_purpose::STANDARD.encode(signature_bytes2);

        let execute_msg = ExecuteMsg::ExecuteSlashRequest {
            slash_hash: slash_hash.clone(),
            signatures: vec![signature_base64_1.clone(), signature_base64_2.clone()],
            validators_public_keys: validators_public_keys.clone(),
        };

        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(execute_res.is_ok());

        let execute_res = execute_res.unwrap();

        assert_eq!(execute_res.events.len(), 1);
        let event = &execute_res.events[0];
        assert_eq!(event.ty, "slash_executed_weighted");
        assert_eq!(event.attributes.len(), 4);

        assert_eq!(event.attributes[0].key, "action");
        assert_eq!(event.attributes[0].value, "execute_slash_request");

        assert_eq!(event.attributes[1].key, "slash_hash");
        assert_eq!(event.attributes[1].value, slash_hash.clone());

        assert_eq!(event.attributes[2].key, "operator");
        assert_eq!(event.attributes[2].value, operator_addr.to_string());

        assert_eq!(event.attributes[3].key, "total_slash_share");
        assert_eq!(event.attributes[3].value, 1_000_000.to_string());

        let updated_slash_details = SLASH_DETAILS.load(&deps.storage, slash_hash).unwrap();
        assert_eq!(updated_slash_details.status, false);
    }

    #[test]
    fn test_set_strategy_manager() {
        let (mut deps, env, info, _delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let new_strategy_manager = deps.api.addr_make("new_strategy_manager").to_string();

        let set_strategy_manager_msg = ExecuteMsg::SetStrategyManager {
            new_strategy_manager: new_strategy_manager.clone(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            set_strategy_manager_msg,
        );
        assert!(res.is_ok());

        let strategy_manager_addr = STRATEGY_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(
            strategy_manager_addr,
            Addr::unchecked(new_strategy_manager.clone())
        );

        let event = res.unwrap().events[0].clone();
        assert_eq!(event.ty, "strategy_manager_set");
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_strategy_manager");
        assert_eq!(event.attributes[1].key, "new_strategy_manager");
        assert_eq!(event.attributes[1].value, new_strategy_manager.clone());
    }

    #[test]
    fn test_slash_share_calculation() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();
        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        let operator_addr = deps.api.addr_make("operator");
        let total_slash_amount = Uint128::new(40_000_000); // 40e6

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator, _, _) = generate_osmosis_public_key_from_private_key(private_key_hex);

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: total_slash_amount,
            slash_signature: 1,
            slash_validator: vec![validator.to_string()],
            reason: "Test slash".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        // Mock delegation manager query response
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == delegation_manager.to_string() =>
            {
                let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    DelegationManagerQueryMsg::IsOperator { .. } => {
                        let operator_response = OperatorResponse { is_operator: true };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&operator_response).unwrap(),
                        ))
                    }
                    DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                        let stakers_response = OperatorStakersResponse {
                            stakers_and_shares: vec![
                                StakerShares {
                                    staker: deps.api.addr_make("staker_a"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(20_000_000)), // 20e6
                                        (deps.api.addr_make("strategy2"), Uint128::new(15_000_000)), // 15e6
                                    ],
                                },
                                StakerShares {
                                    staker: deps.api.addr_make("staker_b"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(30_000_000)), // 30e6
                                        (deps.api.addr_make("strategy2"), Uint128::new(1_000_000)), // 1e6
                                    ],
                                },
                            ],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&stakers_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query_msg).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let slasher_info = message_info(&slasher_addr, &[]);

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: vec![validator.to_string()],
            validator_public_keys: vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()],
            values: vec![true],
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(response.is_ok());

        let submit_msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()],
        };

        let submit_res =
            execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg).unwrap();
        let slash_hash = submit_res.events[0].attributes[0].value.clone();

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (_validator, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);

        let message_byte = calculate_slash_hash(
            &slasher_addr,
            &SlashDetails {
                slasher: slasher_addr.clone(),
                operator: operator_addr.clone(),
                share: total_slash_amount,
                slash_signature: 1,
                slash_validator: vec![validator.clone()],
                reason: "Test slash".to_string(),
                start_time: env.block.time.seconds(),
                end_time: env.block.time.seconds() + 1000,
                status: true,
            },
            &env.contract.address,
            &[public_key_bytes],
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();

        let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

        let execute_msg = ExecuteMsg::ExecuteSlashRequest {
            slash_hash,
            signatures: vec![signature_base64],
            validators_public_keys: vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()],
        };

        let res = execute(deps.as_mut(), env.clone(), slasher_info, execute_msg).unwrap();

        let mut found_messages = vec![];
        for submsg in res.messages {
            match submsg.msg {
                CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) => {
                    let parsed: Result<DelegationManagerExecuteMsg, _> = from_json(&msg);
                    if let Ok(DelegationManagerExecuteMsg::DecreaseDelegatedShares {
                        shares, ..
                    }) = parsed
                    {
                        found_messages.push(shares);
                    }
                }
                _ => {}
            }
        }

        // Calculate expected slash shares

        // Total shares = 20e6 + 15e6 + 30e6 + 1e6 = 66e6
        // Total slash amount = 40e6

        // Staker A - Strategy 1: 20e6 * (40e6 / 66e6) ≈ 12.121212e6
        // Staker A - Strategy 2: 15e6 * (40e6 / 66e6) ≈ 9.090909e6
        // Staker B - Strategy 1: 30e6 * (40e6 / 66e6) ≈ 18.181818e6
        // Staker B - Strategy 2: 1e6 * (40e6 / 66e6) ≈ 0.606061e6

        // Total shares = 12.121212e6 + 9.090909e6 + 18.181818e6 + 0.606061e6 = 40e6

        let expected_shares = vec![
            Uint128::new(12_121_212),
            Uint128::new(9_090_909),
            Uint128::new(18_181_818),
            Uint128::new(606_061),
        ];

        for (i, shares) in found_messages.iter().enumerate() {
            let diff = shares.u128().abs_diff(expected_shares[i].u128());

            // Allow 1 unit of error due to rounding
            assert!(
                diff <= 1,
                "Share calculation mismatch at index {}: expected {}, got {}",
                i,
                expected_shares[i],
                shares
            );
        }
    }

    #[test]
    fn test_slash_share_calculation_edge_cases() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();
        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        let operator_addr = deps.api.addr_make("operator");

        let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator, _, _) = generate_osmosis_public_key_from_private_key(private_key_hex);

        // test case 1: total_slash_share is 0
        {
            let slash_details = ExecuteSlashDetails {
                slasher: slasher_addr.to_string(),
                operator: operator_addr.to_string(),
                share: Uint128::zero(),
                slash_signature: 1,
                slash_validator: vec![validator.to_string()],
                reason: "Test slash".to_string(),
                start_time: env.block.time.seconds(),
                end_time: env.block.time.seconds() + 1000,
                status: true,
            };

            let slasher_info = message_info(&slasher_addr, &[]);

            let execute_msg = ExecuteMsg::SetSlasherValidator {
                validators: vec![validator.to_string()],
                validator_public_keys: vec![
                    "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()
                ],
                values: vec![true],
            };

            let response = execute(
                deps.as_mut(),
                env.clone(),
                slasher_info.clone(),
                execute_msg,
            );

            assert!(response.is_ok());

            let submit_msg = ExecuteMsg::SubmitSlashRequest {
                slash_details: slash_details.clone(),
                validators_public_keys: vec![
                    "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()
                ],
            };

            let err =
                execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg).unwrap_err();
            assert_eq!(err, ContractError::InvalidShare {});
        }

        // test case 2: small shares and total_slash_share > sum_of_shares
        {
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { contract_addr, msg }
                    if *contract_addr == delegation_manager.to_string() =>
                {
                    let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                    match query_msg {
                        DelegationManagerQueryMsg::IsOperator { .. } => {
                            let operator_response = OperatorResponse { is_operator: true };
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&operator_response).unwrap(),
                            ))
                        }
                        DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                            let stakers_response = OperatorStakersResponse {
                                stakers_and_shares: vec![
                                    StakerShares {
                                        staker: deps.api.addr_make("staker_a"),
                                        shares_per_strategy: vec![
                                            (deps.api.addr_make("strategy1"), Uint128::new(1)), // small shares
                                        ],
                                    },
                                    StakerShares {
                                        staker: deps.api.addr_make("staker_b"),
                                        shares_per_strategy: vec![
                                            (deps.api.addr_make("strategy1"), Uint128::zero()), // 0 shares
                                        ],
                                    },
                                ],
                            };
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&stakers_response).unwrap(),
                            ))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "Unhandled request".to_string(),
                            request: to_json_binary(&query_msg).unwrap(),
                        }),
                    }
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "Unhandled request".to_string(),
                    request: to_json_binary(&query).unwrap(),
                }),
            });

            let slash_details = ExecuteSlashDetails {
                slasher: slasher_addr.to_string(),
                operator: operator_addr.to_string(),
                share: Uint128::new(10), // total_slash_share > sum_of_shares
                slash_signature: 1,
                slash_validator: vec![validator.to_string()],
                reason: "Test slash".to_string(),
                start_time: env.block.time.seconds(),
                end_time: env.block.time.seconds() + 1000,
                status: true,
            };

            let slasher_info = message_info(&slasher_addr, &[]);
            let submit_msg = ExecuteMsg::SubmitSlashRequest {
                slash_details: slash_details.clone(),
                validators_public_keys: vec![
                    "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()
                ],
            };

            let submit_res =
                execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg).unwrap();
            let slash_hash = submit_res.events[0].attributes[0].value.clone();

            let private_key_hex =
                "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
            let (_validator, secret_key, public_key_bytes) =
                generate_osmosis_public_key_from_private_key(private_key_hex);

            let message_byte = calculate_slash_hash(
                &slasher_addr,
                &SlashDetails {
                    slasher: slasher_addr.clone(),
                    operator: operator_addr.clone(),
                    share: Uint128::new(10),
                    slash_signature: 1,
                    slash_validator: vec![validator.clone()],
                    reason: "Test slash".to_string(),
                    start_time: env.block.time.seconds(),
                    end_time: env.block.time.seconds() + 1000,
                    status: true,
                },
                &env.contract.address,
                &[public_key_bytes],
            );

            let secp = Secp256k1::new();
            let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
            let signature = secp.sign_ecdsa(&message, &secret_key);
            let signature_bytes = signature.serialize_compact().to_vec();

            let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

            let execute_msg = ExecuteMsg::ExecuteSlashRequest {
                slash_hash,
                signatures: vec![signature_base64],
                validators_public_keys: vec![
                    "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()
                ],
            };

            let err = execute(deps.as_mut(), env.clone(), slasher_info, execute_msg).unwrap_err();
            assert_eq!(
                err,
                ContractError::InsufficientSharesForStaker {
                    staker: deps.api.addr_make("staker_a").to_string()
                }
            );
        }
    }

    #[test]
    fn test_execute_slash_request_signature_not_enough() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");

        let private_key_hex1 = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator1, secret_key1, public_key_bytes1) =
            generate_osmosis_public_key_from_private_key(private_key_hex1);

        let private_key_hex2 = "e5dbc50cb04311a2a5c3c0e0258d396e962f64c6c2f758458ffb677d7f0c0e94";
        let (validator2, _, _) = generate_osmosis_public_key_from_private_key(private_key_hex2);

        let slash_validator = vec![validator1.to_string(), validator2.to_string()];
        let slash_validator_addr = vec![validator1, validator2];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(1_000_000),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let expected_slash_details = SlashDetails {
            slasher: slasher_addr.clone(),
            operator: operator_addr.clone(),
            share: Uint128::new(1_000_000),
            slash_signature: 2,
            slash_validator: slash_validator_addr.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == delegation_manager.to_string() =>
            {
                let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    DelegationManagerQueryMsg::IsOperator { .. } => {
                        let operator_response = OperatorResponse { is_operator: true };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&operator_response).unwrap(),
                        ))
                    }
                    DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                        let stakers_response = OperatorStakersResponse {
                            stakers_and_shares: vec![
                                StakerShares {
                                    staker: deps.api.addr_make("staker1"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(10_000_000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(20_000_000)),
                                    ],
                                },
                                StakerShares {
                                    staker: deps.api.addr_make("staker2"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(15_000_000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(25_000_000)),
                                    ],
                                },
                            ],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&stakers_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query_msg).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: slash_validator_addr.iter().map(|v| v.to_string()).collect(),
            validator_public_keys: validators_public_keys.clone(),
            values: vec![true, true],
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(response.is_ok());

        let submit_msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let submit_res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg);

        assert!(submit_res.is_ok());

        let submit_res = submit_res.unwrap();
        let slash_hash = submit_res.events[0].attributes[0].value.clone();

        // In this case, the slash signature is 2, but the validator only has 1 public key
        // So the signature is not enough

        let message_byte = calculate_slash_hash(
            &slasher_addr,
            &expected_slash_details,
            &env.contract.address,
            &[public_key_bytes1.clone(), public_key_bytes1.clone()],
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key1);
        let signature_bytes = signature.serialize_compact().to_vec();

        let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

        let execute_msg = ExecuteMsg::ExecuteSlashRequest {
            slash_hash: slash_hash.clone(),
            signatures: vec![signature_base64.clone(), signature_base64.clone()],
            validators_public_keys: validators_public_keys.clone(),
        };

        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(execute_res.is_err());
        match execute_res.unwrap_err() {
            ContractError::SignatureNotEnough {} => {}
            err => panic!("Expected SignatureNotEnough error, got: {:?}", err),
        }
    }

    #[test]
    fn test_submit_slash_request_duplicate_validator() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");
        let slash_validator = vec![
            deps.api.addr_make("validator1").to_string(),
            deps.api.addr_make("validator1").to_string(),
        ];
        let slash_validator_addr = vec![
            deps.api.addr_make("validator1"),
            deps.api.addr_make("validator1"),
        ];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(10),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == delegation_manager.to_string() => {
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

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        for validator in slash_validator_addr.iter() {
            VALIDATOR
                .save(&mut deps.storage, validator.clone(), &true)
                .unwrap();
        }

        let msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::DuplicateValidator {} => {}
            err => panic!("Expected DuplicateValidator error, got: {:?}", err),
        }
    }

    #[test]
    fn test_submit_slash_request_end_time_too_large() {
        let (mut deps, env, _info, delegation_manager, _owner, _pauser, _unpauser) =
            instantiate_contract();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");
        let slash_validator = vec![
            deps.api.addr_make("validator1").to_string(),
            deps.api.addr_make("validator2").to_string(),
        ];
        let slash_validator_addr = vec![
            deps.api.addr_make("validator1"),
            deps.api.addr_make("validator2"),
        ];

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(10),
            slash_signature: 2,
            slash_validator: slash_validator.clone(),
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 999_999_999_999_999,
            status: true,
        };

        let validators_public_keys = vec![
            "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string(),
            "AggozHu/LCQC7T7WATaTNHOm8XTOTKNzVz+s8SKoZm85".to_string(),
        ];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == delegation_manager.to_string() => {
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

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        for validator in slash_validator_addr.iter() {
            VALIDATOR
                .save(&mut deps.storage, validator.clone(), &true)
                .unwrap();
        }

        let msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::EndTimeTooLarge {} => {}
            err => panic!("Expected EndTimeTooLarge error, got: {:?}", err),
        }
    }

    #[test]
    fn test_pause_and_unpause_execute_slash_request() {
        let (mut deps, env, _info, delegation_manager, _owner, pauser, unpauser) =
            instantiate_contract();

        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");

        let private_key_hex1 = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator1, secret_key1, public_key_bytes1) =
            generate_osmosis_public_key_from_private_key(private_key_hex1);

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(1000000),
            slash_signature: 1,
            slash_validator: vec![validator1.to_string()],
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let expected_slash_details = SlashDetails {
            slasher: slasher_addr.clone(),
            operator: operator_addr.clone(),
            share: Uint128::new(1000000),
            slash_signature: 1,
            slash_validator: vec![validator1.clone()],
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys =
            vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == delegation_manager.to_string() =>
            {
                let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    DelegationManagerQueryMsg::IsOperator { .. } => {
                        let operator_response = OperatorResponse { is_operator: true };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&operator_response).unwrap(),
                        ))
                    }
                    DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                        let stakers_response = OperatorStakersResponse {
                            stakers_and_shares: vec![
                                StakerShares {
                                    staker: deps.api.addr_make("staker1"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(10000000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(20000000)),
                                    ],
                                },
                                StakerShares {
                                    staker: deps.api.addr_make("staker2"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(15000000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(25000000)),
                                    ],
                                },
                            ],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&stakers_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query_msg).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: vec![validator1.to_string()],
            validator_public_keys: validators_public_keys.clone(),
            values: vec![true],
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(response.is_ok());

        let submit_msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let submit_res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg);

        assert!(submit_res.is_ok());

        let submit_res = submit_res.unwrap();
        let slash_hash = submit_res.events[0].attributes[0].value.clone();

        let message_byte = calculate_slash_hash(
            &slasher_addr,
            &expected_slash_details,
            &env.contract.address,
            &[public_key_bytes1],
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key1);
        let signature_bytes = signature.serialize_compact().to_vec();

        let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

        let pauser_info = message_info(&pauser, &[]);

        // Pause the execute slash request function
        let pause_msg = ExecuteMsg::PauseBit {
            index: PAUSED_EXECUTE_SLASH_REQUEST,
        };
        let pause_res = execute(deps.as_mut(), env.clone(), pauser_info.clone(), pause_msg);
        assert!(pause_res.is_ok());

        // Try to execute slash request while paused, should fail
        let execute_msg = ExecuteMsg::ExecuteSlashRequest {
            slash_hash: slash_hash.clone(),
            signatures: vec![signature_base64.clone()],
            validators_public_keys: validators_public_keys.clone(),
        };

        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg.clone(),
        );

        assert!(execute_res.is_err());
        if let Err(err) = execute_res {
            match err {
                ContractError::Std(err) if err.to_string().contains("Functionality is paused") => {
                    ()
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let unpauser_info = message_info(&unpauser, &[]);

        // Unpause the execute slash request function
        let unpause_msg = ExecuteMsg::UnpauseBit {
            index: PAUSED_EXECUTE_SLASH_REQUEST,
        };
        let unpause_res = execute(
            deps.as_mut(),
            env.clone(),
            unpauser_info.clone(),
            unpause_msg,
        );
        assert!(unpause_res.is_ok());

        // Try to execute slash request again, should succeed
        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(execute_res.is_ok());

        let execute_res = execute_res.unwrap();

        assert_eq!(execute_res.events.len(), 1);
        let event = &execute_res.events[0];
        assert_eq!(event.ty, "slash_executed_weighted");
        assert_eq!(event.attributes.len(), 4);

        assert_eq!(event.attributes[0].key, "action");
        assert_eq!(event.attributes[0].value, "execute_slash_request");

        assert_eq!(event.attributes[1].key, "slash_hash");
        assert_eq!(event.attributes[1].value, slash_hash.clone());

        assert_eq!(event.attributes[2].key, "operator");
        assert_eq!(event.attributes[2].value, operator_addr.to_string());

        assert_eq!(event.attributes[3].key, "total_slash_share");
        assert_eq!(event.attributes[3].value, "1000000");

        let updated_slash_details = SLASH_DETAILS.load(&deps.storage, slash_hash).unwrap();
        assert_eq!(updated_slash_details.status, false);
    }

    #[test]
    fn test_pause_and_unpause_all() {
        let (mut deps, env, _info, delegation_manager, _owner, pauser, unpauser) =
            instantiate_contract();

        STRATEGY_MANAGER
            .save(&mut deps.storage, &deps.api.addr_make("strategy_manager"))
            .unwrap();

        let slasher_addr = deps.api.addr_make("slasher");
        let operator_addr = deps.api.addr_make("operator");

        let private_key_hex1 = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
        let (validator1, secret_key1, public_key_bytes1) =
            generate_osmosis_public_key_from_private_key(private_key_hex1);

        let slash_details = ExecuteSlashDetails {
            slasher: slasher_addr.to_string(),
            operator: operator_addr.to_string(),
            share: Uint128::new(1000000),
            slash_signature: 1,
            slash_validator: vec![validator1.to_string()],
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let expected_slash_details = SlashDetails {
            slasher: slasher_addr.clone(),
            operator: operator_addr.clone(),
            share: Uint128::new(1000000),
            slash_signature: 1,
            slash_validator: vec![validator1.clone()],
            reason: "Invalid action".to_string(),
            start_time: env.block.time.seconds(),
            end_time: env.block.time.seconds() + 1000,
            status: true,
        };

        let validators_public_keys =
            vec!["A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD".to_string()];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == delegation_manager.to_string() =>
            {
                let query_msg: DelegationManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    DelegationManagerQueryMsg::IsOperator { .. } => {
                        let operator_response = OperatorResponse { is_operator: true };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&operator_response).unwrap(),
                        ))
                    }
                    DelegationManagerQueryMsg::GetOperatorStakers { .. } => {
                        let stakers_response = OperatorStakersResponse {
                            stakers_and_shares: vec![
                                StakerShares {
                                    staker: deps.api.addr_make("staker1"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(10000000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(20000000)),
                                    ],
                                },
                                StakerShares {
                                    staker: deps.api.addr_make("staker2"),
                                    shares_per_strategy: vec![
                                        (deps.api.addr_make("strategy1"), Uint128::new(15000000)),
                                        (deps.api.addr_make("strategy2"), Uint128::new(25000000)),
                                    ],
                                },
                            ],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&stakers_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query_msg).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        MINIMAL_SLASH_SIGNATURE.save(&mut deps.storage, &1).unwrap();

        SLASHER
            .save(&mut deps.storage, slasher_addr.clone(), &true)
            .unwrap();

        let slasher_info = message_info(&slasher_addr, &[]);

        let execute_msg = ExecuteMsg::SetSlasherValidator {
            validators: vec![validator1.to_string()],
            validator_public_keys: validators_public_keys.clone(),
            values: vec![true],
        };

        let response = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(response.is_ok());

        let submit_msg = ExecuteMsg::SubmitSlashRequest {
            slash_details: slash_details.clone(),
            validators_public_keys: validators_public_keys.clone(),
        };

        let submit_res = execute(deps.as_mut(), env.clone(), slasher_info.clone(), submit_msg);

        assert!(submit_res.is_ok());

        let submit_res = submit_res.unwrap();
        let slash_hash = submit_res.events[0].attributes[0].value.clone();

        let message_byte = calculate_slash_hash(
            &slasher_addr,
            &expected_slash_details,
            &env.contract.address,
            &[public_key_bytes1],
        );

        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, &secret_key1);
        let signature_bytes = signature.serialize_compact().to_vec();

        let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

        let pauser_info = message_info(&pauser, &[]);

        // Pause all functions
        let pause_all_msg = ExecuteMsg::PauseAll {};
        let pause_all_res = execute(
            deps.as_mut(),
            env.clone(),
            pauser_info.clone(),
            pause_all_msg,
        );
        println!("{:?}", pause_all_res);
        assert!(pause_all_res.is_ok());

        // Try to execute slash request while paused, should fail
        let execute_msg = ExecuteMsg::ExecuteSlashRequest {
            slash_hash: slash_hash.clone(),
            signatures: vec![signature_base64.clone()],
            validators_public_keys: validators_public_keys.clone(),
        };

        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg.clone(),
        );

        assert!(execute_res.is_err());
        if let Err(err) = execute_res {
            match err {
                ContractError::Std(err) if err.to_string().contains("Functionality is paused") => {
                    ()
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let unpauser_info = message_info(&unpauser, &[]);

        // Unpause all functions
        let unpause_all_msg = ExecuteMsg::UnpauseAll {};
        let unpause_all_res = execute(
            deps.as_mut(),
            env.clone(),
            unpauser_info.clone(),
            unpause_all_msg,
        );
        assert!(unpause_all_res.is_ok());

        // Try to execute slash request again, should succeed
        let execute_res = execute(
            deps.as_mut(),
            env.clone(),
            slasher_info.clone(),
            execute_msg,
        );

        assert!(execute_res.is_ok());

        let execute_res = execute_res.unwrap();

        assert_eq!(execute_res.events.len(), 1);
        let event = &execute_res.events[0];
        assert_eq!(event.ty, "slash_executed_weighted");
        assert_eq!(event.attributes.len(), 4);

        assert_eq!(event.attributes[0].key, "action");
        assert_eq!(event.attributes[0].value, "execute_slash_request");

        assert_eq!(event.attributes[1].key, "slash_hash");
        assert_eq!(event.attributes[1].value, slash_hash.clone());

        assert_eq!(event.attributes[2].key, "operator");
        assert_eq!(event.attributes[2].value, operator_addr.to_string());

        assert_eq!(event.attributes[3].key, "total_slash_share");
        assert_eq!(event.attributes[3].value, "1000000");

        let updated_slash_details = SLASH_DETAILS.load(&deps.storage, slash_hash).unwrap();
        assert_eq!(updated_slash_details.status, false);
    }
}
