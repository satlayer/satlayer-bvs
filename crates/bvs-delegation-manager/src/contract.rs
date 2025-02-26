#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, OperatorDetails, QueryMsg, QueuedWithdrawalParams},
    query::{
        CumulativeWithdrawalsQueuedResponse, DelegatableSharesResponse, DelegatedResponse,
        OperatorDetailsResponse, OperatorResponse, OperatorSharesResponse, OperatorStakersResponse,
        StakerOptOutWindowBlocksResponse, StakerShares, WithdrawalDelayResponse,
    },
    state::{
        DelegationManagerState, CUMULATIVE_WITHDRAWALS_QUEUED, DELEGATED_TO,
        DELEGATION_MANAGER_STATE, MIN_WITHDRAWAL_DELAY_BLOCKS, OPERATOR_DETAILS, OPERATOR_SHARES,
        OWNER, PENDING_WITHDRAWALS, STRATEGY_WITHDRAWAL_DELAY_BLOCKS,
    },
    utils::{calculate_withdrawal_root, validate_addresses, DelegateParams, Withdrawal},
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use bvs_base::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use bvs_base::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};

use bvs_strategy_manager::{
    msg::ExecuteMsg as StrategyManagerExecuteMsg, msg::QueryMsg as StrategyManagerQueryMsg,
    query::DepositsResponse, query::StakerStrategyListResponse,
    query::StakerStrategySharesResponse,
};

const CONTRACT_NAME: &str = "BVS Delegation Manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAUSED_NEW_DELEGATION: u8 = 0;
const PAUSED_ENTER_WITHDRAWAL_QUEUE: u8 = 1;
const PAUSED_EXIT_WITHDRAWAL_QUEUE: u8 = 2;

const MAX_STAKER_OPT_OUT_WINDOW_BLOCKS: u64 = 180 * 24 * 60 * 60 / 12;
const MAX_WITHDRAWAL_DELAY_BLOCKS: u64 = 216_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;
    let slash_manager = deps.api.addr_validate(&msg.slash_manager)?;
    let initial_owner = deps.api.addr_validate(&msg.initial_owner)?;

    let state = DelegationManagerState {
        strategy_manager: strategy_manager.clone(),
        slash_manager: slash_manager.clone(),
    };

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    DELEGATION_MANAGER_STATE.save(deps.storage, &state)?;
    OWNER.save(deps.storage, &initial_owner)?;
    PAUSED_STATE.save(deps.storage, &msg.initial_paused_status)?;

    set_min_withdrawal_delay_blocks_internal(deps.branch(), msg.min_withdrawal_delay_blocks)?;

    let strategies_addr = validate_addresses(deps.api, &msg.strategies)?;

    let withdrawal_delay_blocks = msg.withdrawal_delay_blocks.to_vec();

    set_strategy_withdrawal_delay_blocks_internal(
        deps.branch(),
        strategies_addr,
        withdrawal_delay_blocks,
    )?;

    let response = Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("slasher", state.slash_manager.to_string())
        .add_attribute(
            "min_withdrawal_delay_blocks",
            msg.min_withdrawal_delay_blocks.to_string(),
        )
        .add_attribute("owner", msg.initial_owner.to_string());

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAsOperator {
            operator_details,
            metadata_uri,
        } => register_as_operator(deps, info, env, operator_details, metadata_uri),
        ExecuteMsg::ModifyOperatorDetails {
            new_operator_details,
        } => modify_operator_details(deps, info, new_operator_details),
        ExecuteMsg::UpdateOperatorMetadataUri { metadata_uri } => {
            update_operator_metadata_uri(deps, info, metadata_uri)
        }
        ExecuteMsg::DelegateTo { params } => {
            let staker_addr = deps.api.addr_validate(&params.staker)?;
            let operator_addr = deps.api.addr_validate(&params.operator)?;

            let delegate_params = DelegateParams {
                staker: staker_addr.clone(),
                operator: operator_addr.clone(),
            };

            delegate_to(deps, info, env, delegate_params)
        }
        ExecuteMsg::Undelegate { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            let (mut response, withdrawal_roots) = undelegate(deps, env, info, staker_addr)?;
            for root in withdrawal_roots {
                response = response.add_attribute("withdrawal_root", root.to_base64());
            }

            Ok(response)
        }
        ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params,
        } => {
            let (response, withdrawal_roots) =
                queue_withdrawals(deps, env, info, queued_withdrawal_params)?;

            let root_strings: Vec<String> = withdrawal_roots
                .iter()
                .map(|root| root.to_base64())
                .collect();
            let response_with_roots =
                response.add_attribute("withdrawal_roots", root_strings.join(","));

            Ok(response_with_roots)
        }
        ExecuteMsg::CompleteQueuedWithdrawal {
            withdrawal,
            tokens,
            middleware_times_index,
            receive_as_tokens,
        } => complete_queued_withdrawal(
            deps,
            env,
            info,
            withdrawal,
            tokens,
            middleware_times_index,
            receive_as_tokens,
        ),
        ExecuteMsg::CompleteQueuedWithdrawals {
            withdrawals,
            tokens,
            middleware_times_indexes,
            receive_as_tokens,
        } => complete_queued_withdrawals(
            deps,
            env,
            info,
            withdrawals,
            tokens,
            middleware_times_indexes,
            receive_as_tokens,
        ),
        ExecuteMsg::IncreaseDelegatedShares {
            staker,
            strategy,
            shares,
        } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            increase_delegated_shares(deps, info, staker_addr, strategy_addr, shares)
        }
        ExecuteMsg::DecreaseDelegatedShares {
            staker,
            strategy,
            shares,
        } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            decrease_delegated_shares(deps, info, staker_addr, strategy_addr, shares)
        }
        ExecuteMsg::SetSlashManager { new_slash_manager } => {
            let new_slash_manager_addr = deps.api.addr_validate(&new_slash_manager)?;
            set_slash_manager(deps, info, new_slash_manager_addr)
        }
        ExecuteMsg::SetMinWithdrawalDelayBlocks {
            new_min_withdrawal_delay_blocks,
        } => set_min_withdrawal_delay_blocks(deps, info, new_min_withdrawal_delay_blocks),
        ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
            strategies,
            withdrawal_delay_blocks,
        } => {
            let strategies_addr = validate_addresses(deps.api, &strategies)?;

            set_strategy_withdrawal_delay_blocks(
                deps,
                info,
                strategies_addr,
                withdrawal_delay_blocks,
            )
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr: Addr = Addr::unchecked(new_owner);
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

pub fn set_slash_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_slash_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut state = DELEGATION_MANAGER_STATE.load(deps.storage)?;

    state.slash_manager = new_slash_manager.clone();
    DELEGATION_MANAGER_STATE.save(deps.storage, &state)?;

    let event = Event::new("SlashManagerSet")
        .add_attribute("method", "set_slash_manager")
        .add_attribute("new_slash_manager", new_slash_manager.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_min_withdrawal_delay_blocks(
    deps: DepsMut,
    info: MessageInfo,
    new_min_withdrawal_delay_blocks: u64,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    set_min_withdrawal_delay_blocks_internal(deps, new_min_withdrawal_delay_blocks)
}

pub fn set_strategy_withdrawal_delay_blocks(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
    withdrawal_delay_blocks: Vec<u64>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    set_strategy_withdrawal_delay_blocks_internal(deps, strategies, withdrawal_delay_blocks)
}

pub fn register_as_operator(
    mut deps: DepsMut,
    info: MessageInfo,
    env: Env,
    registering_operator_details: OperatorDetails,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let operator = info.sender.clone();

    let is_delegated_response = query_is_delegated(deps.as_ref(), operator.clone())?;
    if is_delegated_response.is_delegated {
        return Err(ContractError::StakerAlreadyDelegated {});
    }

    set_operator_details(
        deps.branch(),
        operator.clone(),
        registering_operator_details,
    )?;

    let params = DelegateParams {
        staker: info.sender.clone(),
        operator: info.sender.clone(),
    };

    delegate(deps, info, env, params)?;

    let mut response = Response::new();

    let register_event =
        Event::new("OperatorRegistered").add_attribute("operator", operator.to_string());
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

    let operator_response = query_is_operator(deps.as_ref(), operator.clone())?;
    if !operator_response.is_operator {
        return Err(ContractError::OperatorNotRegistered {});
    }

    set_operator_details(deps, operator, new_operator_details)
}

pub fn update_operator_metadata_uri(
    deps: DepsMut,
    info: MessageInfo,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    let operator = info.sender.clone();

    let operator_response = query_is_operator(deps.as_ref(), operator.clone())?;
    if !operator_response.is_operator {
        return Err(ContractError::OperatorNotRegistered {});
    }

    let mut response = Response::new();
    let metadata_event = Event::new("OperatorMetadataURIUpdated")
        .add_attribute("operator", operator.to_string())
        .add_attribute("metadata_uri", metadata_uri);
    response = response.add_event(metadata_event);

    Ok(response)
}

pub fn delegate_to(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    params: DelegateParams,
) -> Result<Response, ContractError> {
    let staker = info.sender.clone();

    let is_delegated_response = query_is_delegated(deps.as_ref(), staker.clone())?;
    if is_delegated_response.is_delegated {
        return Err(ContractError::StakerAlreadyDelegated {});
    }

    let operator_response = query_is_operator(deps.as_ref(), params.operator.clone())?;
    if !operator_response.is_operator {
        return Err(ContractError::OperatorNotRegistered {});
    }

    delegate(deps, info, env, params)
}

pub fn undelegate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: Addr,
) -> Result<(Response, Vec<Binary>), ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_ENTER_WITHDRAWAL_QUEUE)?;

    let is_delegated_response = query_is_delegated(deps.as_ref(), staker.clone())?;
    if !is_delegated_response.is_delegated {
        return Err(ContractError::StakerNotDelegated {});
    }

    let operator_response = query_is_operator(deps.as_ref(), staker.clone())?;
    if operator_response.is_operator {
        return Err(ContractError::OperatorCannotBeUndelegated {});
    }

    if staker == Addr::unchecked("0") {
        return Err(ContractError::CannotBeZero {});
    }

    let operator = DELEGATED_TO.load(deps.storage, &staker)?;

    if info.sender != staker && info.sender != operator {
        return Err(ContractError::Unauthorized {});
    }

    // Gather strategies and shares to remove from staker/operator during undelegation
    let (strategies, shares) = get_delegatable_shares(deps.as_ref(), staker.clone())?;

    let mut response = Response::new();
    let mut withdrawal_roots = Vec::new();

    // Emit an event if this action was not initiated by the staker themselves
    if info.sender != staker {
        response = response.add_event(
            Event::new("StakerForceUndelegated")
                .add_attribute("staker", staker.to_string())
                .add_attribute("operator", operator.to_string()),
        );
    }

    response = response.add_event(
        Event::new("StakerUndelegated")
            .add_attribute("staker", staker.to_string())
            .add_attribute("operator", operator.to_string()),
    );

    DELEGATED_TO.remove(deps.storage, &staker);

    if !strategies.is_empty() {
        for (strategy, share) in strategies.iter().zip(shares.iter()) {
            let single_strategy = vec![strategy.clone()];
            let single_share = vec![*share];

            let withdrawal_response = remove_shares_and_queue_withdrawal(
                deps.branch(),
                env.clone(),
                staker.clone(),
                operator.clone(),
                staker.clone(),
                single_strategy,
                single_share,
            )?;

            let withdrawal_root = withdrawal_response
                .attributes
                .iter()
                .find(|attr| attr.key == "withdrawal_root")
                .map(|attr| Binary::from_base64(&attr.value).unwrap());

            if let Some(root) = withdrawal_root {
                withdrawal_roots.push(root);
            }

            response = response.add_attributes(withdrawal_response.attributes);
            response = response.add_events(withdrawal_response.events);
            response = response.add_submessages(withdrawal_response.messages);
        }
    }

    Ok((response, withdrawal_roots))
}

