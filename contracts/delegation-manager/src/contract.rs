use crate::{
    error::ContractError,
    strategy_manager,
    msg::{InstantiateMsg, SignatureWithExpiry, QueryMsg},
    state::{
        DelegationManagerState, DELEGATION_MANAGER_STATE, OPERATOR_DETAILS, OWNER, OperatorDetails, MIN_WITHDRAWAL_DELAY_BLOCKS,
        DELEGATED_TO, STRATEGY_WITHDRAWAL_DELAY_BLOCKS, OPERATOR_SHARES, DELEGATION_APPROVER_SALT_SPENT, STRATEGY_MANAGER, SLASHER,
        STAKER_NONCE, PENDING_WITHDRAWALS, CUMULATIVE_WITHDRAWALS_QUEUED, QueuedWithdrawalParams
    },
    utils::{calculate_delegation_approval_digest_hash, calculate_staker_delegation_digest_hash, recover, 
        ApproverDigestHashParams, StakerDigestHashParams, DelegateParams, calculate_withdrawal_root, Withdrawal,
        calculate_current_staker_delegation_digest_hash, CurrentStakerDigestHashParams
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
        strategy_manager: msg.strategy_manager.clone(),
        slasher: msg.slasher.clone(),
    };

    DELEGATION_MANAGER_STATE.save(deps.storage, &state)?;
    STRATEGY_MANAGER.save(deps.storage, &msg.strategy_manager)?;
    SLASHER.save(deps.storage, &msg.slasher)?;
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
    if shares.is_zero() {
        return Err(ContractError::Underflow {});
    }

    let current_shares = OPERATOR_SHARES
        .may_load(deps.storage, (&operator, &strategy))?
        .unwrap_or_else(Uint128::zero);

    let new_shares = current_shares.checked_add(shares).map_err(|_| ContractError::Underflow)?;
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
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;

    // Calculate the withdrawal root
    let withdrawal_root = calculate_withdrawal_root(&withdrawal)?;

    // Ensure the withdrawal is pending
    if !PENDING_WITHDRAWALS.has(deps.storage, &withdrawal_root) {
        return Err(ContractError::ActionNotInQueue {});
    }

    // Ensure minWithdrawalDelayBlocks period has passed
    if withdrawal.start_block + MIN_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage)? > env.block.height {
        return Err(ContractError::MinWithdrawalDelayNotPassed {});
    }

    // Ensure only the withdrawer can complete the action
    if info.sender != withdrawal.withdrawer {
        return Err(ContractError::Unauthorized {});
    }

    // Check input length mismatch
    if receive_as_tokens && tokens.len() != withdrawal.strategies.len() {
        return Err(ContractError::InputLengthMismatch {});
    }

    // Remove the withdrawal from pending withdrawals
    PENDING_WITHDRAWALS.remove(deps.storage, &withdrawal_root);

    let mut response = Response::new();

    if receive_as_tokens {
        for (i, strategy) in withdrawal.strategies.iter().enumerate() {
            // Ensure strategyWithdrawalDelayBlocks period has passed for this strategy
            let delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage, strategy)?;
            if withdrawal.start_block + delay_blocks.u64() > env.block.height {
                return Err(ContractError::StrategyWithdrawalDelayNotPassed {});
            }

            let sub_response = _withdraw_shares_as_tokens(
                withdrawal.staker.clone(),
                info.sender.clone(),
                withdrawal.strategies[i].clone(),
                withdrawal.shares[i],
                tokens[i].clone(),
            )?;

            response = response.add_attributes(sub_response.attributes);
        }
    } else {
        let current_operator = DELEGATED_TO.may_load(deps.storage, &info.sender)?;

        for (i, strategy) in withdrawal.strategies.iter().enumerate() {
            let delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage, strategy)?;
            if withdrawal.start_block + delay_blocks.u64() > env.block.height {
                return Err(ContractError::StrategyWithdrawalDelayNotPassed {});
            }

            let msg = WasmMsg::Execute {
                contract_addr: state.strategy_manager.to_string(),
                msg: to_json_binary(&strategy_manager::ExecuteMsg::AddShares {
                    staker: info.sender.clone(),
                    token: tokens[i].clone(),
                    strategy: withdrawal.strategies[i].clone(),
                    shares: withdrawal.shares[i],
                })?,
                funds: vec![],
            };

            response = response.add_message(CosmosMsg::Wasm(msg));

            if let Some(ref operator) = current_operator {
                if operator != Addr::unchecked("0") {
                    _increase_operator_shares(
                        deps.branch(),
                        operator.clone(),
                        info.sender.clone(),
                        strategy.clone(),
                        withdrawal.shares[i],
                    )?;
                }
            }
        }
    }

    response = response.add_event(
        Event::new("WithdrawalCompleted")
            .add_attribute("withdrawal_root", withdrawal_root.to_string())
    );

    Ok(response)
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
    if staker == Addr::unchecked("0") {
        return Err(ContractError::CannotBeZero {});
    }

    if strategies.is_empty() {
        return Err(ContractError::CannotBeEmpty {});
    }

    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;

    let mut response: Response<()> = Response::new();

    for (i, strategy) in strategies.iter().enumerate() {
        let share_amount = shares[i];

        if operator != Addr::unchecked("0") {
            _decrease_operator_shares(deps.branch(), operator.clone(), staker.clone(), strategy.clone(), share_amount)?;
        }

        let forbidden: bool = deps.querier.query_wasm_smart(
            state.strategy_manager.clone(),
            &strategy_manager::QueryMsg::IsThirdPartyTransfersForbidden {
                strategy: strategy.clone(),
            },
        )?;

        if staker != withdrawer && forbidden {
            return Err(ContractError::MustBeSameAddress {});
        }

        let msg = WasmMsg::Execute {
            contract_addr: state.strategy_manager.to_string(),
            msg: to_json_binary(&strategy_manager::ExecuteMsg::RemoveShares {
                staker: staker.clone(),
                strategy: strategy.clone(),
                shares: share_amount,
            })?,
            funds: vec![],
        };

        response = response.add_message(CosmosMsg::Wasm(msg));
    }

    let nonce = CUMULATIVE_WITHDRAWALS_QUEUED.may_load(deps.storage, &staker)?.unwrap_or(0);
    let new_nonce = nonce + 1;
    CUMULATIVE_WITHDRAWALS_QUEUED.save(deps.storage, &staker, &new_nonce)?;

    let withdrawal = Withdrawal {
        staker: staker.clone(),
        delegated_to: operator.clone(),
        withdrawer: withdrawer.clone(),
        nonce,
        start_block: env.block.height,
        strategies: strategies.clone(),
        shares: shares.clone(),
    };

    let withdrawal_root = calculate_withdrawal_root(&withdrawal)?;
    PENDING_WITHDRAWALS.save(deps.storage, &withdrawal_root, &true)?;

    response.add_event(
        Event::new("WithdrawalQueued")
            .add_attribute("withdrawal_root", withdrawal_root.to_string())
            .add_attribute("staker", staker.to_string())
            .add_attribute("operator", operator.to_string())
            .add_attribute("withdrawer", withdrawer.to_string())
    );

    Ok(withdrawal_root.to_string())
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

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsDelegated { staker } => to_json_binary(&query_is_delegated(deps, staker)?),
        QueryMsg::IsOperator { operator } => to_json_binary(&query_is_operator(deps, operator)?),
        QueryMsg::OperatorDetails { operator } => to_json_binary(&query_operator_details(deps, operator)?),
        QueryMsg::DelegationApprover { operator } => to_json_binary(&query_delegation_approver(deps, operator)?),
        QueryMsg::StakerOptOutWindowBlocks { operator } => to_json_binary(&query_staker_opt_out_window_blocks(deps, operator)?),
        QueryMsg::GetOperatorShares { operator, strategies } => to_json_binary(&query_get_operator_shares(deps, operator, strategies)?),
        QueryMsg::GetDelegatableShares { staker } => to_json_binary(&get_delegatable_shares(deps, staker)?),
        QueryMsg::GetWithdrawalDelay { strategies } => to_json_binary(&query_get_withdrawal_delay(deps, strategies)?),
        QueryMsg::CalculateWithdrawalRoot { withdrawal } => to_json_binary(&calculate_withdrawal_root(&withdrawal)?),
        QueryMsg::CurrentStakerDelegationDigestHash { current_staker_digest_hash_params } => to_json_binary(&calculate_current_staker_delegation_digest_hash(current_staker_digest_hash_params)?),
        QueryMsg::StakerDelegationDigestHash { staker_digest_hash_params } => to_json_binary(&calculate_staker_delegation_digest_hash(&staker_digest_hash_params)),
        QueryMsg::DelegationApprovalDigestHash { approver_digest_hash_params } => to_json_binary(&calculate_delegation_approval_digest_hash(&approver_digest_hash_params))
    }
}

