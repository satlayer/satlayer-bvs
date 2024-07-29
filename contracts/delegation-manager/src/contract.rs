use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StrategyManagerQueryMsg, SignatureWithSaltAndExpiry},
    state::{
        DelegationManagerState, DELEGATION_MANAGER_STATE, OPERATOR_DETAILS, OWNER, OperatorDetails,
        MIN_WITHDRAWAL_DELAY_BLOCKS, STRATEGY_WITHDRAWAL_DELAY_BLOCKS, OPERATOR_SHARES, DELEGATION_APPROVER_SALT_SPENT
    },
    utils::{calculate_digest_hash, recover, DigestHashParams},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64, Uint128, WasmQuery, QueryRequest,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_STAKER_OPT_OUT_WINDOW_BLOCKS: u64 = 180 * 24 * 60 * 60 / 12; // 180天，每秒一个块

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
    set_min_withdrawal_delay_blocks(deps.branch(), msg.min_withdrawal_delay_blocks)?;
    set_strategy_withdrawal_delay_blocks(deps.branch(), msg.strategies, msg.withdrawal_delay_blocks)?;

    let response = Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("slasher", state.slasher.to_string())
        .add_attribute("min_withdrawal_delay_blocks", msg.min_withdrawal_delay_blocks.to_string())
        .add_attribute("owner", msg.initial_owner.to_string());

    Ok(response)
}

fn set_strategy_withdrawal_delay_blocks(
    deps: DepsMut,
    strategies: Vec<Addr>,
    withdrawal_delay_blocks: Vec<Uint64>,
) -> Result<(), ContractError> {
    if strategies.len() != withdrawal_delay_blocks.len() {
        return Err(ContractError::InvalidInput {});
    }

    for (i, strategy) in strategies.iter().enumerate() {
        STRATEGY_WITHDRAWAL_DELAY_BLOCKS.save(deps.storage, strategy, &withdrawal_delay_blocks[i])?;
    }

    Ok(())
}