pub fn get_delegatable_shares(deps: Deps, staker: Addr) -> StdResult<(Vec<Addr>, Vec<Uint128>)> {
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;
    let strategy_manager = state.strategy_manager;

    let query = WasmQuery::Smart {
        contract_addr: strategy_manager.to_string(),
        msg: to_json_binary(&StrategyManagerQueryMsg::GetDeposits {
            staker: staker.to_string(),
        })?,
    }
    .into();

    let response: DepositsResponse = deps.querier.query(&query)?;

    Ok((response.strategies, response.shares))
}

pub fn increase_delegated_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_strategy_manager(deps.as_ref(), &info)?;

    let is_delegated_response = query_is_delegated(deps.as_ref(), staker.clone())?;
    if is_delegated_response.is_delegated {
        let operator = DELEGATED_TO.load(deps.storage, &staker)?;
        increase_operator_shares(deps, operator, staker, strategy, shares)
    } else {
        Err(ContractError::NotDelegated {})
    }
}

pub fn decrease_delegated_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_strategy_manager(deps.as_ref(), &info)?;

    let is_delegated_response = query_is_delegated(deps.as_ref(), staker.clone())?;
    if is_delegated_response.is_delegated {
        let operator = DELEGATED_TO.load(deps.storage, &staker)?;
        decrease_operator_shares(deps, operator, staker, strategy, shares)
    } else {
        Err(ContractError::StakerNotDelegated {})
    }
}

pub fn queue_withdrawals(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    queued_withdrawal_params: Vec<QueuedWithdrawalParams>,
) -> Result<(Response, Vec<Binary>), ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_ENTER_WITHDRAWAL_QUEUE)?;

    let operator = DELEGATED_TO
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_else(|| Addr::unchecked(""));

    let mut response = Response::new();
    let mut withdrawal_roots = Vec::new();

    for params in queued_withdrawal_params.iter() {
        if params.strategies.len() != params.shares.len() {
            return Err(ContractError::InputLengthMismatch {});
        }
        if params.withdrawer != info.sender {
            return Err(ContractError::WithdrawerMustBeStaker {});
        }

        let withdrawal_response = remove_shares_and_queue_withdrawal(
            deps.branch(),
            env.clone(),
            info.sender.clone(),
            operator.clone(),
            params.withdrawer.clone(),
            params.strategies.clone(),
            params.shares.clone(),
        )?;

        for event in &withdrawal_response.events {
            if event.ty == "WithdrawalQueued" {
                if let Some(attr) = event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "withdrawal_root")
                {
                    let withdrawal_root = Binary::from_base64(&attr.value).unwrap();
                    withdrawal_roots.push(withdrawal_root);
                }
            }
        }

        response = response
            .add_submessages(withdrawal_response.messages)
            .add_events(withdrawal_response.events);
    }

    Ok((response, withdrawal_roots))
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
    only_when_not_paused(deps.as_ref(), PAUSED_EXIT_WITHDRAWAL_QUEUE)?;

    let mut response = Response::new();

    // Loop through each withdrawal and complete it
    for (i, withdrawal) in withdrawals.iter().enumerate() {
        let res = complete_queued_withdrawal_internal(
            deps.branch(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            tokens[i].clone(),
            middleware_times_indexes[i],
            receive_as_tokens[i],
        )?;
        response = response.add_submessages(res.messages);
        response = response.add_events(res.events);
    }

    Ok(response)
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
    only_when_not_paused(deps.as_ref(), PAUSED_EXIT_WITHDRAWAL_QUEUE)?;

    let response = complete_queued_withdrawal_internal(
        deps.branch(),
        env.clone(),
        info.clone(),
        withdrawal.clone(),
        tokens.clone(),
        middleware_times_indexe,
        receive_as_tokens,
    )?;

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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsDelegated { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            to_json_binary(&query_is_delegated(deps, staker_addr)?)
        }
        QueryMsg::IsOperator { operator } => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            to_json_binary(&query_is_operator(deps, operator_addr)?)
        }
        QueryMsg::OperatorDetails { operator } => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            to_json_binary(&query_operator_details(deps, operator_addr)?)
        }
        QueryMsg::StakerOptOutWindowBlocks { operator } => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            to_json_binary(&query_staker_opt_out_window_blocks(deps, operator_addr)?)
        }
        QueryMsg::GetOperatorShares {
            operator,
            strategies,
        } => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            let strategies_addr = validate_addresses(deps.api, &strategies)?;

            to_json_binary(&query_operator_shares(
                deps,
                operator_addr,
                strategies_addr,
            )?)
        }
        QueryMsg::GetDelegatableShares { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            to_json_binary(&query_delegatable_shares(deps, staker_addr)?)
        }
        QueryMsg::GetWithdrawalDelay { strategies } => {
            let strategies_addr = validate_addresses(deps.api, &strategies)?;
            to_json_binary(&query_withdrawal_delay(deps, strategies_addr)?)
        }
        QueryMsg::CalculateWithdrawalRoot { withdrawal } => {
            to_json_binary(&calculate_withdrawal_root(&withdrawal)?)
        }
        QueryMsg::GetOperatorStakers { operator } => {
            let operator_addr = deps.api.addr_validate(&operator)?;

            let stakers_and_shares = query_operator_stakers(deps, operator_addr)?;
            to_json_binary(&stakers_and_shares)
        }
        QueryMsg::GetCumulativeWithdrawalsQueued { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            to_json_binary(&query_cumulative_withdrawals_queued(deps, staker_addr)?)
        }
    }
}

pub fn query_delegatable_shares(deps: Deps, staker: Addr) -> StdResult<DelegatableSharesResponse> {
    let (strategies, shares) = get_delegatable_shares(deps, staker)?;

    Ok(DelegatableSharesResponse { strategies, shares })
}

pub fn query_is_delegated(deps: Deps, staker: Addr) -> StdResult<DelegatedResponse> {
    let is_delegated = DELEGATED_TO
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(|| Addr::unchecked(""))
        != Addr::unchecked("");
    Ok(DelegatedResponse { is_delegated })
}

pub fn query_is_operator(deps: Deps, operator: Addr) -> StdResult<OperatorResponse> {
    if operator == Addr::unchecked("") {
        return Ok(OperatorResponse { is_operator: false });
    }

    let delegated_to_operator = DELEGATED_TO.may_load(deps.storage, &operator)?;

    let is_operator = if let Some(stored_operator) = delegated_to_operator {
        stored_operator == operator
    } else {
        false
    };

    Ok(OperatorResponse { is_operator })
}

pub fn query_operator_details(deps: Deps, operator: Addr) -> StdResult<OperatorDetailsResponse> {
    let details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    Ok(OperatorDetailsResponse { details })
}

pub fn query_staker_opt_out_window_blocks(
    deps: Deps,
    operator: Addr,
) -> StdResult<StakerOptOutWindowBlocksResponse> {
    let details = OPERATOR_DETAILS.load(deps.storage, &operator)?;
    Ok(StakerOptOutWindowBlocksResponse {
        staker_opt_out_window_blocks: details.staker_opt_out_window_blocks,
    })
}

pub fn query_operator_shares(
    deps: Deps,
    operator: Addr,
    strategies: Vec<Addr>,
) -> StdResult<OperatorSharesResponse> {
    let mut shares = Vec::with_capacity(strategies.len());
    for strategy in strategies.iter() {
        let share = OPERATOR_SHARES
            .may_load(deps.storage, (&operator, strategy))?
            .unwrap_or_else(Uint128::zero);
        shares.push(share);
    }
    Ok(OperatorSharesResponse { shares })
}

pub fn query_withdrawal_delay(
    deps: Deps,
    strategies: Vec<Addr>,
) -> StdResult<WithdrawalDelayResponse> {
    let min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage)?;

    let mut withdrawal_delays = vec![];
    for strategy in strategies.iter() {
        let curr_withdrawal_delay =
            STRATEGY_WITHDRAWAL_DELAY_BLOCKS.may_load(deps.storage, strategy)?;
        let delay = curr_withdrawal_delay.unwrap_or(0);
        withdrawal_delays.push(std::cmp::max(delay, min_withdrawal_delay_blocks));
    }

    Ok(WithdrawalDelayResponse { withdrawal_delays })
}