// VIEW FUNCTIONS
pub fn query_is_delegated(deps: Deps, staker: Addr) -> StdResult<bool> {
    let is_delegated = DELEGATED_TO.may_load(deps.storage, &staker)?.unwrap_or_else(|| Addr::unchecked("")) != Addr::unchecked("");
    Ok(is_delegated)
}

pub fn query_is_operator(deps: Deps, operator: Addr) -> StdResult<bool> {
    let is_operator = operator != Addr::unchecked("0") && DELEGATED_TO.may_load(deps.storage, &operator)? == Some(operator.clone());
    Ok(is_operator)
}

pub fn query_operator_details(deps: Deps, operator: Addr) -> StdResult<OperatorDetails> {
    let details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    Ok(details)
}

pub fn query_delegation_approver(deps: Deps, operator: Addr) -> StdResult<Addr> {
    let details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    Ok(details.delegation_approver)
}

pub fn query_staker_opt_out_window_blocks(deps: Deps, operator: Addr) -> StdResult<u64> {
    let details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    Ok(details.staker_opt_out_window_blocks)
}

pub fn query_get_operator_shares(deps: Deps, operator: Addr, strategies: Vec<Addr>) -> StdResult<Vec<Uint128>> {
    let mut shares = Vec::with_capacity(strategies.len());
    for strategy in strategies.iter() {
        let share = OPERATOR_SHARES.may_load(deps.storage, (&operator, strategy))?.unwrap_or_else(Uint128::zero);
        shares.push(share);
    }
    Ok(shares)
}