pub fn set_operator_details(
    deps: DepsMut,
    operator: Addr,
    new_operator_details: OperatorDetails,
) -> Result<Response, ContractError> {
    let current_operator_details = OPERATOR_DETAILS.load(deps.storage, &operator)?;

    if new_operator_details.staker_opt_out_window_blocks > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS {
        return Err(ContractError::InvalidInput {});
    }

    if new_operator_details.staker_opt_out_window_blocks < current_operator_details.staker_opt_out_window_blocks {
        return Err(ContractError::CannotBeDecreased {});
    }

    OPERATOR_DETAILS.save(deps.storage, &operator, &new_operator_details)?;

    Ok(Response::new()
        .add_attribute("method", "set_operator_details")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker_opt_out_window_blocks", new_operator_details.staker_opt_out_window_blocks.to_string()))
}

pub fn delegate(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    staker: Addr,
    operator: Addr,
    approver_signature_and_expiry: SignatureWithSaltAndExpiry,
    approver_salt: String,
) -> Result<Response, ContractError> {
    let delegation_approver = OPERATOR_DETAILS.load(deps.storage, &operator)?.delegation_approver;

    let current_time: Uint64 = env.block.time.seconds().into();

    if delegation_approver != Addr::unchecked("0") && info.sender != delegation_approver && info.sender != operator {
        if approver_signature_and_expiry.expiry < current_time.u64() {
            return Err(ContractError::ApproverSignatureExpired {});
        }

        if DELEGATION_APPROVER_SALT_SPENT.load(deps.storage, (&delegation_approver, &approver_salt)).is_ok() {
            return Err(ContractError::ApproverSaltSpent {});
        }

        let digest_hash = calculate_digest_hash(
            DigestHashParams {
                staker: &staker.clone(),
                operator: &operator.clone(),
                delegation_approver: &delegation_approver.clone(),
                approver_salt: &approver_salt.clone(),
                expiry: approver_signature_and_expiry.expiry.clone(),
            },
        )?;

        recover(&approver_signature_and_expiry.signature, &digest_hash, &delegation_approver)?;

        DELEGATION_APPROVER_SALT_SPENT.save(deps.storage, (&delegation_approver, &approver_salt), &true)?;
    }

    DELEGATED_TO.save(deps.storage, &staker, &operator)?;

    let (strategies, shares) = get_delegatable_shares(deps.as_ref(), staker.clone())?;

    let mut response = Response::new()
        .add_attribute("method", "delegate")
        .add_attribute("staker", staker.to_string())
        .add_attribute("operator", operator.to_string());

    for (strategy, share) in strategies.iter().zip(shares.iter()) {
        let increase_shares_response = increase_operator_shares(
            deps.branch(),
            operator.clone(),
            staker.clone(),
            strategy.clone(),
            *share,
        )?;
        response = response.add_attributes(increase_shares_response.attributes);
    }

    Ok(response)
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

pub fn increase_operator_shares(
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

    Ok(Response::new()
        .add_attribute("method", "increase_operator_shares")
        .add_attribute("operator", operator.to_string())
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("new_shares", new_shares.to_string()))
}


// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn execute(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: ExecuteMsg,
// ) -> Result<Response, ContractError> {
//     match msg {
//         ExecuteMsg::RegisterAsOperator { operator_details, metadata_uri } => {
//             register_as_operator(deps, info, operator_details, metadata_uri)
//         }
//         ExecuteMsg::ModifyOperatorDetails { new_operator_details } => modify_operator_details(deps, info, new_operator_details),
//         ExecuteMsg::UpdateOperatorMetadataUri { metadata_uri } => update_operator_metadata_uri(deps, info, metadata_uri),
//         ExecuteMsg::DelegateTo { operator, approver_signature_and_expiry, approver_salt } => {
//             delegate_to(deps, info, operator, approver_signature_and_expiry, approver_salt)
//         }
//         ExecuteMsg::DelegateToBySignature { staker, operator, staker_signature_and_expiry, approver_signature_and_expiry, approver_salt } => {
//             delegate_to_by_signature(deps, info, staker, operator, staker_signature_and_expiry, approver_signature_and_expiry, approver_salt)
//         }
//         ExecuteMsg::Undelegate { staker } => undelegate(deps, info, staker),
//         ExecuteMsg::QueueWithdrawals { queued_withdrawal_params } => queue_withdrawals(deps, info, queued_withdrawal_params),
//         ExecuteMsg::CompleteQueuedWithdrawal { withdrawal, tokens, middleware_times_index, receive_as_tokens } => {
//             complete_queued_withdrawal(deps, info, withdrawal, tokens, middleware_times_index, receive_as_tokens)
//         }
//         ExecuteMsg::CompleteQueuedWithdrawals { withdrawals, tokens, middleware_times_indexes, receive_as_tokens } => {
//             complete_queued_withdrawals(deps, info, withdrawals, tokens, middleware_times_indexes, receive_as_tokens)
//         }
//         ExecuteMsg::IncreaseDelegatedShares { staker, strategy, shares } => increase_delegated_shares(deps, info, staker, strategy, shares),
//         ExecuteMsg::DecreaseDelegatedShares { staker, strategy, shares } => decrease_delegated_shares(deps, info, staker, strategy, shares),
//         ExecuteMsg::SetMinWithdrawalDelayBlocks { new_min_withdrawal_delay_blocks } => set_min_withdrawal_delay_blocks(deps, info, new_min_withdrawal_delay_blocks),
//         ExecuteMsg::SetStrategyWithdrawalDelayBlocks { strategies, withdrawal_delay_blocks } => {
//             set_strategy_withdrawal_delay_blocks(deps, info, strategies, withdrawal_delay_blocks)
//         }
//     }
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::DomainSeparator {} => to_json_binary(&query_domain_separator(deps)?),
//         QueryMsg::IsDelegated { staker } => to_json_binary(&query_is_delegated(deps, staker)?),
//         QueryMsg::IsOperator { operator } => to_json_binary(&query_is_operator(deps, operator)?),
//         QueryMsg::OperatorDetails { operator } => to_json_binary(&query_operator_details(deps, operator)?),
//         QueryMsg::DelegationApprover { operator } => to_json_binary(&query_delegation_approver(deps, operator)?),
//         QueryMsg::StakerOptOutWindowBlocks { operator } => to_json_binary(&query_staker_opt_out_window_blocks(deps, operator)?),
//         QueryMsg::OperatorShares { operator, strategies } => to_json_binary(&query_operator_shares(deps, operator, strategies)?),
//         QueryMsg::DelegatableShares { staker } => to_json_binary(&query_delegatable_shares(deps, staker)?),
//         QueryMsg::WithdrawalDelay { strategies } => to_json_binary(&query_withdrawal_delay(deps, strategies)?),
//         QueryMsg::WithdrawalRoot { withdrawal } => to_json_binary(&query_withdrawal_root(deps, withdrawal)?),
//         QueryMsg::CurrentStakerDelegationDigestHash { staker, operator, expiry } => to_binary(&query_current_staker_delegation_digest_hash(deps, staker, operator, expiry)?),
//         QueryMsg::StakerDelegationDigestHash { staker, nonce, operator, expiry } => to_binary(&query_staker_delegation_digest_hash(deps, staker, nonce, operator, expiry)?),
//         QueryMsg::DelegationApprovalDigestHash { staker, operator, delegation_approver, approver_salt, expiry } => to_binary(&query_delegation_approval_digest_hash(deps, staker, operator, delegation_approver, approver_salt, expiry)?),
//     }
// }
