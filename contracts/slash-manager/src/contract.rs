use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{MinimalSlashSignatureResponse, SlashDetailsResponse, ValidatorResponse},
    state::{
        DELEGATION_MANAGER, MINIMAL_SLASH_SIGNATURE, OWNER, SLASHER, SLASH_DETAILS, VALIDATOR, STRATEGY_MANAGER
    },
    utils::{calculate_slash_hash, recover, validate_addresses, SlashDetails},
};

use common::delegation::{
    ExecuteMsg as DelegationManagerExecuteMsg, OperatorResponse,
    QueryMsg as DelegationManagerQueryMsg, OperatorStakersResponse
};
use common::strategy::ExecuteMsg as StrategyManagerExecuteMsg;
use common::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use common::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, Uint128, SubMsg, WasmMsg
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
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
        ExecuteMsg::SubmitSlashRequest { 
            slash_details,
            validators_public_keys 
        } => {
            let slasher_addr = deps.api.addr_validate(&slash_details.slasher)?;
            let operator_addr = deps.api.addr_validate(&slash_details.operator)?;
            let slash_validator = validate_addresses(deps.api, &slash_details.slash_validator)?;

            let validators_public_keys_binary: Result<Vec<Binary>, ContractError> = validators_public_keys
            .iter()
            .map(|val| Binary::from_base64(val).map_err(|_| ContractError::InvalidValidator {}))
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
            validators_public_keys
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
            let validators_binary = validators_binary?;
            
            execute_slash_request(deps, env, info, slash_hash, signatures_binary, validators_binary) 
        },
        ExecuteMsg::CancelSlashRequest { slash_hash } => {
            cancel_slash_request(deps, info, slash_hash)
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
        ExecuteMsg::SetDelegationManager { new_delegation_manager } => {
            let new_delegation_manager_addr = deps.api.addr_validate(&new_delegation_manager)?;
            set_delegation_manager(deps, info, new_delegation_manager_addr)
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
    validators_public_keys: Vec<Binary>,
) -> Result<Response, ContractError> {
    only_slasher(deps.as_ref(), &info)?;

    if slash_details.share.is_zero() {
        return Err(ContractError::InvalidShare {});
    }

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
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

    let current_minimal_signature = MINIMAL_SLASH_SIGNATURE.load(deps.storage)?;

    if slash_details.slash_signature < current_minimal_signature {
        return Err(ContractError::InvalidSlashSignature {});
    }

    for validator in slash_details.slash_validator.iter() {
        if !VALIDATOR.load(deps.storage, validator.clone())? {
            return Err(ContractError::Unauthorized {});
        }
    }

    if slash_details.start_time == 0 || slash_details.start_time < env.block.time.seconds() {
        return Err(ContractError::InvalidStartTime {});
    }

    if slash_details.end_time == 0 || slash_details.end_time < env.block.time.seconds() {
        return Err(ContractError::InvalidEndTime {});
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
    slash_hash: String,
    signatures: Vec<Binary>,
    validators_public_keys: Vec<Binary>,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_EXECUTE_SLASH_REQUEST)?;

    only_slasher(deps.as_ref(), &info)?;

    let mut slash_details = match SLASH_DETAILS.may_load(deps.storage, slash_hash.clone())? {
        Some(details) => details,
        None => return Err(ContractError::SlashDetailsNotFound {}),
    };

    if !slash_details.status {
        return Err(ContractError::InvalidSlashStatus {});
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

    for (signature, public_key) in signatures.iter().zip(validators_public_keys.iter()) {
        if !recover(&message_bytes, signature.as_slice(), public_key.as_slice())? {
            return Err(ContractError::InvalidSignature {});
        }
    }

    let query_msg = DelegationManagerQueryMsg::GetOperatorStakers {
        operator: slash_details.operator.to_string(),
    };
    let stakers_response: OperatorStakersResponse = deps.querier.query_wasm_smart(
        DELEGATION_MANAGER.load(deps.storage)?,
        &query_msg
    )?;

    let total_shares: Uint128 = stakers_response.stakers_and_shares
        .iter()
        .flat_map(|staker_shares| &staker_shares.shares_per_strategy)
        .map(|(_, shares)| shares)
        .sum();

    let mut messages = vec![];

    for staker_shares in stakers_response.stakers_and_shares {
        for (strategy, shares) in staker_shares.shares_per_strategy {
            let slash_amount = slash_details.share.multiply_ratio(shares, total_shares);
            
            let decrease_delegated_msg = DelegationManagerExecuteMsg::DecreaseDelegatedShares {
                staker: staker_shares.staker.to_string(),
                strategy: strategy.to_string(),
                shares: slash_amount,
            };
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: DELEGATION_MANAGER.load(deps.storage)?.to_string(),
                msg: to_json_binary(&decrease_delegated_msg)?,
                funds: vec![],
            }));

            let remove_share_msg = StrategyManagerExecuteMsg::RemoveShares {
                staker: staker_shares.staker.to_string(),
                strategy: strategy.to_string(),
                shares: slash_amount,
            };

            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: STRATEGY_MANAGER.load(deps.storage)?.to_string(),
                msg: to_json_binary(&remove_share_msg)?,
                funds: vec![],
            }));
        }
    }

    slash_details.status = false;
    SLASH_DETAILS.save(deps.storage, slash_hash.clone(), &slash_details)?;

    let response = Response::new()
        .add_submessages(messages)
        .add_attribute("action", "execute_slash_request")
        .add_attribute("slash_hash", slash_hash)
        .add_attribute("operator", slash_details.operator.to_string())
        .add_attribute("decreased_share", slash_details.share.to_string());

    Ok(response)
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
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
    }
}

fn query_slash_details(deps: Deps, slash_hash: String) -> StdResult<SlashDetailsResponse> {
    let slash_details = SLASH_DETAILS.load(deps.storage, slash_hash)?;
    Ok(SlashDetailsResponse { slash_details })
}

fn query_is_validator(deps: Deps, validator: Addr) -> StdResult<ValidatorResponse> {
    let is_validator = VALIDATOR.load(deps.storage, validator)?;
    Ok(ValidatorResponse { is_validator })
}

fn query_minimal_slash_signature(deps: Deps) -> StdResult<MinimalSlashSignatureResponse> {
    let minimal_slash_signature = MINIMAL_SLASH_SIGNATURE.load(deps.storage)?;
    Ok(MinimalSlashSignatureResponse { minimal_slash_signature })
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{attr, OwnedDeps};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("creator");
        let delegation_manager = deps.api.addr_make("delegation_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let info = message_info(&initial_owner, &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            delegation_manager: delegation_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
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
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            initial_paused_status: 0,
        };
    
        let result = instantiate(deps.as_mut(), env.clone(), invalid_info.clone(), invalid_msg);
        assert!(result.is_err()); 
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        Addr,
        Addr,
        Addr,
        Addr
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("creator");
        let delegation_manager = deps.api.addr_make("delegation_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let info = message_info(&initial_owner, &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            delegation_manager: delegation_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
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
            unpauser
        )
    }
}