pub fn query_get_withdrawal_delay(deps: Deps, strategies: Vec<Addr>) -> StdResult<Uint64> {
    let min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage)?;

    let mut withdrawal_delay = min_withdrawal_delay_blocks;
    for strategy in strategies.iter() {
        let curr_withdrawal_delay = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage, strategy)?;
        if curr_withdrawal_delay > withdrawal_delay.into() {
            withdrawal_delay = curr_withdrawal_delay.into();
        }
    }

    Ok(withdrawal_delay.into())
}

pub fn query_calculate_current_staker_delegation_digest_hash(
    deps: Deps,
    env: Env,
    staker: Addr,
    operator: Addr,
    staker_public_key: Binary,
    expiry: u64
) -> StdResult<Binary> {
    let current_staker_nonce: u128 = STAKER_NONCE.may_load(deps.storage, &staker)?.unwrap_or(0);

    let params = CurrentStakerDigestHashParams {
        staker: staker.clone(),
        operator: operator.clone(),
        staker_public_key: staker_public_key.clone(), 
        expiry,
        current_nonce: current_staker_nonce,
        chain_id: env.block.chain_id.clone(),
        contract_addr: env.contract.address.clone()
    };

    calculate_current_staker_delegation_digest_hash(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{attr, Addr, Uint64, SystemResult, ContractResult, SystemError};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use sha2::{Sha256, Digest};
    use ripemd::Ripemd160;
    use bech32::{self, ToBase32, Variant};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0], attr("method", "instantiate"));
        assert_eq!(res.attributes[1], attr("strategy_manager", "strategy_manager_addr"));
        assert_eq!(res.attributes[2], attr("slasher", "slasher_addr"));
        assert_eq!(res.attributes[3], attr("min_withdrawal_delay_blocks", "100"));
        assert_eq!(res.attributes[4], attr("owner", "owner_addr"));

        let state = DELEGATION_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.strategy_manager, Addr::unchecked("strategy_manager_addr"));
        assert_eq!(state.slasher, Addr::unchecked("slasher_addr"));

        let strategy_manager = STRATEGY_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(strategy_manager, Addr::unchecked("strategy_manager_addr"));

        let slasher = SLASHER.load(&deps.storage).unwrap();
        assert_eq!(slasher, Addr::unchecked("slasher_addr"));

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, Addr::unchecked("owner_addr"));

        let min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS.load(&deps.storage).unwrap();
        assert_eq!(min_withdrawal_delay_blocks, 100);

        let withdrawal_delay_blocks1 = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(&deps.storage, &Addr::unchecked("strategy1_addr")).unwrap();
        assert_eq!(withdrawal_delay_blocks1, Uint64::from(50u64));

        let withdrawal_delay_blocks2 = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(&deps.storage, &Addr::unchecked("strategy2_addr")).unwrap();
        assert_eq!(withdrawal_delay_blocks2, Uint64::from(60u64));
    }

    #[test]
    fn test_only_owner() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let _res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        let owner_info = message_info(&Addr::unchecked("owner_addr"), &[]);

        let result = _only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);

        let result = _only_owner(deps.as_ref(), &non_owner_info);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_only_strategy_manager() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let _res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        let strategy_manager_info = message_info(&Addr::unchecked("strategy_manager_addr"), &[]);

        let result = _only_strategy_manager(deps.as_ref(), &strategy_manager_info);
        assert!(result.is_ok());

        let non_strategy_manager_info = message_info(&Addr::unchecked("not_strategy_manager"), &[]);

        let result = _only_strategy_manager(deps.as_ref(), &non_strategy_manager_info);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_min_withdrawal_delay_blocks() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let _res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        let owner_info = message_info(&Addr::unchecked("owner_addr"), &[]);

        let new_min_delay = 150;
        let result = set_min_withdrawal_delay_blocks(deps.as_mut(), owner_info, new_min_delay);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.attributes.len(), 0);
        assert_eq!(response.events.len(), 1);

        let event = &response.events[0];
        assert_eq!(event.ty, "MinWithdrawalDelayBlocksSet");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[1].key, "prev_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[1].value, "100");
        assert_eq!(event.attributes[2].key, "new_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[2].value, new_min_delay.to_string());

        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);
    
        let result = set_min_withdrawal_delay_blocks(deps.as_mut(), non_owner_info, new_min_delay);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }
    
    #[test]
    fn test_set_min_withdrawal_delay_blocks_exceeds_max() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };


        let _res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        let owner_info: MessageInfo = message_info(&Addr::unchecked("owner_addr"), &[]);
        let new_min_delay = MAX_WITHDRAWAL_DELAY_BLOCKS + 1;
        let result = set_min_withdrawal_delay_blocks(deps.as_mut(), owner_info, new_min_delay);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MinCannotBeExceedMAXWITHDRAWALDELAYBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_min_withdrawal_delay_blocks_internal() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager_addr"),
            slasher: Addr::unchecked("slasher_addr"),
            min_withdrawal_delay_blocks: 100,
            initial_owner: Addr::unchecked("owner_addr"),
            strategies: vec![Addr::unchecked("strategy1_addr"), Addr::unchecked("strategy2_addr")],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let _res = instantiate(deps.as_mut(), msg.clone()).unwrap();

        let new_min_delay = 150;
        let result = _set_min_withdrawal_delay_blocks(deps.as_mut(), new_min_delay);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.attributes.len(), 0);
        assert_eq!(response.events.len(), 1);

        let event = &response.events[0];
        assert_eq!(event.ty, "MinWithdrawalDelayBlocksSet");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[1].key, "prev_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[1].value, "100");
        assert_eq!(event.attributes[2].key, "new_min_withdrawal_delay_blocks");
        assert_eq!(event.attributes[2].value, new_min_delay.to_string());

        let new_min_delay = MAX_WITHDRAWAL_DELAY_BLOCKS + 1;
        let result = _set_min_withdrawal_delay_blocks(deps.as_mut(), new_min_delay);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MinCannotBeExceedMAXWITHDRAWALDELAYBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_strategy_withdrawal_delay_blocks() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        // Test set_strategy_withdrawal_delay_blocks
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let withdrawal_delay_blocks = vec![Uint64::new(15), Uint64::new(20)];

        let owner_info: MessageInfo = message_info(&Addr::unchecked("owner"), &[]);

        let res = set_strategy_withdrawal_delay_blocks(deps.as_mut(), owner_info.clone(), strategies.clone(), withdrawal_delay_blocks.clone()).unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(res.events[0].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[0].attributes[0].value, "strategy1");
        assert_eq!(res.events[0].attributes[1].value, "5");
        assert_eq!(res.events[0].attributes[2].value, "15");

        assert_eq!(res.events[1].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[1].attributes[0].value, "strategy2");
        assert_eq!(res.events[1].attributes[1].value, "10");
        assert_eq!(res.events[1].attributes[2].value, "20");

        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);

        let res = set_strategy_withdrawal_delay_blocks(deps.as_mut(), non_owner_info, strategies.clone(), withdrawal_delay_blocks.clone());
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let strategies = vec![Addr::unchecked("strategy1")];
        let res = set_strategy_withdrawal_delay_blocks(deps.as_mut(), owner_info.clone(), strategies, withdrawal_delay_blocks.clone());
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let strategies = vec![Addr::unchecked("strategy1")];
        let withdrawal_delay_blocks = vec![Uint64::new(MAX_WITHDRAWAL_DELAY_BLOCKS + 1)];
        let res = set_strategy_withdrawal_delay_blocks(deps.as_mut(), owner_info.clone(), strategies, withdrawal_delay_blocks);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMAXWITHDRAWALDELAYBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_strategy_withdrawal_delay_blocks_internal() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        // Test _set_strategy_withdrawal_delay_blocks
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let withdrawal_delay_blocks = vec![Uint64::new(15), Uint64::new(20)];

        let res = _set_strategy_withdrawal_delay_blocks(deps.as_mut(), strategies.clone(), withdrawal_delay_blocks.clone()).unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(res.events[0].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[0].attributes[0].value, "strategy1");
        assert_eq!(res.events[0].attributes[1].value, "5");
        assert_eq!(res.events[0].attributes[2].value, "15");

        assert_eq!(res.events[1].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[1].attributes[0].value, "strategy2");
        assert_eq!(res.events[1].attributes[1].value, "10");
        assert_eq!(res.events[1].attributes[2].value, "20");

        // Test with input length mismatch
        let strategies = vec![Addr::unchecked("strategy1")];
        let res = _set_strategy_withdrawal_delay_blocks(deps.as_mut(), strategies, withdrawal_delay_blocks.clone());
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test with delay blocks exceeding max
        let strategies = vec![Addr::unchecked("strategy1")];
        let withdrawal_delay_blocks = vec![Uint64::new(MAX_WITHDRAWAL_DELAY_BLOCKS + 1)];
        let res = _set_strategy_withdrawal_delay_blocks(deps.as_mut(), strategies, withdrawal_delay_blocks);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMAXWITHDRAWALDELAYBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_modify_operator_details() {
        let mut deps = mock_dependencies();
        let info_operator: MessageInfo = message_info(&Addr::unchecked("operator"), &[]);
        let info_delegation_approver: MessageInfo = message_info(&Addr::unchecked("approver"), &[]);
    
        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();
    
        let operator = info_operator.sender.clone();
    
        let initial_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver1"),
            delegation_approver: info_delegation_approver.sender.clone(),
            staker_opt_out_window_blocks: 100,
        };
    
        OPERATOR_DETAILS.save(deps.as_mut().storage, &operator, &initial_operator_details).unwrap();
    
        let new_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver2"),
            delegation_approver: Addr::unchecked("approver2"),
            staker_opt_out_window_blocks: 200,
        };
    
        let res = modify_operator_details(deps.as_mut(), info_operator.clone(), new_operator_details.clone()).unwrap();
    
        // Check events
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorDetailsSet");
        assert_eq!(res.events[0].attributes.len(), 2);
        assert_eq!(res.events[0].attributes[0].key, "operator");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].key, "staker_opt_out_window_blocks");
        assert_eq!(res.events[0].attributes[1].value, new_operator_details.staker_opt_out_window_blocks.to_string());
    
        // Verify the updated operator details
        let updated_details = OPERATOR_DETAILS.load(&deps.storage, &operator).unwrap();
        assert_eq!(updated_details.deprecated_earnings_receiver, new_operator_details.deprecated_earnings_receiver);
        assert_eq!(updated_details.delegation_approver, new_operator_details.delegation_approver);
        assert_eq!(updated_details.staker_opt_out_window_blocks, new_operator_details.staker_opt_out_window_blocks);
    
        // Modify operator details with staker_opt_out_window_blocks exceeding max
        let invalid_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver3"),
            delegation_approver: Addr::unchecked("approver3"),
            staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
        };
    
        let res = modify_operator_details(deps.as_mut(), info_operator.clone(), invalid_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMAXSTAKEROPTOUTWINDOWBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Modify operator details with staker_opt_out_window_blocks decreasing
        let decreasing_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver4"),
            delegation_approver: Addr::unchecked("approver4"),
            staker_opt_out_window_blocks: 50,
        };
    
        let res = modify_operator_details(deps.as_mut(), info_operator.clone(), decreasing_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeDecreased {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }    

    #[test]
    fn test_set_operator_details() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        // Initialize operator details
        let operator = Addr::unchecked("operator1");
        let initial_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver1"),
            staker_opt_out_window_blocks: 100,
            delegation_approver: Addr::unchecked("approver1"),
        };
        OPERATOR_DETAILS.save(deps.as_mut().storage, &operator, &initial_operator_details).unwrap();

        // Test setting operator details with valid data
        let new_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver2"),
            staker_opt_out_window_blocks: 200,
            delegation_approver: Addr::unchecked("approver2"),
        };

        let res = _set_operator_details(deps.as_mut(), operator.clone(), new_operator_details.clone()).unwrap();
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorDetailsSet");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, new_operator_details.staker_opt_out_window_blocks.to_string());

        // Test setting operator details with staker_opt_out_window_blocks exceeding max
        let invalid_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver3"),
            staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
            delegation_approver: Addr::unchecked("approver3"),
        };

        let res = _set_operator_details(deps.as_mut(), operator.clone(), invalid_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMAXSTAKEROPTOUTWINDOWBLOCKS {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test setting operator details with staker_opt_out_window_blocks decreasing
        let decreasing_operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver4"),
            staker_opt_out_window_blocks: 50,
            delegation_approver: Addr::unchecked("approver4"),
        };

        let res = _set_operator_details(deps.as_mut(), operator.clone(), decreasing_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeDecreased {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_increase_operator_shares() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        let operator = Addr::unchecked("operator1");
        let staker = Addr::unchecked("staker1");
        let strategy = Addr::unchecked("strategy1");
        let initial_shares = Uint128::new(100);
        OPERATOR_SHARES.save(deps.as_mut().storage, (&operator, &strategy), &initial_shares).unwrap();

        let additional_shares = Uint128::new(50);
        let res = _increase_operator_shares(deps.as_mut(), operator.clone(), staker.clone(), strategy.clone(), additional_shares).unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesIncreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(res.events[0].attributes[3].value, additional_shares.to_string());
        assert_eq!(res.events[0].attributes[4].value, (initial_shares + additional_shares).to_string());

        let stored_shares = OPERATOR_SHARES.load(deps.as_ref().storage, (&operator, &strategy)).unwrap();
        assert_eq!(stored_shares, initial_shares + additional_shares);

        let more_shares = Uint128::new(25);
        let res = _increase_operator_shares(deps.as_mut(), operator.clone(), staker.clone(), strategy.clone(), more_shares).unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesIncreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(res.events[0].attributes[3].value, more_shares.to_string());
        assert_eq!(res.events[0].attributes[4].value, (initial_shares + additional_shares + more_shares).to_string());

        let updated_shares = OPERATOR_SHARES.load(deps.as_ref().storage, (&operator, &strategy)).unwrap();
        assert_eq!(updated_shares, initial_shares + additional_shares + more_shares);

        
        let zero_shares = Uint128::new(0);
        let res = _increase_operator_shares(deps.as_mut(), operator.clone(), staker.clone(), strategy.clone(), zero_shares);

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Underflow {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_get_delegatable_shares() {
        let mut deps = mock_dependencies();
        
        // Instantiate the contract
        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        let staker = Addr::unchecked("staker1");
        
        // Mock the response from strategy_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg:_ } if contract_addr == "strategy_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&(vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")], vec![Uint128::new(100), Uint128::new(200)])).unwrap()))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        // Call get_delegatable_shares
        let (strategies, shares) = get_delegatable_shares(deps.as_ref(), staker.clone()).unwrap();

        // Verify the results
        assert_eq!(strategies.len(), 2);
        assert_eq!(shares.len(), 2);
        assert_eq!(strategies[0], Addr::unchecked("strategy1"));
        assert_eq!(shares[0], Uint128::new(100));
        assert_eq!(strategies[1], Addr::unchecked("strategy2"));
        assert_eq!(shares[1], Uint128::new(200));
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
        params: ApproverDigestHashParams,
        secret_key: &SecretKey,
    ) -> Binary {
        let message_bytes = calculate_delegation_approval_digest_hash(&params);
    
        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();
        
        Binary::from(signature_bytes)
    }

    #[test]
    fn test_delegate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_operator: MessageInfo = message_info(&Addr::unchecked("operator"), &[]);
        let info_delegation_approver: MessageInfo = message_info(&Addr::unchecked("approver"), &[]);

    
        // Instantiate the contract
        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("strategy_manager"),
            slasher: Addr::unchecked("slasher"),
            min_withdrawal_delay_blocks: 10,
            initial_owner: Addr::unchecked("owner"),
            strategies: vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")],
            withdrawal_delay_blocks: vec![5, 10],
        };
        let _res = instantiate(deps.as_mut(), msg).unwrap();

        // Register operator details
        let operator = info_operator.sender.clone();
        let operator_details = OperatorDetails {
            deprecated_earnings_receiver: Addr::unchecked("earnings_receiver"),
            delegation_approver: info_delegation_approver.sender.clone(),
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS.save(deps.as_mut().storage, &operator, &operator_details).unwrap();

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (_approver, secret_key, approver_public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);

        let staker = Addr::unchecked("staker1");
        let salt = Binary::from(vec![0]);

        let approver_public_key = Binary::from(approver_public_key_bytes.as_slice());

        let delegate_params = DelegateParams {
            staker: staker.clone(),
            operator: operator.clone(),
            public_key: approver_public_key.clone(),
            salt: salt.clone(),
        };

        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let chain_id = env.block.chain_id.clone();
        let contract_addr = env.contract.address.clone();

        let params = ApproverDigestHashParams {
            staker: staker.clone(),
            operator: operator.clone(),
            delegation_approver: info_delegation_approver.sender.clone(),
            approver_public_key: approver_public_key.clone(),
            approver_salt: salt.clone(),
            expiry,
            chain_id: chain_id.clone(),
            contract_addr: contract_addr.clone(),
        };

        let approver_signature_and_expiry = SignatureWithExpiry {
            signature: mock_signature_with_message(params.clone(), &secret_key),
            expiry,
        };

        // Mock the response from strategy_manager contract
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg: _ } if contract_addr == "strategy_manager" => {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&(vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")], vec![Uint128::new(100), Uint128::new(200)])).unwrap()))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = _delegate(deps.as_mut(), info_delegation_approver.clone(), env.clone(), approver_signature_and_expiry, delegate_params).unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "Delegate");
        assert_eq!(res.events[0].attributes[0].value, "delegate");
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, operator.to_string());
    }
}