pub fn query_operator_stakers(deps: Deps, operator: Addr) -> StdResult<OperatorStakersResponse> {
    let mut stakers_and_shares: Vec<StakerShares> = Vec::new();

    let stakers: Vec<Addr> = DELEGATED_TO
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (staker, delegated_operator) = item.ok()?;
            if delegated_operator == operator {
                Some(staker)
            } else {
                None
            }
        })
        .collect();

    for staker in stakers.iter() {
        let mut shares_per_strategy: Vec<(Addr, Uint128)> = Vec::new();

        let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;
        let strategy_list_response: StakerStrategyListResponse = deps.querier.query_wasm_smart(
            state.strategy_manager.clone(),
            &StrategyManagerQueryMsg::GetStakerStrategyList {
                staker: staker.to_string(),
            },
        )?;
        let strategies = strategy_list_response.strategies;

        for strategy in strategies {
            let shares_response: StakerStrategySharesResponse = deps.querier.query_wasm_smart(
                state.strategy_manager.clone(),
                &StrategyManagerQueryMsg::GetStakerStrategyShares {
                    staker: staker.to_string(),
                    strategy: strategy.to_string(),
                },
            )?;

            if !shares_response.shares.is_zero() {
                shares_per_strategy.push((strategy, shares_response.shares));
            }
        }

        if !shares_per_strategy.is_empty() {
            stakers_and_shares.push(StakerShares {
                staker: staker.clone(),
                shares_per_strategy,
            });
        }
    }

    Ok(OperatorStakersResponse { stakers_and_shares })
}

