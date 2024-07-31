use crate::{
    error::ContractError,
    strategy_manager,
    msg::{InstantiateMsg, SignatureWithExpiry},
    state::{
        DelegationManagerState, DELEGATION_MANAGER_STATE, OPERATOR_DETAILS, OWNER, OperatorDetails, MIN_WITHDRAWAL_DELAY_BLOCKS,
        DELEGATED_TO, STRATEGY_WITHDRAWAL_DELAY_BLOCKS, OPERATOR_SHARES, DELEGATION_APPROVER_SALT_SPENT, DOMAIN_SEPARATOR,
        STAKER_NONCE, PENDING_WITHDRAWALS, CUMULATIVE_WITHDRAWALS_QUEUED, QueuedWithdrawalParams
    },
    utils::{calculate_delegation_approval_digest_hash, calculate_staker_delegation_digest_hash, recover, 
        ApproverDigestHashParams, StakerDigestHashParams, DelegateParams, calculate_withdrawal_root, Withdrawal
    },
};
use strategy_manager::QueryMsg as StrategyManagerQueryMsg;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Uint128, WasmQuery, Event,
    Binary, WasmMsg, CosmosMsg
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_STAKER_OPT_OUT_WINDOW_BLOCKS: u64 = 180 * 24 * 60 * 60 / 12; 
const MAX_WITHDRAWAL_DELAY_BLOCKS: u64 = 216_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = DelegationManagerState {
        strategy_manager: msg.strategy_manager,
        slasher: msg.slasher,
    };

    DELEGATION_MANAGER_STATE.save(deps.storage, &state)?;
    DOMAIN_SEPARATOR.save(deps.storage, &msg.domain_separator)?;
    OWNER.save(deps.storage, &msg.initial_owner)?;
    _set_min_withdrawal_delay_blocks(deps.branch(), msg.min_withdrawal_delay_blocks)?;

    let withdrawal_delay_blocks: Vec<Uint64> = msg.withdrawal_delay_blocks.iter().map(|&block| Uint64::from(block)).collect();
    _set_strategy_withdrawal_delay_blocks(deps.branch(), msg.strategies, withdrawal_delay_blocks)?;

    let response = Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("slasher", state.slasher.to_string())
        .add_attribute("min_withdrawal_delay_blocks", msg.min_withdrawal_delay_blocks.to_string())
        .add_attribute("owner", msg.initial_owner.to_string());

    Ok(response)
}

fn _only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn _only_strategy_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;
    if info.sender != state.strategy_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn set_min_withdrawal_delay_blocks(
    deps: DepsMut,
    info: MessageInfo,
    new_min_withdrawal_delay_blocks: u64,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    _set_min_withdrawal_delay_blocks(deps, new_min_withdrawal_delay_blocks)
}

fn _set_min_withdrawal_delay_blocks(
    deps: DepsMut,
    min_withdrawal_delay_blocks: u64,
) -> Result<Response, ContractError> {
    if min_withdrawal_delay_blocks > MAX_WITHDRAWAL_DELAY_BLOCKS {
        return Err(ContractError::MinCannotBeExceedMAXWITHDRAWALDELAYBLOCKS {});
    }

    let prev_min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS.may_load(deps.storage)?.unwrap_or(0);

    MIN_WITHDRAWAL_DELAY_BLOCKS.save(deps.storage, &min_withdrawal_delay_blocks)?;

    let event = Event::new("MinWithdrawalDelayBlocksSet")
        .add_attribute("method", "set_min_withdrawal_delay_blocks")
        .add_attribute("prev_min_withdrawal_delay_blocks", prev_min_withdrawal_delay_blocks.to_string())
        .add_attribute("new_min_withdrawal_delay_blocks", min_withdrawal_delay_blocks.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_strategy_withdrawal_delay_blocks(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
    withdrawal_delay_blocks: Vec<Uint64>,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    _set_strategy_withdrawal_delay_blocks(deps, strategies, withdrawal_delay_blocks)
}

fn _set_strategy_withdrawal_delay_blocks(
    deps: DepsMut,
    strategies: Vec<Addr>,
    withdrawal_delay_blocks: Vec<Uint64>,
) -> Result<Response, ContractError> {
    if strategies.len() != withdrawal_delay_blocks.len() {
        return Err(ContractError::InputLengthMismatch {});
    }

    let mut response = Response::new();

    for (i, strategy) in strategies.iter().enumerate() {
        let new_withdrawal_delay_blocks = withdrawal_delay_blocks[i];
        if new_withdrawal_delay_blocks > MAX_WITHDRAWAL_DELAY_BLOCKS.into() {
            return Err(ContractError::CannotBeExceedMAXWITHDRAWALDELAYBLOCKS {});
        }

        let prev_withdrawal_delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.may_load(deps.storage, strategy)?.unwrap_or(Uint64::new(0));

        STRATEGY_WITHDRAWAL_DELAY_BLOCKS.save(deps.storage, strategy, &new_withdrawal_delay_blocks)?;

        let event = Event::new("StrategyWithdrawalDelayBlocksSet")
            .add_attribute("strategy", strategy.to_string())
            .add_attribute("prev", prev_withdrawal_delay_blocks.to_string())
            .add_attribute("new", new_withdrawal_delay_blocks.to_string());
        
        response = response.add_event(event);
    }

    Ok(response)
}

pub fn register_as_operator(
    mut deps: DepsMut,
    info: MessageInfo,
    env: Env,
    sender_public_key: Binary,
    registering_operator_details: OperatorDetails,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let operator = info.sender.clone();

    if DELEGATED_TO.may_load(deps.storage, &operator)?.is_some() {
        return Err(ContractError::StakerAlreadyDelegated {});
    }

    _set_operator_details(deps.branch(), operator.clone(), registering_operator_details)?;

    let empty_signature_and_expiry = SignatureWithExpiry {
        signature: Binary::from(vec![]),
        expiry: 0,
    };

    let params = DelegateParams {
        staker: info.sender.clone(),
        operator: info.sender.clone(),
        public_key: sender_public_key,
        salt: Binary::from(vec![0])
    };

    _delegate(deps, info, env, empty_signature_and_expiry, params)?;

    let mut response = Response::new();

    let register_event = Event::new("OperatorRegistered")
        .add_attribute("operator", operator.to_string());
    response = response.add_event(register_event);

    let metadata_event = Event::new("OperatorMetadataURIUpdated")
        .add_attribute("operator", operator.to_string())
        .add_attribute("metadata_uri", metadata_uri);
    response = response.add_event(metadata_event);

    Ok(response)
}

pub fn modify_operator_details(
    deps: DepsMut,
    info: MessageInfo,
    new_operator_details: OperatorDetails,
) -> Result<Response, ContractError> {
    let operator = info.sender.clone();

    if OPERATOR_DETAILS.may_load(deps.storage, &operator)?.is_none() {
        return Err(ContractError::OperatorNotRegistered {});
    }

    _set_operator_details(deps, operator, new_operator_details)
}

pub fn update_operator_metadata_uri(
    deps: DepsMut,
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let operator = info.sender.clone();

    if OPERATOR_DETAILS.may_load(deps.storage, &operator)?.is_none() {
        return Err(ContractError::OperatorNotRegistered {});
    }

    let mut response = Response::new();
    let metadata_event = Event::new("OperatorMetadataURIUpdated")
        .add_attribute("operator", operator.to_string())
        .add_attribute("metadata_uri", metadata_uri);
    response = response.add_event(metadata_event);

    Ok(response)
}

fn _set_operator_details(
    deps: DepsMut,
    operator: Addr,
    new_operator_details: OperatorDetails,
) -> Result<Response, ContractError> {
    let current_operator_details = OPERATOR_DETAILS.load(deps.storage, &operator)?;

    if new_operator_details.staker_opt_out_window_blocks > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS {
        return Err(ContractError::CannotBeExceedMAXSTAKEROPTOUTWINDOWBLOCKS {});
    }

    if new_operator_details.staker_opt_out_window_blocks < current_operator_details.staker_opt_out_window_blocks {
        return Err(ContractError::CannotBeDecreased {});
    }

    OPERATOR_DETAILS.save(deps.storage, &operator, &new_operator_details)?;

    let event = Event::new("OperatorDetailsSet")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker_opt_out_window_blocks", new_operator_details.staker_opt_out_window_blocks.to_string());

    Ok(Response::new().add_event(event))
}

pub fn delegate_to(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    params: DelegateParams,
    approver_signature_and_expiry: SignatureWithExpiry,
) -> Result<Response, ContractError> {
    let staker = info.sender.clone();
    if DELEGATED_TO.may_load(deps.storage, &staker)?.is_some() {
        return Err(ContractError::StakerAlreadyDelegated {});
    }

    if OPERATOR_DETAILS.may_load(deps.storage, &params.operator)?.is_none() {
        return Err(ContractError::OperatorNotRegistered {});
    }

    _delegate(deps, info, env, approver_signature_and_expiry, params)
}

pub fn delegate_to_by_signature(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: DelegateParams,
    staker_public_key: Binary,
    staker_signature_and_expiry: SignatureWithExpiry,
    approver_signature_and_expiry: SignatureWithExpiry,
) -> Result<Response, ContractError> {
    if staker_signature_and_expiry.expiry < env.block.time.seconds() {
        return Err(ContractError::StakerSignatureExpired {});
    }

    if DELEGATED_TO.may_load(deps.storage, &params.staker)?.is_some() {
        return Err(ContractError::StakerAlreadyDelegated {});
    }

    if OPERATOR_DETAILS.may_load(deps.storage, &params.operator)?.is_none() {
        return Err(ContractError::OperatorNotRegistered {});
    }

    let current_staker_nonce: u128 = STAKER_NONCE.may_load(deps.storage, &params.staker)?.unwrap_or(0);

    let chain_id = env.block.chain_id.clone();

    let digest_params = StakerDigestHashParams {
        staker: params.staker.clone(),
        staker_nonce: current_staker_nonce,
        operator: params.operator.clone(),
        staker_public_key: staker_public_key.clone(),
        expiry: staker_signature_and_expiry.expiry,
        chain_id,
        contract_addr: env.contract.address.clone(),
    };

    let staker_digest_hash = calculate_staker_delegation_digest_hash(&digest_params);

    let staker_nonce = current_staker_nonce.checked_add(1).ok_or(ContractError::NonceOverflow)?;

    STAKER_NONCE.save(deps.storage, &params.staker, &staker_nonce)?;

    recover(&staker_digest_hash, &staker_signature_and_expiry.signature, &staker_public_key)?;

    let params2 = DelegateParams {
        staker: params.staker.clone(),
        operator: params.operator.clone(),
        public_key: params.public_key.clone(),
        salt: params.salt.clone(),
    };

    _delegate(deps, info, env, approver_signature_and_expiry, params2)
}


fn _delegate(
    mut deps: DepsMut,
    info: MessageInfo,
    env: Env,
    approver_signature_and_expiry: SignatureWithExpiry,
    params: DelegateParams,
) -> Result<Response, ContractError> {
    let delegation_approver = OPERATOR_DETAILS.load(deps.storage, &params.operator)?.delegation_approver;

    let current_time: Uint64 = env.block.time.seconds().into();

    if delegation_approver != Addr::unchecked("0") && info.sender != delegation_approver && info.sender != params.operator {
        if approver_signature_and_expiry.expiry < current_time.u64() {
            return Err(ContractError::ApproverSignatureExpired {});
        }

        let approver_salt_str = &params.salt.to_string();

        if DELEGATION_APPROVER_SALT_SPENT.load(deps.storage, (&delegation_approver, approver_salt_str)).is_ok() {
            return Err(ContractError::ApproverSaltSpent {});
        }

        let chain_id = env.block.chain_id.clone();

        let digest_params = ApproverDigestHashParams {
            staker: params.staker.clone(),
            operator: params.operator.clone(),
            delegation_approver: delegation_approver.clone(),
            approver_public_key: params.public_key.clone(),
            approver_salt: params.salt.clone(),
            expiry: approver_signature_and_expiry.expiry,
            chain_id,
            contract_addr: env.contract.address.clone(),
        };

        let approver_digest_hash = calculate_delegation_approval_digest_hash(&digest_params);

        recover(&approver_digest_hash, &approver_signature_and_expiry.signature, &params.public_key)?;

        DELEGATION_APPROVER_SALT_SPENT.save(deps.storage, (&delegation_approver, approver_salt_str), &true)?;
    }

    DELEGATED_TO.save(deps.storage, &params.staker, &params.operator)?;

    let mut response = Response::new();

    let event = Event::new("Delegate")
        .add_attribute("method", "delegate")
        .add_attribute("staker", params.staker.to_string())
        .add_attribute("operator", params.operator.to_string());
    response = response.add_event(event);

    let (strategies, shares) = get_delegatable_shares(deps.as_ref(), params.staker.clone())?;

    for (strategy, share) in strategies.iter().zip(shares.iter()) {
        let increase_shares_response = _increase_operator_shares(
            deps.branch(),
            params.operator.clone(),
            params.staker.clone(),
            strategy.clone(),
            *share,
        )?;
        response = response.add_attributes(increase_shares_response.attributes);
    }

    Ok(response)
}

pub fn undelegate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: Addr,
) -> Result<Vec<String>, ContractError> {
    // Ensure the staker is delegated
    if DELEGATED_TO.may_load(deps.storage, &staker)?.is_none() {
        return Err(ContractError::StakerNotDelegated {});
    }

    // Ensure the staker is not an operator
    if OPERATOR_DETAILS.may_load(deps.storage, &staker)?.is_some() {
        return Err(ContractError::OperatorCannotBeUndelegated {});
    }

    // Ensure the staker is not the zero address
    if staker == Addr::unchecked("0") {
        return Err(ContractError::CannotBeZero {});
    }

    let operator = DELEGATED_TO.load(deps.storage, &staker)?;

    // Ensure the caller is the staker, operator, or delegation approver
    let operator_details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    if info.sender != staker && info.sender != operator && info.sender != operator_details.delegation_approver {
        return Err(ContractError::Unauthorized {});
    }

    // Gather strategies and shares to remove from staker/operator during undelegation
    let (strategies, shares) = get_delegatable_shares(deps.as_ref(), staker.clone())?;

    // Emit an event if this action was not initiated by the staker themselves
    let mut response: Response<()> = Response::new();
    
    if info.sender != staker {
        response = response.add_event(
            Event::new("StakerForceUndelegated")
                .add_attribute("staker", staker.to_string())
                .add_attribute("operator", operator.to_string())
        );
    }

    // Emit the undelegation event
    response.add_event(
        Event::new("StakerUndelegated")
            .add_attribute("staker", staker.to_string())
            .add_attribute("operator", operator.to_string())
    );

    // Undelegate the staker
    DELEGATED_TO.save(deps.storage, &staker, &Addr::unchecked("0"))?;

    let mut withdrawal_roots = Vec::new();
    if strategies.is_empty() {
        Ok(withdrawal_roots)
    } else {
        for (strategy, share) in strategies.iter().zip(shares.iter()) {
            let single_strategy = vec![strategy.clone()];
            let single_share = vec![*share];

            let withdrawal_root = _remove_shares_and_queue_withdrawal(
                deps.branch(),
                env.clone(),
                staker.clone(),
                operator.clone(),
                staker.clone(),
                single_strategy,
                single_share,
            )?;

            withdrawal_roots.push(withdrawal_root);
        }

        Ok(withdrawal_roots)
    }
}

pub fn get_delegatable_shares(
    deps: Deps,
    staker: Addr,
) -> StdResult<(Vec<Addr>, Vec<Uint128>)> {
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;
    let strategy_manager = state.strategy_manager;

    // Query the strategy manager for the staker's deposits
    let query = WasmQuery::Smart {
        contract_addr: strategy_manager.to_string(),
        msg: to_json_binary(&StrategyManagerQueryMsg::GetDeposits { staker: staker.clone() })?,
    }
    .into();

    let response: (Vec<Addr>, Vec<Uint128>) = deps.querier.query(&query)?;

    Ok(response)
}

pub fn increase_delegated_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    _only_strategy_manager(deps.as_ref(), &info)?;

    if let Some(operator) = DELEGATED_TO.may_load(deps.storage, &staker)? {
        _increase_operator_shares(deps, operator, staker, strategy, shares)
    } else {
        Ok(Response::new())
    }
}