pub fn query_cumulative_withdrawals_queued(
    deps: Deps,
    staker: Addr,
) -> StdResult<CumulativeWithdrawalsQueuedResponse> {
    let cumulative_withdrawals = CUMULATIVE_WITHDRAWALS_QUEUED
        .may_load(deps.storage, &staker)?
        .unwrap_or(Uint128::new(0));

    Ok(CumulativeWithdrawalsQueuedResponse {
        cumulative_withdrawals,
    })
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn only_strategy_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;
    if info.sender != state.strategy_manager && info.sender != state.slash_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn set_min_withdrawal_delay_blocks_internal(
    deps: DepsMut,
    min_withdrawal_delay_blocks: u64,
) -> Result<Response, ContractError> {
    if min_withdrawal_delay_blocks > MAX_WITHDRAWAL_DELAY_BLOCKS {
        return Err(ContractError::MinCannotBeExceedMaxWithdrawalDelayBlocks {});
    }

    let prev_min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS
        .may_load(deps.storage)?
        .unwrap_or(0);

    MIN_WITHDRAWAL_DELAY_BLOCKS.save(deps.storage, &min_withdrawal_delay_blocks)?;

    let event = Event::new("MinWithdrawalDelayBlocksSet")
        .add_attribute("method", "set_min_withdrawal_delay_blocks")
        .add_attribute(
            "prev_min_withdrawal_delay_blocks",
            prev_min_withdrawal_delay_blocks.to_string(),
        )
        .add_attribute(
            "new_min_withdrawal_delay_blocks",
            min_withdrawal_delay_blocks.to_string(),
        );

    Ok(Response::new().add_event(event))
}

fn set_strategy_withdrawal_delay_blocks_internal(
    deps: DepsMut,
    strategies: Vec<Addr>,
    withdrawal_delay_blocks: Vec<u64>,
) -> Result<Response, ContractError> {
    if strategies.len() != withdrawal_delay_blocks.len() {
        return Err(ContractError::InputLengthMismatch {});
    }

    let mut response = Response::new();

    for (i, strategy) in strategies.iter().enumerate() {
        let new_withdrawal_delay_blocks = withdrawal_delay_blocks[i];
        if new_withdrawal_delay_blocks > MAX_WITHDRAWAL_DELAY_BLOCKS {
            return Err(ContractError::CannotBeExceedMaxWithdrawalDelayBlocks {});
        }

        let prev_withdrawal_delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .may_load(deps.storage, strategy)?
            .unwrap_or(0);

        STRATEGY_WITHDRAWAL_DELAY_BLOCKS.save(
            deps.storage,
            strategy,
            &new_withdrawal_delay_blocks,
        )?;

        let event = Event::new("StrategyWithdrawalDelayBlocksSet")
            .add_attribute("strategy", strategy.to_string())
            .add_attribute("prev", prev_withdrawal_delay_blocks.to_string())
            .add_attribute("new", new_withdrawal_delay_blocks.to_string());

        response = response.add_event(event);
    }

    Ok(response)
}

fn set_operator_details(
    deps: DepsMut,
    operator: Addr,
    updated: OperatorDetails,
) -> Result<Response, ContractError> {
    let current = OPERATOR_DETAILS
        .may_load(deps.storage, &operator)?
        .unwrap_or(OperatorDetails {
            staker_opt_out_window_blocks: 0,
        });

    if updated.staker_opt_out_window_blocks > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS {
        return Err(ContractError::CannotBeExceedMaxStakerOptOutWindowBlocks {});
    }

    if updated.staker_opt_out_window_blocks < current.staker_opt_out_window_blocks {
        return Err(ContractError::CannotBeDecreased {});
    }

    OPERATOR_DETAILS.save(deps.storage, &operator, &updated)?;

    let event = Event::new("OperatorDetailsSet")
        .add_attribute("operator", operator.to_string())
        .add_attribute(
            "staker_opt_out_window_blocks",
            updated.staker_opt_out_window_blocks.to_string(),
        );

    Ok(Response::new().add_event(event))
}

fn delegate(
    mut deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    params: DelegateParams,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_NEW_DELEGATION)?;

    DELEGATED_TO.save(deps.storage, &params.staker, &params.operator)?;

    let mut response = Response::new();

    let event = Event::new("Delegate")
        .add_attribute("method", "delegate")
        .add_attribute("staker", params.staker.to_string())
        .add_attribute("operator", params.operator.to_string());
    response = response.add_event(event);

    let (strategies, shares) = get_delegatable_shares(deps.as_ref(), params.staker.clone())?;

    for (strategy, share) in strategies.iter().zip(shares.iter()) {
        let increase_shares_response = increase_operator_shares(
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

fn increase_operator_shares(
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

    let new_shares = current_shares
        .checked_add(shares)
        .map_err(|_| ContractError::Underflow)?;
    OPERATOR_SHARES.save(deps.storage, (&operator, &strategy), &new_shares)?;

    let event = Event::new("OperatorSharesIncreased")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("new_shares", new_shares.to_string());

    Ok(Response::new().add_event(event))
}

fn decrease_operator_shares(
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

fn complete_queued_withdrawal_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdrawal: Withdrawal,
    tokens: Vec<Addr>,
    _middleware_times_index: u64,
    receive_as_tokens: bool,
) -> Result<Response, ContractError> {
    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;

    let withdrawal_root = calculate_withdrawal_root(&withdrawal)?;

    if !PENDING_WITHDRAWALS.has(deps.storage, &withdrawal_root) {
        return Err(ContractError::ActionNotInQueue {});
    }

    if withdrawal.start_block + MIN_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage)? > env.block.height {
        return Err(ContractError::MinWithdrawalDelayNotPassed {});
    }

    if info.sender != withdrawal.withdrawer {
        return Err(ContractError::Unauthorized {});
    }

    if receive_as_tokens && tokens.len() != withdrawal.strategies.len() {
        return Err(ContractError::InputLengthMismatch {});
    }

    PENDING_WITHDRAWALS.remove(deps.storage, &withdrawal_root);

    let mut response = Response::new();

    if receive_as_tokens {
        for (i, strategy) in withdrawal.strategies.iter().enumerate() {
            let delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage, strategy)?;
            if withdrawal.start_block + delay_blocks > env.block.height {
                return Err(ContractError::StrategyWithdrawalDelayNotPassed {});
            }

            let state: DelegationManagerState = DELEGATION_MANAGER_STATE.load(deps.storage)?;

            let strategy_manager = state.strategy_manager;

            let msg = WasmMsg::Execute {
                contract_addr: strategy_manager.to_string(),
                msg: to_json_binary(&StrategyManagerExecuteMsg::WithdrawSharesAsTokens {
                    recipient: info.sender.to_string(),
                    strategy: strategy.to_string(),
                    shares: withdrawal.shares[i],
                    token: tokens[i].to_string(),
                })?,
                funds: vec![],
            };

            response = response
                .add_message(CosmosMsg::Wasm(msg))
                .add_attribute("method", "withdraw_shares_as_tokens_internal")
                .add_attribute("staker", withdrawal.staker.to_string())
                .add_attribute("withdrawer", info.sender.to_string())
                .add_attribute("strategy", strategy.to_string())
                .add_attribute("shares", withdrawal.shares[i].to_string())
                .add_attribute("token", tokens[i].to_string());
        }
    } else {
        let current_operator = DELEGATED_TO.may_load(deps.storage, &info.sender)?;

        for (i, strategy) in withdrawal.strategies.iter().enumerate() {
            let delay_blocks = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.load(deps.storage, strategy)?;
            if withdrawal.start_block + delay_blocks > env.block.height {
                return Err(ContractError::StrategyWithdrawalDelayNotPassed {});
            }

            let msg = WasmMsg::Execute {
                contract_addr: state.strategy_manager.to_string(),
                msg: to_json_binary(&StrategyManagerExecuteMsg::AddShares {
                    staker: info.sender.to_string(),
                    token: tokens[i].to_string(),
                    strategy: withdrawal.strategies[i].to_string(),
                    shares: withdrawal.shares[i],
                })?,
                funds: vec![],
            };

            response = response.add_message(CosmosMsg::Wasm(msg));

            if let Some(ref operator) = current_operator {
                if operator != Addr::unchecked("0") {
                    increase_operator_shares(
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
            .add_attribute("withdrawal_root", withdrawal_root.to_string()),
    );

    Ok(response)
}

fn remove_shares_and_queue_withdrawal(
    mut deps: DepsMut,
    env: Env,
    staker: Addr,
    operator: Addr,
    withdrawer: Addr,
    strategies: Vec<Addr>,
    shares: Vec<Uint128>,
) -> Result<Response, ContractError> {
    if staker == Addr::unchecked("0") {
        return Err(ContractError::CannotBeZero {});
    }

    if strategies.is_empty() {
        return Err(ContractError::CannotBeEmpty {});
    }

    let state = DELEGATION_MANAGER_STATE.load(deps.storage)?;

    let mut response = Response::new();

    for (i, strategy) in strategies.iter().enumerate() {
        let share_amount = shares[i];

        if operator != Addr::unchecked("0") {
            decrease_operator_shares(
                deps.branch(),
                operator.clone(),
                staker.clone(),
                strategy.clone(),
                share_amount,
            )?;
        }

        let msg = WasmMsg::Execute {
            contract_addr: state.strategy_manager.to_string(),
            msg: to_json_binary(&StrategyManagerExecuteMsg::RemoveShares {
                staker: staker.to_string(),
                strategy: strategy.to_string(),
                shares: share_amount,
            })?,
            funds: vec![],
        };

        response = response.add_message(CosmosMsg::Wasm(msg));
    }

    let nonce = CUMULATIVE_WITHDRAWALS_QUEUED
        .may_load(deps.storage, &staker)?
        .unwrap_or(Uint128::new(0));
    let new_nonce = nonce + Uint128::new(1);

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

    response = response.add_event(
        Event::new("WithdrawalQueued")
            .add_attribute("withdrawal_root", withdrawal_root.to_base64())
            .add_attribute("staker", staker.to_string())
            .add_attribute("operator", operator.to_string())
            .add_attribute("withdrawer", withdrawer.to_string()),
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ExecuteDelegateParams;
    use bvs_base::roles::{PAUSER, UNPAUSER};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, from_json, Addr, ContractResult, OwnedDeps, SystemError, SystemResult,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let slasher = deps.api.addr_make("slasher").to_string();
        let initial_owner = deps.api.addr_make("initial_owner").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();
        let strategy1 = deps.api.addr_make("strategy1").to_string();
        let strategy2 = deps.api.addr_make("strategy2").to_string();

        let msg = InstantiateMsg {
            strategy_manager: strategy_manager.clone(),
            slash_manager: slasher.clone(),
            min_withdrawal_delay_blocks: 100,
            initial_owner: initial_owner.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
            strategies: vec![strategy1.clone(), strategy2.clone()],
            withdrawal_delay_blocks: vec![50, 60],
        };

        let res = instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0], attr("method", "instantiate"));
        assert_eq!(
            res.attributes[1],
            attr("strategy_manager", strategy_manager.clone())
        );
        assert_eq!(res.attributes[2], attr("slasher", slasher.clone()));
        assert_eq!(
            res.attributes[3],
            attr("min_withdrawal_delay_blocks", "100")
        );
        assert_eq!(res.attributes[4], attr("owner", initial_owner.clone()));

        let state = DELEGATION_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(
            state.strategy_manager,
            Addr::unchecked(strategy_manager.clone())
        );
        assert_eq!(state.slash_manager, Addr::unchecked(slasher));

        let state = DELEGATION_MANAGER_STATE.load(&deps.storage).unwrap();

        assert_eq!(Addr::unchecked(strategy_manager), state.strategy_manager);

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, Addr::unchecked(initial_owner.clone()));

        let min_withdrawal_delay_blocks = MIN_WITHDRAWAL_DELAY_BLOCKS.load(&deps.storage).unwrap();
        assert_eq!(min_withdrawal_delay_blocks, 100);

        let withdrawal_delay_blocks1 = STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .load(&deps.storage, &Addr::unchecked(strategy1.clone()))
            .unwrap();
        assert_eq!(withdrawal_delay_blocks1, 50);

        let withdrawal_delay_blocks2 = STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .load(&deps.storage, &Addr::unchecked(strategy2.clone()))
            .unwrap();
        assert_eq!(withdrawal_delay_blocks2, 60);

        let paused_status = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_status, 0);
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner").to_string();
        let owner_info = message_info(&Addr::unchecked(owner.clone()), &[]);

        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let slasher = deps.api.addr_make("slasher").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();
        let strategy1 = deps.api.addr_make("strategy1").to_string();
        let strategy2 = deps.api.addr_make("strategy2").to_string();

        let pauser_info = message_info(&Addr::unchecked(pauser.clone()), &[]);
        let unpauser_info = message_info(&Addr::unchecked(unpauser.clone()), &[]);
        let strategy_manager_info = message_info(&Addr::unchecked(strategy_manager.clone()), &[]);

        let msg = InstantiateMsg {
            strategy_manager: strategy_manager.clone(),
            slash_manager: slasher.clone(),
            min_withdrawal_delay_blocks: 100,
            initial_owner: owner.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
            strategies: vec![strategy1.clone(), strategy2.clone()],
            withdrawal_delay_blocks: vec![50, 60],
        };

        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        (
            deps,
            env,
            owner_info,
            pauser_info,
            unpauser_info,
            strategy_manager_info,
        )
    }

    #[test]
    fn test_only_owner() {
        let (deps, _env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let result = only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);

        let result = only_owner(deps.as_ref(), &non_owner_info);
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
        let (deps, _env, _owner_info, _pauser_info, _unpauser_info, strategy_manager_info) =
            instantiate_contract();

        let result = only_strategy_manager(deps.as_ref(), &strategy_manager_info);
        assert!(result.is_ok());

        let non_strategy_manager_info = message_info(&Addr::unchecked("not_strategy_manager"), &[]);

        let result = only_strategy_manager(deps.as_ref(), &non_strategy_manager_info);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_slash_manager() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_slash_manager = deps.api.addr_make("new_slash_manager").to_string();
        let execute_msg = ExecuteMsg::SetSlashManager {
            new_slash_manager: new_slash_manager.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), execute_msg).unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "SlashManagerSet");
        assert_eq!(res.events[0].attributes[0].key, "method");
        assert_eq!(res.events[0].attributes[0].value, "set_slash_manager");
        assert_eq!(res.events[0].attributes[1].key, "new_slash_manager");
        assert_eq!(res.events[0].attributes[1].value, new_slash_manager.clone());

        let state = DELEGATION_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.slash_manager, Addr::unchecked(new_slash_manager));
    }

    #[test]
    fn test_set_min_withdrawal_delay_blocks() {
        let (mut deps, _env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

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
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_min_delay = MAX_WITHDRAWAL_DELAY_BLOCKS + 1;

        let execute_msg = ExecuteMsg::SetMinWithdrawalDelayBlocks {
            new_min_withdrawal_delay_blocks: new_min_delay,
        };

        let result = execute(deps.as_mut(), env, owner_info, execute_msg);

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MinCannotBeExceedMaxWithdrawalDelayBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_min_withdrawal_delay_blocks_internal() {
        let (mut deps, _env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_min_delay = 150;
        let result =
            set_min_withdrawal_delay_blocks(deps.as_mut(), owner_info.clone(), new_min_delay);
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
        let result =
            set_min_withdrawal_delay_blocks(deps.as_mut(), owner_info.clone(), new_min_delay);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MinCannotBeExceedMaxWithdrawalDelayBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_strategy_withdrawal_delay_blocks() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let strategy1 = deps.api.addr_make("strategy1").to_string();
        let strategy2 = deps.api.addr_make("strategy2").to_string();

        // Test set_strategy_withdrawal_delay_blocks
        let strategies_str = vec![strategy1.clone(), strategy2.clone()];

        let withdrawal_delay_blocks = vec![15, 20];

        let execute_msg = ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
            strategies: strategies_str.clone(),
            withdrawal_delay_blocks: withdrawal_delay_blocks.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), execute_msg).unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(res.events[0].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[0].attributes[0].value, strategy1.clone());
        assert_eq!(res.events[0].attributes[1].value, "50");
        assert_eq!(res.events[0].attributes[2].value, "15");

        assert_eq!(res.events[1].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[1].attributes[0].value, strategy2.clone());
        assert_eq!(res.events[1].attributes[1].value, "60");
        assert_eq!(res.events[1].attributes[2].value, "20");

        // Test unauthorized attempt
        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            non_owner_info,
            ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
                strategies: strategies_str.clone(),
                withdrawal_delay_blocks: withdrawal_delay_blocks.clone(),
            },
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test input length mismatch error
        let strategies = vec![deps.api.addr_make("strategy1").to_string()];
        let res = execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
                strategies: strategies.clone(),
                withdrawal_delay_blocks: withdrawal_delay_blocks.clone(),
            },
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test exceeding max withdrawal delay blocks
        let strategies = vec![deps.api.addr_make("strategy1").to_string()];
        let withdrawal_delay_blocks = vec![MAX_WITHDRAWAL_DELAY_BLOCKS + 1];
        let res = execute(
            deps.as_mut(),
            env,
            owner_info.clone(),
            ExecuteMsg::SetStrategyWithdrawalDelayBlocks {
                strategies: strategies.clone(),
                withdrawal_delay_blocks: withdrawal_delay_blocks.clone(),
            },
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMaxWithdrawalDelayBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_strategy_withdrawal_delay_blocks_internal() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        let strategies = vec![strategy1.clone(), strategy2.clone()];
        let withdrawal_delay_blocks = vec![15, 20];

        let res = set_strategy_withdrawal_delay_blocks_internal(
            deps.as_mut(),
            strategies.clone(),
            withdrawal_delay_blocks.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(res.events[0].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[0].attributes[0].value, strategy1.to_string());
        assert_eq!(res.events[0].attributes[1].value, "50");
        assert_eq!(res.events[0].attributes[2].value, "15");

        assert_eq!(res.events[1].ty, "StrategyWithdrawalDelayBlocksSet");
        assert_eq!(res.events[1].attributes[0].value, strategy2.to_string());
        assert_eq!(res.events[1].attributes[1].value, "60");
        assert_eq!(res.events[1].attributes[2].value, "20");

        // Test with input length mismatch
        let strategies = vec![deps.api.addr_make("strategy1")];
        let res = set_strategy_withdrawal_delay_blocks_internal(
            deps.as_mut(),
            strategies,
            withdrawal_delay_blocks.clone(),
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test with delay blocks exceeding max
        let strategies = vec![deps.api.addr_make("strategy1")];
        let withdrawal_delay_blocks = vec![MAX_WITHDRAWAL_DELAY_BLOCKS + 1];
        let res = set_strategy_withdrawal_delay_blocks_internal(
            deps.as_mut(),
            strategies,
            withdrawal_delay_blocks,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMaxWithdrawalDelayBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_modify_operator_details() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator");
        let earnings_receiver1 = deps.api.addr_make("earnings_receiver1");

        let info_operator = message_info(&Addr::unchecked(operator.clone()), &[]);

        DELEGATED_TO
            .save(deps.as_mut().storage, &operator.clone(), &operator)
            .unwrap();

        let initial_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &initial_operator_details)
            .unwrap();

        let new_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 200,
        };

        let modify_msg = ExecuteMsg::ModifyOperatorDetails {
            new_operator_details: new_operator_details.clone(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_operator.clone(),
            modify_msg,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorDetailsSet");
        assert_eq!(res.events[0].attributes.len(), 2);
        assert_eq!(res.events[0].attributes[0].key, "operator");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(
            res.events[0].attributes[1].key,
            "staker_opt_out_window_blocks"
        );
        assert_eq!(
            res.events[0].attributes[1].value,
            new_operator_details
                .staker_opt_out_window_blocks
                .to_string()
        );

        // Verify the updated operator details
        let updated_details = OPERATOR_DETAILS.load(&deps.storage, &operator).unwrap();
        assert_eq!(
            updated_details.staker_opt_out_window_blocks,
            new_operator_details.staker_opt_out_window_blocks
        );

        // Modify operator details with staker_opt_out_window_blocks exceeding max
        let invalid_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
        };

        let modify_msg = ExecuteMsg::ModifyOperatorDetails {
            new_operator_details: invalid_operator_details,
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_operator.clone(),
            modify_msg,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMaxStakerOptOutWindowBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Modify operator details with staker_opt_out_window_blocks decreasing
        let decreasing_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 50,
        };

        let modify_msg = ExecuteMsg::ModifyOperatorDetails {
            new_operator_details: decreasing_operator_details,
        };
        let res = execute(deps.as_mut(), env, info_operator, modify_msg);
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
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let initial_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &initial_operator_details)
            .unwrap();

        let new_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 200,
        };

        let res = set_operator_details(
            deps.as_mut(),
            operator.clone(),
            new_operator_details.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorDetailsSet");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(
            res.events[0].attributes[1].value,
            new_operator_details
                .staker_opt_out_window_blocks
                .to_string()
        );

        let invalid_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
        };

        let res = set_operator_details(deps.as_mut(), operator.clone(), invalid_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeExceedMaxStakerOptOutWindowBlocks {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let decreasing_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 50,
        };

        let res = set_operator_details(deps.as_mut(), operator, decreasing_operator_details);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::CannotBeDecreased {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_increase_operator_shares_internal() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let staker = deps.api.addr_make("staker1");
        let strategy = deps.api.addr_make("strategy1");
        let initial_shares = Uint128::new(100);
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategy),
                &initial_shares,
            )
            .unwrap();

        let additional_shares = Uint128::new(50);
        let res = increase_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            additional_shares,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesIncreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(
            res.events[0].attributes[3].value,
            additional_shares.to_string()
        );
        assert_eq!(
            res.events[0].attributes[4].value,
            (initial_shares + additional_shares).to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, initial_shares + additional_shares);

        let more_shares = Uint128::new(25);
        let res = increase_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            more_shares,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesIncreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(res.events[0].attributes[3].value, more_shares.to_string());
        assert_eq!(
            res.events[0].attributes[4].value,
            (initial_shares + additional_shares + more_shares).to_string()
        );

        let updated_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(
            updated_shares,
            initial_shares + additional_shares + more_shares
        );

        let zero_shares = Uint128::new(0);
        let res = increase_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            zero_shares,
        );

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
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&DepositsResponse {
                        strategies: vec![
                            deps.api.addr_make("strategy1"),
                            deps.api.addr_make("strategy2"),
                        ],
                        shares: vec![Uint128::new(100), Uint128::new(200)],
                    })
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let query_msg = QueryMsg::GetDelegatableShares {
            staker: staker.to_string(),
        };

        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let delegatable_shares: DelegatableSharesResponse = from_json(res).unwrap();

        assert_eq!(delegatable_shares.strategies.len(), 2);
        assert_eq!(delegatable_shares.shares.len(), 2);
        assert_eq!(
            delegatable_shares.strategies[0],
            deps.api.addr_make("strategy1")
        );
        assert_eq!(delegatable_shares.shares[0], Uint128::new(100));
        assert_eq!(
            delegatable_shares.strategies[1],
            deps.api.addr_make("strategy2")
        );
        assert_eq!(delegatable_shares.shares[1], Uint128::new(200));
    }

    #[test]
    fn test_delegate() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let operator = deps.api.addr_make("operator");

        let operator = operator.clone();
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        let delegate_params = DelegateParams {
            staker: staker.clone(),
            operator: operator.clone(),
        };

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&DepositsResponse {
                        strategies: vec![
                            deps.api.addr_make("strategy1"),
                            deps.api.addr_make("strategy2"),
                        ],
                        shares: vec![Uint128::new(100), Uint128::new(200)],
                    })
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = delegate(
            deps.as_mut(),
            owner_info.clone(),
            env.clone(),
            delegate_params,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "Delegate");
        assert_eq!(res.events[0].attributes[0].value, "delegate");
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, operator.to_string());
    }

    #[test]
    fn test_delegate_to() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker: Addr = deps.api.addr_make("staker");
        let operator: Addr = deps.api.addr_make("operator");

        let operator = operator.clone();
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        DELEGATED_TO
            .save(deps.as_mut().storage, &operator, &operator)
            .unwrap();

        let delegate_params = ExecuteDelegateParams {
            staker: staker.to_string(),
            operator: operator.to_string(),
        };

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&DepositsResponse {
                        strategies: vec![
                            deps.api.addr_make("strategy1"),
                            deps.api.addr_make("strategy2"),
                        ],
                        shares: vec![Uint128::new(100), Uint128::new(200)],
                    })
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let execute_msg = ExecuteMsg::DelegateTo {
            params: delegate_params.clone(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        assert_eq!(event.ty, "Delegate");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "delegate");
        assert_eq!(event.attributes[1].key, "staker");
        assert_eq!(event.attributes[1].value, staker.to_string());
        assert_eq!(event.attributes[2].key, "operator");
        assert_eq!(event.attributes[2].value, operator.to_string());

        let delegated_to = DELEGATED_TO.load(&deps.storage, &staker).unwrap();
        assert_eq!(delegated_to, operator);
    }

    #[test]
    fn test_register_as_operator() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&DepositsResponse {
                        strategies: vec![
                            deps.api.addr_make("strategy1"),
                            deps.api.addr_make("strategy2"),
                        ],
                        shares: vec![Uint128::new(100), Uint128::new(200)],
                    })
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        let sender_addr = deps.api.addr_make("sender");

        let info_operator = MessageInfo {
            sender: sender_addr.clone(),
            funds: vec![],
        };

        let metadata_uri = "https://example.com/metadata";
        let register_msg = ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_operator.clone(),
            register_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 2);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorRegistered");
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(event.attributes[0].key, "operator");
        assert_eq!(event.attributes[0].value, info_operator.sender.to_string());

        let event = &res.events[1];
        assert_eq!(event.ty, "OperatorMetadataURIUpdated");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "operator");
        assert_eq!(event.attributes[0].value, info_operator.sender.to_string());
        assert_eq!(event.attributes[1].key, "metadata_uri");
        assert_eq!(event.attributes[1].value, metadata_uri.to_string());

        let stored_operator_details = OPERATOR_DETAILS
            .load(&deps.storage, &info_operator.sender)
            .unwrap();
        assert_eq!(
            stored_operator_details.staker_opt_out_window_blocks,
            operator_details.staker_opt_out_window_blocks
        );

        let delegated_to = DELEGATED_TO
            .load(&deps.storage, &info_operator.sender)
            .unwrap();
        assert_eq!(delegated_to, info_operator.sender);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_operator.clone(),
            register_msg.clone(),
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::StakerAlreadyDelegated {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_update_operator_metadata_uri() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        DELEGATED_TO
            .save(deps.as_mut().storage, &operator, &operator)
            .unwrap();

        let info_operator: MessageInfo = message_info(&Addr::unchecked(operator), &[]);

        let metadata_uri = "https://example.com/metadata";
        let update_msg = ExecuteMsg::UpdateOperatorMetadataUri {
            metadata_uri: metadata_uri.to_string(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_operator.clone(),
            update_msg,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorMetadataURIUpdated");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "operator");
        assert_eq!(event.attributes[0].value, info_operator.sender.to_string());
        assert_eq!(event.attributes[1].key, "metadata_uri");
        assert_eq!(event.attributes[1].value, metadata_uri.to_string());

        // Check for an operator not registered error
        let info_non_operator: MessageInfo = message_info(&Addr::unchecked("non_operator"), &[]);

        let update_msg = ExecuteMsg::UpdateOperatorMetadataUri {
            metadata_uri: metadata_uri.to_string(),
        };
        let res = execute(deps.as_mut(), env, info_non_operator, update_msg);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::OperatorNotRegistered {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_increase_delegated_shares() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let strategy = deps.api.addr_make("strategy1");
        let initial_shares = Uint128::new(100);

        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategy),
                &initial_shares,
            )
            .unwrap();

        // Test increasing shares
        let additional_shares = Uint128::new(50);
        let increase_msg = ExecuteMsg::IncreaseDelegatedShares {
            staker: staker.to_string(),
            strategy: strategy.to_string(),
            shares: additional_shares,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            increase_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        assert_eq!(event.ty, "OperatorSharesIncreased");
        assert_eq!(event.attributes[0].value, operator.to_string());
        assert_eq!(event.attributes[1].value, staker.to_string());
        assert_eq!(event.attributes[2].value, strategy.to_string());
        assert_eq!(event.attributes[3].value, additional_shares.to_string());
        assert_eq!(
            event.attributes[4].value,
            (initial_shares + additional_shares).to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, initial_shares + additional_shares);

        // Test unauthorized increase (should fail)
        let unauthorized_info = message_info(&Addr::unchecked("not_strategy_manager"), &[]);

        let res_unauthorized = execute(
            deps.as_mut(),
            env.clone(),
            unauthorized_info,
            increase_msg.clone(),
        );
        assert!(res_unauthorized.is_err());
        if let Err(err) = res_unauthorized {
            assert!(matches!(err, ContractError::Unauthorized {}));
        }

        // Test increase when staker is not delegated (should return an empty response)
        let non_delegated_staker = deps.api.addr_make("staker2");
        let strategy = deps.api.addr_make("stratey1");
        let additional_shares = Uint128::new(50);
        let increase_msg_non_delegated = ExecuteMsg::IncreaseDelegatedShares {
            staker: non_delegated_staker.to_string(),
            strategy: strategy.to_string(),
            shares: additional_shares,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            increase_msg_non_delegated,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            assert!(matches!(err, ContractError::NotDelegated {}));
        }
    }

    #[test]
    fn test_decrease_operator_shares_internal() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let staker = deps.api.addr_make("staker1");
        let strategy = deps.api.addr_make("strategy1");
        let initial_shares = Uint128::new(100);
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategy),
                &initial_shares,
            )
            .unwrap();

        let decrease_shares = Uint128::new(50);
        let res = decrease_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            decrease_shares,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesDecreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(
            res.events[0].attributes[3].value,
            decrease_shares.to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, initial_shares - decrease_shares);

        // Test decreasing shares with amount greater than current shares (should error)
        let excess_decrease = Uint128::new(60);
        let res = decrease_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            excess_decrease,
        );

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Underflow {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test decreasing shares to zero
        let res = decrease_operator_shares(
            deps.as_mut(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            decrease_shares,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesDecreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(
            res.events[0].attributes[3].value,
            decrease_shares.to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, Uint128::new(0));
    }

    #[test]
    fn test_decrease_delegated_shares() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let strategy = deps.api.addr_make("strategy1");
        let initial_shares = Uint128::new(100);

        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategy),
                &initial_shares,
            )
            .unwrap();

        // Test decreasing shares
        let decrease_shares = Uint128::new(50);
        let decrease_msg = ExecuteMsg::DecreaseDelegatedShares {
            staker: staker.to_string(),
            strategy: strategy.to_string(),
            shares: decrease_shares,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            decrease_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesDecreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(
            res.events[0].attributes[3].value,
            decrease_shares.to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, initial_shares - decrease_shares);

        // Test decreasing shares with amount greater than current shares (should error)
        let excess_decrease = Uint128::new(60);
        let decrease_msg_excess = ExecuteMsg::DecreaseDelegatedShares {
            staker: staker.to_string(),
            strategy: strategy.to_string(),
            shares: excess_decrease,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            decrease_msg_excess.clone(),
        );

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Underflow {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test decreasing shares to zero
        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            decrease_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "OperatorSharesDecreased");
        assert_eq!(res.events[0].attributes[0].value, operator.to_string());
        assert_eq!(res.events[0].attributes[1].value, staker.to_string());
        assert_eq!(res.events[0].attributes[2].value, strategy.to_string());
        assert_eq!(
            res.events[0].attributes[3].value,
            decrease_shares.to_string()
        );

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategy))
            .unwrap();
        assert_eq!(stored_shares, Uint128::new(0));

        // Test non-strategy manager attempt to decrease shares (should error)
        let non_strategy_manager_info = message_info(&Addr::unchecked("not_strategy_manager"), &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            non_strategy_manager_info.clone(),
            decrease_msg.clone(),
        );

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test staker not delegated (should error)
        let new_staker = deps.api.addr_make("staker2");
        let decrease_msg_non_delegated = ExecuteMsg::DecreaseDelegatedShares {
            staker: new_staker.to_string(),
            strategy: strategy.to_string(),
            shares: decrease_shares,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            strategy_manager_info.clone(),
            decrease_msg_non_delegated.clone(),
        );

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::StakerNotDelegated {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_remove_shares_and_queue_withdrawal() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let withdrawer = staker.clone();
        let strategies = vec![deps.api.addr_make("strategy1")];
        let shares = vec![Uint128::new(100)];

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategies[0]),
                &shares[0],
            )
            .unwrap();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == deps.api.addr_make("strategy_manager").to_string() =>
            {
                let query_msg: Result<StrategyManagerQueryMsg, _> = from_json(msg);
                if let Ok(StrategyManagerQueryMsg::GetDeposits { staker: _ }) = query_msg {
                    let simulated_response = DepositsResponse {
                        strategies: vec![deps.api.addr_make("strategy1")],
                        shares: vec![Uint128::new(100)],
                    };
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&simulated_response).unwrap(),
                    ))
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = remove_shares_and_queue_withdrawal(
            deps.as_mut(),
            env.clone(),
            staker.clone(),
            operator.clone(),
            withdrawer.clone(),
            strategies.clone(),
            shares.clone(),
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "WithdrawalQueued");
        assert_eq!(events[0].attributes[0].key, "withdrawal_root");
        assert_eq!(events[0].attributes[1].key, "staker");
        assert_eq!(events[0].attributes[2].key, "operator");
        assert_eq!(events[0].attributes[3].key, "withdrawer");

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategies[0]))
            .unwrap();
        assert_eq!(stored_shares, Uint128::zero());

        let withdrawal_root_base64 = events[0].attributes[0].value.clone();
        let withdrawal_root_bytes = Binary::from_base64(&withdrawal_root_base64).unwrap();
        let pending_withdrawal_exists =
            PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root_bytes);
        assert!(pending_withdrawal_exists);
    }

    #[test]
    fn test_calculate_withdrawal_root() {
        let (deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let delegated_to = deps.api.addr_make("operator");
        let withdrawer = deps.api.addr_make("withdrawer");
        let strategies = vec![
            deps.api.addr_make("strategy1"),
            deps.api.addr_make("strategy2"),
        ];
        let shares = vec![Uint128::new(100), Uint128::new(200)];

        let withdrawal = Withdrawal {
            staker: staker.clone(),
            delegated_to: delegated_to.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(1),
            start_block: 12345,
            strategies: strategies.clone(),
            shares: shares.clone(),
        };

        let query_msg = QueryMsg::CalculateWithdrawalRoot {
            withdrawal: withdrawal.clone(),
        };

        let res: String = from_json(query(deps.as_ref(), env, query_msg).unwrap()).unwrap();

        let expected_hash = "5iYF5vxKZ9YCauoTabLxzUs45D9WQD8+IBXBVrjAZYg=";
        assert_eq!(res, expected_hash);
    }

    #[test]
    fn test_undelegate() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let strategies = [deps.api.addr_make("strategy1")];
        let shares = [Uint128::new(100)];

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategies[0]),
                &shares[0],
            )
            .unwrap();
        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == deps.api.addr_make("strategy_manager").to_string() =>
            {
                let query_msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    StrategyManagerQueryMsg::GetDeposits { staker: _ } => {
                        let simulated_response = DepositsResponse {
                            strategies: vec![deps.api.addr_make("strategy1")],
                            shares: vec![Uint128::new(100)],
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&simulated_response).unwrap(),
                        ))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let info = message_info(&staker.clone(), &[]);

        let undelegate_msg = ExecuteMsg::Undelegate {
            staker: staker.to_string(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            undelegate_msg.clone(),
        )
        .unwrap();

        let events = res.events.clone();
        assert!(events.iter().any(|e| e.ty == "WithdrawalQueued"));
        assert!(events.iter().any(|e| e.ty == "StakerUndelegated"));

        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategies[0]))
            .unwrap();
        assert_eq!(stored_shares, Uint128::zero());

        for withdrawal_root in &res.attributes {
            if withdrawal_root.key == "withdrawal_root" {
                let withdrawal_root_bytes = Binary::from_base64(&withdrawal_root.value).unwrap();
                assert!(PENDING_WITHDRAWALS
                    .has(deps.as_ref().storage, withdrawal_root_bytes.as_slice()));
            }
        }

        // Test unauthorized call
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategies[0]),
                &shares[0],
            )
            .unwrap();
        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();

        let unauthorized_info = message_info(&Addr::unchecked("not_authorized"), &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            unauthorized_info,
            undelegate_msg.clone(),
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test undelegating a non-delegated staker
        let non_delegated_staker = deps.api.addr_make("staker2");
        let undelegate_msg_non_delegated = ExecuteMsg::Undelegate {
            staker: non_delegated_staker.to_string(),
        };
        let info = message_info(&non_delegated_staker, &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            undelegate_msg_non_delegated,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::StakerNotDelegated {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test staker as operator
        let operator_staker = deps.api.addr_make("operator_staker");
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator_staker, &operator_details)
            .unwrap();
        DELEGATED_TO
            .save(deps.as_mut().storage, &operator_staker, &operator_staker)
            .unwrap();

        let undelegate_msg_operator = ExecuteMsg::Undelegate {
            staker: operator_staker.to_string(),
        };
        let info = message_info(&operator_staker, &[]);
        let res = execute(deps.as_mut(), env.clone(), info, undelegate_msg_operator);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::OperatorCannotBeUndelegated {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_queue_withdrawals() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let withdrawer = staker.clone();
        let strategies = vec![deps.api.addr_make("strategy1")];
        let shares = vec![Uint128::new(100)];

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &strategies[0]),
                &shares[0],
            )
            .unwrap();
        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == deps.api.addr_make("strategy_manager").to_string() =>
            {
                let query_msg: Result<StrategyManagerQueryMsg, _> = from_json(msg);
                if let Ok(StrategyManagerQueryMsg::GetDeposits { staker: _ }) = query_msg {
                    let simulated_response = DepositsResponse {
                        strategies: vec![deps.api.addr_make("strategy1")],
                        shares: vec![Uint128::new(100)],
                    };
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&simulated_response).unwrap(),
                    ))
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let queued_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: withdrawer.clone(),
            strategies: strategies.clone(),
            shares: shares.clone(),
        }];

        let info = message_info(&staker, &[]);

        let queue_withdrawals_msg = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params: queued_withdrawal_params.clone(),
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            queue_withdrawals_msg.clone(),
        )
        .unwrap();

        // Verify withdrawal roots
        let events = res.events.clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "WithdrawalQueued");
        assert_eq!(events[0].attributes[1].value, staker.to_string());
        assert_eq!(events[0].attributes[2].value, operator.to_string());
        assert_eq!(events[0].attributes[3].value, withdrawer.to_string());

        // Verify the state changes
        let stored_shares = OPERATOR_SHARES
            .load(deps.as_ref().storage, (&operator, &strategies[0]))
            .unwrap();
        assert_eq!(stored_shares, Uint128::zero());

        let withdrawal_root_base64 = events[0].attributes[0].value.clone();
        let withdrawal_root_bytes = Binary::from_base64(&withdrawal_root_base64).unwrap();
        let pending_withdrawal_exists =
            PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root_bytes);
        assert!(pending_withdrawal_exists);

        // Test input length mismatch error
        let invalid_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: withdrawer.clone(),
            strategies: strategies.clone(),
            shares: vec![Uint128::new(100), Uint128::new(200)],
        }];
        let queue_withdrawals_msg_invalid = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params: invalid_withdrawal_params,
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            queue_withdrawals_msg_invalid,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test withdrawer is not the staker
        let invalid_withdrawal_params = vec![QueuedWithdrawalParams {
            withdrawer: deps.api.addr_make("other_address"),
            strategies: strategies.clone(),
            shares: shares.clone(),
        }];
        let queue_withdrawals_msg_invalid_withdrawer = ExecuteMsg::QueueWithdrawals {
            queued_withdrawal_params: invalid_withdrawal_params,
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            queue_withdrawals_msg_invalid_withdrawer,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::WithdrawerMustBeStaker {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_complete_queued_withdrawal_internal() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let withdrawer = staker.clone();
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");
        let tokens = vec![deps.api.addr_make("token1"), deps.api.addr_make("token2")];
        let shares = vec![Uint128::new(100), Uint128::new(200)];
        let strategies = vec![strategy1.clone(), strategy2.clone()];

        DELEGATED_TO
            .save(deps.as_mut().storage, &withdrawer, &operator)
            .unwrap();

        let withdrawal = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(0),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies.clone(),
            shares: shares.clone(),
        };

        let withdrawal_root = calculate_withdrawal_root(&withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root, &true)
            .unwrap();

        let strategy1_clone = strategy1.clone();
        let strategy2_clone = strategy2.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&(
                        vec![strategy1_clone.clone(), strategy2_clone.clone()],
                        vec![Uint128::new(100), Uint128::new(200)],
                    ))
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let info = message_info(&withdrawer, &[]);

        let res = complete_queued_withdrawal_internal(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            tokens.clone(),
            0,
            true,
        )
        .unwrap();

        assert_eq!(res.events.len(), 1); // 2 withdrawals as tokens + 1 completion
        assert_eq!(res.events[0].ty, "WithdrawalCompleted");

        assert_eq!(
            res.events[0].attributes[0].value,
            withdrawal_root.to_string()
        );

        assert!(
            res.events[0]
                .attributes
                .iter()
                .any(|attr| attr.key == "withdrawal_root"
                    && attr.value == withdrawal_root.to_string())
        );

        // Verify state changes: the pending withdrawal should be removed
        assert!(!PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root));

        // Test for unauthorized attempt to complete
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root, &true)
            .unwrap();

        let unauthorized_info = message_info(&Addr::unchecked("not_authorized"), &[]);
        let res = complete_queued_withdrawal_internal(
            deps.as_mut(),
            env.clone(),
            unauthorized_info,
            withdrawal.clone(),
            tokens.clone(),
            0,
            true,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for insufficient delay
        let premature_withdrawal = Withdrawal {
            start_block: env.block.height - 5, // Not enough delay
            ..withdrawal.clone()
        };
        let premature_withdrawal_root: Binary =
            calculate_withdrawal_root(&premature_withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &premature_withdrawal_root, &true)
            .unwrap();
        let res = complete_queued_withdrawal_internal(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            premature_withdrawal.clone(),
            tokens.clone(),
            0,
            true,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::MinWithdrawalDelayNotPassed {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for input length mismatch error
        let res = complete_queued_withdrawal_internal(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            vec![Addr::unchecked("token1")], // Incorrect length
            0,
            true,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for strategy withdrawal delay not passed
        let withdrawal = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(0),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies.clone(),
            shares: shares.clone(),
        };

        let premature_withdrawal_root: Binary = calculate_withdrawal_root(&withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &premature_withdrawal_root, &true)
            .unwrap();

        let withdrawal_delay_blocks1 = STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .load(&deps.storage, &strategy1)
            .unwrap();
        assert_eq!(withdrawal_delay_blocks1, 50);

        let new_withdrawal_delay_blocks = 10000000u64;

        let _ = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.save(
            deps.as_mut().storage,
            &strategy1.clone(),
            &new_withdrawal_delay_blocks,
        );

        let res = complete_queued_withdrawal_internal(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            withdrawal.clone(),
            tokens.clone(),
            0,
            false,
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::StrategyWithdrawalDelayNotPassed {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_complete_queued_withdrawal() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let withdrawer = staker.clone();
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");
        let tokens = vec![deps.api.addr_make("token1"), deps.api.addr_make("token2")];
        let shares = vec![Uint128::new(100), Uint128::new(200)];
        let strategies = vec![strategy1.clone(), strategy2.clone()];

        DELEGATED_TO
            .save(deps.as_mut().storage, &withdrawer, &operator)
            .unwrap();

        let withdrawal = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(0),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies.clone(),
            shares: shares.clone(),
        };

        let withdrawal_root = calculate_withdrawal_root(&withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root, &true)
            .unwrap();

        STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .save(deps.as_mut().storage, &strategy1.clone(), &5u64)
            .unwrap();
        STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .save(deps.as_mut().storage, &strategy2.clone(), &10u64)
            .unwrap();

        let strategy1_clone = strategy1.clone();
        let strategy2_clone = strategy2.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&(
                        vec![strategy1_clone.clone(), strategy2_clone.clone()],
                        vec![Uint128::new(100), Uint128::new(200)],
                    ))
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let info = message_info(&withdrawer, &[]);

        let complete_msg = ExecuteMsg::CompleteQueuedWithdrawal {
            withdrawal: withdrawal.clone(),
            tokens: tokens.clone(),
            middleware_times_index: 0,
            receive_as_tokens: true,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            complete_msg.clone(),
        )
        .unwrap();

        assert_eq!(res.events.len(), 1); // 1 completion event
        assert_eq!(res.events[0].ty, "WithdrawalCompleted");
        assert!(
            res.events[0]
                .attributes
                .iter()
                .any(|attr| attr.key == "withdrawal_root"
                    && attr.value == withdrawal_root.to_string())
        );

        // Verify state changes: the pending withdrawal should be removed
        assert!(!PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root));

        // Test for unauthorized attempt to complete
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root, &true)
            .unwrap();

        let unauthorized_info = message_info(&Addr::unchecked("not_authorized"), &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            unauthorized_info,
            complete_msg.clone(),
        );
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for insufficient delay
        let premature_withdrawal = Withdrawal {
            start_block: env.block.height - 5, // Not enough delay
            ..withdrawal.clone()
        };
        let premature_withdrawal_root = calculate_withdrawal_root(&premature_withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &premature_withdrawal_root, &true)
            .unwrap();
        let premature_msg = ExecuteMsg::CompleteQueuedWithdrawal {
            withdrawal: premature_withdrawal.clone(),
            tokens: tokens.clone(),
            middleware_times_index: 0,
            receive_as_tokens: true,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), premature_msg);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::MinWithdrawalDelayNotPassed {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for input length mismatch error
        let mismatch_msg = ExecuteMsg::CompleteQueuedWithdrawal {
            withdrawal: withdrawal.clone(),
            tokens: vec![Addr::unchecked("token1")], // Incorrect length
            middleware_times_index: 0,
            receive_as_tokens: true,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), mismatch_msg);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::InputLengthMismatch {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test for strategy withdrawal delay not passed
        let delayed_withdrawal = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(0),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies.clone(),
            shares: shares.clone(),
        };

        let delayed_withdrawal_root: Binary =
            calculate_withdrawal_root(&delayed_withdrawal).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &delayed_withdrawal_root, &true)
            .unwrap();

        let new_withdrawal_delay_blocks = 10000000u64;

        let _ = STRATEGY_WITHDRAWAL_DELAY_BLOCKS.save(
            deps.as_mut().storage,
            &strategy1.clone(),
            &new_withdrawal_delay_blocks,
        );

        let delay_msg = ExecuteMsg::CompleteQueuedWithdrawal {
            withdrawal: delayed_withdrawal.clone(),
            tokens: tokens.clone(),
            middleware_times_index: 0,
            receive_as_tokens: false,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), delay_msg);
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::StrategyWithdrawalDelayNotPassed {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_complete_queued_withdrawals() {
        let (mut deps, env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");
        let withdrawer = staker.clone();
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");
        let tokens1 = vec![deps.api.addr_make("token1"), deps.api.addr_make("token2")];
        let tokens2 = vec![deps.api.addr_make("token3"), deps.api.addr_make("token4")];
        let shares1 = vec![Uint128::new(100), Uint128::new(200)];
        let shares2 = vec![Uint128::new(150), Uint128::new(250)];
        let strategies1 = vec![strategy1.clone(), strategy2.clone()];
        let strategies2 = vec![strategy1.clone(), strategy2.clone()];

        DELEGATED_TO
            .save(deps.as_mut().storage, &withdrawer, &operator)
            .unwrap();

        let withdrawal1 = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(0),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies1.clone(),
            shares: shares1.clone(),
        };

        let withdrawal2 = Withdrawal {
            staker: staker.clone(),
            delegated_to: operator.clone(),
            withdrawer: withdrawer.clone(),
            nonce: Uint128::new(1),
            start_block: env.block.height - 100, // Simulate sufficient delay has passed
            strategies: strategies2.clone(),
            shares: shares2.clone(),
        };

        let withdrawal_root1 = calculate_withdrawal_root(&withdrawal1).unwrap();
        let withdrawal_root2 = calculate_withdrawal_root(&withdrawal2).unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root1, &true)
            .unwrap();
        PENDING_WITHDRAWALS
            .save(deps.as_mut().storage, &withdrawal_root2, &true)
            .unwrap();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } if *contract_addr == deps.api.addr_make("strategy_manager").to_string() => {
                SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&(
                        vec![strategy1.clone(), strategy2.clone()],
                        vec![Uint128::new(100), Uint128::new(200)],
                    ))
                    .unwrap(),
                ))
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let info = message_info(&withdrawer, &[]);

        let complete_msg = ExecuteMsg::CompleteQueuedWithdrawals {
            withdrawals: vec![withdrawal1.clone(), withdrawal2.clone()],
            tokens: vec![tokens1.clone(), tokens2.clone()],
            middleware_times_indexes: vec![0, 1],
            receive_as_tokens: vec![true, true],
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), complete_msg).unwrap();

        // Verify the result
        assert_eq!(res.events.len(), 2);
        assert_eq!(res.events[0].ty, "WithdrawalCompleted");
        assert_eq!(res.events[1].ty, "WithdrawalCompleted");

        assert!(res.events[0].attributes.iter().any(
            |attr| attr.key == "withdrawal_root" && attr.value == withdrawal_root1.to_string()
        ));
        assert!(res.events[1].attributes.iter().any(
            |attr| attr.key == "withdrawal_root" && attr.value == withdrawal_root2.to_string()
        ));

        // Verify state changes: the pending withdrawals should be removed
        assert!(!PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root1));
        assert!(!PENDING_WITHDRAWALS.has(deps.as_ref().storage, &withdrawal_root2));
    }

    #[test]
    fn test_query_is_delegated() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let operator = deps.api.addr_make("operator1");

        DELEGATED_TO
            .save(deps.as_mut().storage, &staker, &operator)
            .unwrap();

        let query_msg = QueryMsg::IsDelegated {
            staker: staker.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: DelegatedResponse = from_json(res).unwrap();

        assert!(response.is_delegated);

        // Test for a staker that is not delegated
        let non_delegated_staker = deps.api.addr_make("staker2");
        let query_msg = QueryMsg::IsDelegated {
            staker: non_delegated_staker.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: DelegatedResponse = from_json(res).unwrap();

        // Assert that the non-delegated staker is not delegated
        assert!(!response.is_delegated);
    }

    #[test]
    fn test_query_is_operator() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        DELEGATED_TO
            .save(deps.as_mut().storage, &operator.clone(), &operator.clone())
            .unwrap();
        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator.clone(), &operator_details)
            .unwrap();

        let query_msg = QueryMsg::IsOperator {
            operator: operator.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: OperatorResponse = from_json(res).unwrap();
        let is_operator = response.is_operator;

        assert!(is_operator);

        // Test for an address that is not an operator
        let non_operator = deps.api.addr_make("non_operator");
        let query_msg = QueryMsg::IsOperator {
            operator: non_operator.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: OperatorResponse = from_json(res).unwrap();
        let is_operator = response.is_operator;

        assert!(!is_operator);
    }

    #[test]
    fn test_query_operator_details() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        let query_msg = QueryMsg::OperatorDetails {
            operator: operator.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let details_response: OperatorDetailsResponse = from_json(res).unwrap();

        assert_eq!(
            details_response.details.staker_opt_out_window_blocks,
            operator_details.staker_opt_out_window_blocks
        );

        // Test querying details for an operator that does not exist
        let non_operator = deps.api.addr_make("non_operator");
        let query_msg = QueryMsg::OperatorDetails {
            operator: non_operator.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_query_staker_opt_out_window_blocks() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };

        OPERATOR_DETAILS
            .save(deps.as_mut().storage, &operator, &operator_details)
            .unwrap();

        let query_msg = QueryMsg::StakerOptOutWindowBlocks {
            operator: operator.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: StakerOptOutWindowBlocksResponse = from_json(res).unwrap();

        assert_eq!(
            response.staker_opt_out_window_blocks,
            operator_details.staker_opt_out_window_blocks
        );
    }

    #[test]
    fn test_query_operator_shares() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator1");
        let strategies = vec![
            deps.api.addr_make("strategy1").to_string(),
            deps.api.addr_make("strategy2").to_string(),
        ];

        let shares_strategy1 = Uint128::new(100);
        let shares_strategy2 = Uint128::new(200);

        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &Addr::unchecked(strategies[0].clone())),
                &shares_strategy1,
            )
            .unwrap();
        OPERATOR_SHARES
            .save(
                deps.as_mut().storage,
                (&operator, &Addr::unchecked(strategies[1].clone())),
                &shares_strategy2,
            )
            .unwrap();

        let query_msg = QueryMsg::GetOperatorShares {
            operator: operator.to_string(),
            strategies: strategies.clone(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let operator_shares_response: OperatorSharesResponse = from_json(res).unwrap();

        assert_eq!(operator_shares_response.shares.len(), 2);
        assert_eq!(operator_shares_response.shares[0], shares_strategy1);
        assert_eq!(operator_shares_response.shares[1], shares_strategy2);

        // Test querying shares for an operator with no shares set
        let new_operator = deps.api.addr_make("new_operator");
        let query_msg = QueryMsg::GetOperatorShares {
            operator: new_operator.to_string(),
            strategies,
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let operator_shares_response: OperatorSharesResponse = from_json(res).unwrap();

        assert_eq!(operator_shares_response.shares.len(), 2);
        assert_eq!(operator_shares_response.shares[0], Uint128::zero());
        assert_eq!(operator_shares_response.shares[1], Uint128::zero());
    }

    #[test]
    fn test_query_get_withdrawal_delay() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let strategies = vec![
            deps.api.addr_make("strategy1").to_string(),
            deps.api.addr_make("strategy2").to_string(),
        ];

        STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(strategies[0].clone()),
                &5u64,
            )
            .unwrap();
        STRATEGY_WITHDRAWAL_DELAY_BLOCKS
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(strategies[1].clone()),
                &10u64,
            )
            .unwrap();

        let query_msg = QueryMsg::GetWithdrawalDelay {
            strategies: strategies.clone(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let withdrawal_delays_response: WithdrawalDelayResponse = from_json(res).unwrap();

        assert_eq!(withdrawal_delays_response.withdrawal_delays.len(), 2);
        assert_eq!(withdrawal_delays_response.withdrawal_delays[0], 100); // Assuming we want max of min_delay and strategy delay
        assert_eq!(withdrawal_delays_response.withdrawal_delays[1], 100);

        // Test querying withdrawal delay for strategies with no delay set
        let new_strategy = deps.api.addr_make("strategy3");
        let query_msg = QueryMsg::GetWithdrawalDelay {
            strategies: vec![new_strategy.to_string()],
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let withdrawal_delays_response: WithdrawalDelayResponse = from_json(res).unwrap();

        assert_eq!(withdrawal_delays_response.withdrawal_delays.len(), 1);
        assert_eq!(withdrawal_delays_response.withdrawal_delays[0], 100);
    }

    #[test]
    fn test_transfer_ownership() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_owner = deps.api.addr_make("new_owner");

        let msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), msg);

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
    fn test_query_operator_stakers() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let operator = deps.api.addr_make("operator");
        let staker1 = deps.api.addr_make("staker1");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        DELEGATED_TO
            .save(deps.as_mut().storage, &staker1, &operator)
            .unwrap();

        let staker1_clone = staker1.clone();
        let strategy1_clone = strategy1.clone();
        let strategy2_clone = strategy2.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == deps.api.addr_make("strategy_manager").to_string() =>
            {
                let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyManagerQueryMsg::GetStakerStrategyList { staker } => {
                        assert_eq!(staker, staker1_clone.to_string());
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&StakerStrategyListResponse {
                                strategies: vec![strategy1_clone.clone(), strategy2_clone.clone()],
                            })
                            .unwrap(),
                        ))
                    }
                    StrategyManagerQueryMsg::GetStakerStrategyShares { staker, strategy } => {
                        if staker == staker1_clone.to_string()
                            && strategy == strategy1_clone.to_string()
                        {
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&StakerStrategySharesResponse {
                                    shares: Uint128::new(100),
                                })
                                .unwrap(),
                            ))
                        } else if staker == staker1_clone.to_string()
                            && strategy == strategy2_clone.to_string()
                        {
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&StakerStrategySharesResponse {
                                    shares: Uint128::new(200),
                                })
                                .unwrap(),
                            ))
                        } else {
                            SystemResult::Err(SystemError::InvalidRequest {
                                error: "Unhandled request".to_string(),
                                request: to_json_binary(&query).unwrap(),
                            })
                        }
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = query_operator_stakers(deps.as_ref(), operator.clone()).unwrap();

        assert_eq!(res.stakers_and_shares.len(), 1);

        let staker1_result = res
            .stakers_and_shares
            .iter()
            .find(|staker_shares| staker_shares.staker == staker1)
            .unwrap();

        assert_eq!(staker1_result.staker, staker1);

        assert_eq!(staker1_result.shares_per_strategy.len(), 2);

        assert!(staker1_result
            .shares_per_strategy
            .contains(&(strategy1.clone(), Uint128::new(100))));

        assert!(staker1_result
            .shares_per_strategy
            .contains(&(strategy2.clone(), Uint128::new(200))));
    }

    #[test]
    fn test_query_cumulative_withdrawals_queued() {
        let (mut deps, _env, _owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let staker = deps.api.addr_make("staker1");

        CUMULATIVE_WITHDRAWALS_QUEUED
            .save(deps.as_mut().storage, &staker, &Uint128::new(5))
            .unwrap();

        let msg = QueryMsg::GetCumulativeWithdrawalsQueued {
            staker: staker.to_string(),
        };

        let res: CumulativeWithdrawalsQueuedResponse =
            from_json(query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(res.cumulative_withdrawals, Uint128::new(5));
    }

    #[test]
    fn test_pause() {
        let (mut deps, env, _owner_info, pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let pause_msg = ExecuteMsg::Pause {};
        let res = execute(deps.as_mut(), env.clone(), pauser_info.clone(), pause_msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action", "PAUSED")]);

        let paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_state, 1);
    }

    #[test]
    fn test_unpause() {
        let (mut deps, env, _owner_info, _pauser_info, unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let unpause_msg = ExecuteMsg::Unpause {};
        let res = execute(
            deps.as_mut(),
            env.clone(),
            unpauser_info.clone(),
            unpause_msg,
        )
        .unwrap();

        assert_eq!(res.attributes, vec![attr("action", "UNPAUSED")]);

        let paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_state, 0);
    }

    #[test]
    fn test_set_pauser() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_pauser = deps.api.addr_make("new_pauser").to_string();

        let set_pauser_msg = ExecuteMsg::SetPauser {
            new_pauser: new_pauser.to_string(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            set_pauser_msg,
        )
        .unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "method" && a.value == "set_pauser"));

        let pauser = PAUSER.load(&deps.storage).unwrap();
        assert_eq!(pauser, Addr::unchecked(new_pauser));
    }

    #[test]
    fn test_set_unpauser() {
        let (mut deps, env, owner_info, _pauser_info, _unpauser_info, _strategy_manager_info) =
            instantiate_contract();

        let new_unpauser = deps.api.addr_make("new_unpauser").to_string();

        let set_unpauser_msg = ExecuteMsg::SetUnpauser {
            new_unpauser: new_unpauser.to_string(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            set_unpauser_msg,
        )
        .unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "method" && a.value == "set_unpauser"));

        let unpauser = UNPAUSER.load(&deps.storage).unwrap();
        assert_eq!(unpauser, Addr::unchecked(new_unpauser));
    }
}