fn _increase_operator_shares(
    deps: DepsMut,
    operator: Addr,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let current_shares = OPERATOR_SHARES
        .may_load(deps.storage, (&operator, &strategy))?
        .unwrap_or_else(Uint128::zero);

    let new_shares = current_shares + shares;
    OPERATOR_SHARES.save(deps.storage, (&operator, &strategy), &new_shares)?;

    let event = Event::new("OperatorSharesIncreased")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("new_shares", new_shares.to_string());

    Ok(Response::new().add_event(event))
}

pub fn decrease_delegated_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    _only_strategy_manager(deps.as_ref(), &info)?;

    // Check if the staker is delegated to an operator
    if let Some(operator) = DELEGATED_TO.may_load(deps.storage, &staker)? {
        // Decrease the operator's shares
        _decrease_operator_shares(deps, operator, staker, strategy, shares)
    } else {
        Ok(Response::new())
    }
}

fn _decrease_operator_shares(
    deps: DepsMut,
    operator: Addr,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let current_shares = OPERATOR_SHARES
        .load(deps.storage, (&operator, &strategy))?
        .checked_sub(shares)
        .map_err(|_| ContractError::Underflow)?;

    OPERATOR_SHARES.save(deps.storage, (&operator, &strategy), &current_shares)?;

    let event = Event::new("OperatorSharesDecreased")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string());

    Ok(Response::new().add_event(event))
}

pub fn queue_withdrawals(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    queued_withdrawal_params: Vec<QueuedWithdrawalParams>,
) -> Result<Vec<String>, ContractError> {
    let operator = DELEGATED_TO.may_load(deps.storage, &info.sender)?.unwrap_or_else(|| Addr::unchecked(""));

    let mut withdrawal_roots = Vec::with_capacity(queued_withdrawal_params.len());

    for params in queued_withdrawal_params.iter() {
        if params.strategies.len() != params.shares.len() {
            return Err(ContractError::InputLengthMismatch {});
        }
        if params.withdrawer != info.sender {
            return Err(ContractError::WithdrawerMustBeStaker {});
        }

        let withdrawal_root = _remove_shares_and_queue_withdrawal(
            deps.branch(),
            env.clone(),
            info.sender.clone(),
            operator.clone(),
            params.withdrawer.clone(),
            params.strategies.clone(),
            params.shares.clone(),
        )?;

        withdrawal_roots.push(withdrawal_root.to_string());
    }

    Ok(withdrawal_roots)
}

pub fn complete_queued_withdrawals(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdrawals: Vec<Withdrawal>,
    tokens: Vec<Vec<Addr>>,
    middleware_times_indexes: Vec<u64>, 
    receive_as_tokens: Vec<bool>,
) -> Result<Response, ContractError> {
    // Loop through each withdrawal and complete it
    for (i, withdrawal) in withdrawals.iter().enumerate() {
        _complete_queued_withdrawal(
            deps.branch(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            tokens[i].clone(),
            middleware_times_indexes[i],
            receive_as_tokens[i]
        )?;
    }

    Ok(Response::new())
}

pub fn complete_queued_withdrawal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdrawal: Withdrawal,
    tokens: Vec<Addr>,
    middleware_times_indexe: u64, 
    receive_as_tokens: bool,
) -> Result<Response, ContractError> {
        let response = _complete_queued_withdrawal(
            deps.branch(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            tokens.clone(),
            middleware_times_indexe,
            receive_as_tokens)?;

    Ok(response)
}

fn _complete_queued_withdrawal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdrawal: Withdrawal,
    tokens: Vec<Addr>,
    _middleware_times_index: u64,
    receive_as_tokens: bool,
) -> Result<Response, ContractError> {
}

fn _remove_shares_and_queue_withdrawal(
    mut deps: DepsMut,
    env: Env,
    staker: Addr,
    operator: Addr,
    withdrawer: Addr,
    strategies: Vec<Addr>,
    shares: Vec<Uint128>,
) -> Result<String, ContractError> {
}

fn _withdraw_shares_as_tokens(
    staker: Addr,
    withdrawer: Addr,
    strategy: Addr,
    shares: Uint128,
    token: Addr,
) -> Result<Response, ContractError> {
    let msg = WasmMsg::Execute {
        contract_addr: strategy.to_string(), 
        msg: to_json_binary(&strategy_manager::ExecuteMsg::WithdrawSharesAsTokens {
            recipient: withdrawer.clone(),
            strategy: strategy.clone(),
            shares,
            token: token.clone(),
        })?,
        funds: vec![],
    };

    let response = Response::new()
        .add_message(CosmosMsg::Wasm(msg))
        .add_attribute("method", "withdraw_shares_as_tokens")
        .add_attribute("staker", staker.to_string())
        .add_attribute("withdrawer", withdrawer.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("token", token.to_string());

    Ok(response)
}