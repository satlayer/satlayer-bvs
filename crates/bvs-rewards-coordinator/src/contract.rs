#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{DistributionRoot, ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{
        CalculateDomainSeparatorResponse, CalculateEarnerLeafHashResponse,
        CalculateTokenLeafHashResponse, CheckClaimResponse,
        GetCurrentClaimableDistributionRootResponse, GetCurrentDistributionRootResponse,
        GetDistributionRootAtIndexResponse, GetDistributionRootsLengthResponse,
        GetRootIndexFromHashResponse, MerkleizeLeavesResponse, OperatorCommissionBipsResponse,
    },
    state::{
        ACTIVATION_DELAY, CALCULATION_INTERVAL_SECONDS, CLAIMER_FOR, CUMULATIVE_CLAIMED,
        CURR_REWARDS_CALCULATION_END_TIMESTAMP, DELEGATION_MANAGER, DISTRIBUTION_ROOTS,
        DISTRIBUTION_ROOTS_COUNT, GENESIS_REWARDS_TIMESTAMP, GLOBAL_OPERATOR_COMMISSION_BIPS,
        IS_BVS_REWARDS_SUBMISSION_HASH, MAX_FUTURE_LENGTH, MAX_RETROACTIVE_LENGTH,
        MAX_REWARDS_DURATION, OWNER, REWARDS_FOR_ALL_SUBMITTER, REWARDS_UPDATER, STRATEGY_MANAGER,
        SUBMISSION_NONCE,
    },
    utils::{
        calculate_domain_separator, calculate_earner_leaf_hash, calculate_rewards_submission_hash,
        calculate_token_leaf_hash, merkleize_sha256, verify_inclusion_sha256, EarnerTreeMerkleLeaf,
        RewardsMerkleClaim, RewardsSubmission, TokenTreeMerkleLeaf,
    },
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, HexBinary, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use bvs_strategy_manager::{
    msg::QueryMsg as StrategyManagerQueryMsg, query::StrategyWhitelistedResponse,
};

const CONTRACT_NAME: &str = "BVS Rewards Coordinator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SNAPSHOT_CADENCE: u64 = 86_400;
const MAX_REWARDS_AMOUNT: u128 = 100000000000000000000000000000000000000 - 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    bvs_registry::api::set_registry(deps.storage, &deps.api.addr_validate(&msg.registry)?)?;

    if msg.genesis_rewards_timestamp % msg.calculation_interval_seconds != 0 {
        return Err(ContractError::InvalidGenesisTimestamp {});
    }

    if msg.calculation_interval_seconds % SNAPSHOT_CADENCE != 0 {
        return Err(ContractError::InvalidCalculationInterval {});
    }

    let initial_owner = deps.api.addr_validate(&msg.initial_owner)?;

    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;
    let delegation_manager = deps.api.addr_validate(&msg.delegation_manager)?;

    let rewards_updater = deps.api.addr_validate(&msg.rewards_updater)?;

    OWNER.save(deps.storage, &initial_owner)?;

    CALCULATION_INTERVAL_SECONDS.save(deps.storage, &msg.calculation_interval_seconds)?;
    MAX_REWARDS_DURATION.save(deps.storage, &msg.max_rewards_duration)?;
    MAX_RETROACTIVE_LENGTH.save(deps.storage, &msg.max_retroactive_length)?;
    MAX_FUTURE_LENGTH.save(deps.storage, &msg.max_future_length)?;
    GENESIS_REWARDS_TIMESTAMP.save(deps.storage, &msg.genesis_rewards_timestamp)?;
    DELEGATION_MANAGER.save(deps.storage, &delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    set_rewards_updater_internal(deps.branch(), rewards_updater.clone())?;
    set_activation_delay_internal(deps.branch(), msg.activation_delay)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", initial_owner.to_string())
        .add_attribute("rewards_updater", msg.rewards_updater.to_string())
        .add_attribute("activation_delay", msg.activation_delay.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_registry::api::is_paused(deps.as_ref(), &env, &msg)?;

    match msg {
        ExecuteMsg::CreateBvsRewardsSubmission {
            rewards_submissions,
        } => create_bvs_rewards_submission(deps, env, info, rewards_submissions),
        ExecuteMsg::CreateRewardsForAllSubmission {
            rewards_submissions,
        } => create_rewards_for_all_submission(deps, env, info, rewards_submissions),
        ExecuteMsg::ProcessClaim { claim, recipient } => {
            let earner = deps.api.addr_validate(&claim.earner_leaf.earner)?;
            let recipient = deps.api.addr_validate(&recipient)?;

            let earner_token_root_binary =
                Binary::from_base64(&claim.earner_leaf.earner_token_root)?;

            let earner_tree_merkle_leaf = EarnerTreeMerkleLeaf {
                earner,
                earner_token_root: earner_token_root_binary,
            };

            let params = RewardsMerkleClaim {
                root_index: claim.root_index,
                earner_index: claim.earner_index,
                earner_tree_proof: claim.earner_tree_proof,
                earner_leaf: earner_tree_merkle_leaf,
                token_indices: claim.token_indices,
                token_tree_proofs: claim.token_tree_proofs,
                token_leaves: claim.token_leaves,
            };

            process_claim(deps, env, info, params, recipient)
        }
        ExecuteMsg::SubmitRoot {
            root,
            rewards_calculation_end_timestamp,
        } => {
            let root = Binary::from_base64(&root)?;
            submit_root(deps, env, info, root, rewards_calculation_end_timestamp)
        }
        ExecuteMsg::DisableRoot { root_index } => disable_root(deps, env, info, root_index),
        ExecuteMsg::SetClaimerFor { claimer } => {
            let claimer = deps.api.addr_validate(&claimer)?;
            set_claimer_for(deps, info, claimer)
        }
        ExecuteMsg::SetActivationDelay {
            new_activation_delay,
        } => set_activation_delay(deps, info, new_activation_delay),
        ExecuteMsg::SetRewardsUpdater { new_updater } => {
            let new_updater = deps.api.addr_validate(&new_updater)?;
            set_rewards_updater(deps, info, new_updater)
        }
        ExecuteMsg::SetRewardsForAllSubmitter {
            submitter,
            new_value,
        } => {
            let submitter = deps.api.addr_validate(&submitter)?;
            set_rewards_for_all_submitter(deps, info, submitter, new_value)
        }
        ExecuteMsg::SetGlobalOperatorCommission {
            new_commission_bips,
        } => set_global_operator_commission(deps, info, new_commission_bips),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            transfer_ownership(deps, info, new_owner)
        }
    }
}

pub fn create_bvs_rewards_submission(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rewards_submissions: Vec<RewardsSubmission>,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    for submission in rewards_submissions {
        let nonce = SUBMISSION_NONCE
            .may_load(deps.storage, info.sender.clone())?
            .unwrap_or_default();

        let rewards_submission_hash =
            calculate_rewards_submission_hash(&info.sender, nonce, &submission);

        validate_rewards_submission(&deps.as_ref(), &submission, &env)?;

        IS_BVS_REWARDS_SUBMISSION_HASH.save(
            deps.storage,
            (info.sender.clone(), rewards_submission_hash.to_vec()),
            &true,
        )?;

        SUBMISSION_NONCE.save(deps.storage, info.sender.clone(), &(nonce + 1))?;

        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: submission.token.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: submission.amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(transfer_msg);

        let event = Event::new("BVSRewardsSubmissionCreated")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("nonce", nonce.to_string())
            .add_attribute(
                "rewards_submission_hash",
                rewards_submission_hash.to_string(),
            )
            .add_attribute("token", submission.token.to_string())
            .add_attribute("amount", submission.amount.to_string());

        response = response.add_event(event);
    }

    Ok(response)
}

pub fn create_rewards_for_all_submission(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rewards_submissions: Vec<RewardsSubmission>,
) -> Result<Response, ContractError> {
    only_rewards_for_all_submitter(deps.as_ref(), &info)?;

    let mut response = Response::new();
    for submission in rewards_submissions {
        let nonce = SUBMISSION_NONCE
            .may_load(deps.storage, info.sender.clone())?
            .unwrap_or_default();

        let rewards_submission_hash =
            calculate_rewards_submission_hash(&info.sender, nonce, &submission);

        validate_rewards_submission(&deps.as_ref(), &submission, &env)?;

        IS_BVS_REWARDS_SUBMISSION_HASH.save(
            deps.storage,
            (info.sender.clone(), rewards_submission_hash.to_vec()),
            &true,
        )?;

        SUBMISSION_NONCE.save(deps.storage, info.sender.clone(), &(nonce + 1))?;

        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: submission.token.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: submission.amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(transfer_msg);

        let event = Event::new("RewardsSubmissionForAllCreated")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("nonce", nonce.to_string())
            .add_attribute(
                "rewards_submission_hash",
                rewards_submission_hash.to_string(),
            )
            .add_attribute("token", submission.token.to_string())
            .add_attribute("amount", submission.amount.to_string());

        response = response.add_event(event);
    }

    Ok(response)
}

pub fn process_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    claim: RewardsMerkleClaim,
    recipient: Addr,
) -> Result<Response, ContractError> {
    let root: DistributionRoot = DISTRIBUTION_ROOTS
        .may_load(deps.storage, claim.root_index.into())?
        .ok_or(ContractError::RootNotExist {})?;

    check_claim_internal(env.clone(), &claim, &root)?;

    let earner = claim.earner_leaf.earner.clone();
    let mut claimer = CLAIMER_FOR
        .may_load(deps.storage, earner.clone())?
        .unwrap_or_else(|| earner.clone());

    if claimer == Addr::unchecked("") {
        claimer = earner.clone();
    }

    if info.sender != claimer {
        return Err(ContractError::UnauthorizedClaimer {});
    }

    let mut response = Response::new();

    for token_leaf in claim.token_leaves.iter() {
        let token = &token_leaf.token;

        let curr_cumulative_claimed = CUMULATIVE_CLAIMED
            .may_load(deps.storage, (earner.clone(), token.to_string()))?
            .unwrap_or_default();

        if token_leaf.cumulative_earnings <= curr_cumulative_claimed {
            return Err(ContractError::CumulativeEarningsTooLow {});
        }

        let claim_amount = token_leaf.cumulative_earnings - curr_cumulative_claimed;

        let balance = token_balance(&deps.querier, token, &env.contract.address)?;

        if claim_amount > balance {
            return Err(ContractError::InsufficientBalance {});
        }

        CUMULATIVE_CLAIMED.save(
            deps.storage,
            (earner.clone(), token.to_string()),
            &token_leaf.cumulative_earnings,
        )?;

        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.clone().into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount: claim_amount,
            })?,
            funds: vec![],
        });

        response = response.add_message(transfer_msg);

        let event = Event::new("RewardsClaimed")
            .add_attribute("root", format!("{:?}", root.root))
            .add_attribute("earner", earner.to_string())
            .add_attribute("claimer", claimer.to_string())
            .add_attribute("recipient", recipient.to_string())
            .add_attribute("token", token.to_string())
            .add_attribute("amount", claim_amount.to_string());

        response = response.add_event(event);
    }

    Ok(response)
}

pub fn submit_root(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    root: Binary,
    rewards_calculation_end_timestamp: u64,
) -> Result<Response, ContractError> {
    only_rewards_updater(deps.as_ref(), &info)?;

    let curr_rewards_calculation_end_timestamp = CURR_REWARDS_CALCULATION_END_TIMESTAMP
        .may_load(deps.storage)?
        .unwrap_or(0);

    if rewards_calculation_end_timestamp <= curr_rewards_calculation_end_timestamp {
        return Err(ContractError::InvalidTimestamp {});
    }

    if rewards_calculation_end_timestamp >= env.block.time.seconds() {
        return Err(ContractError::TimestampInFuture {});
    }

    let activation_delay = ACTIVATION_DELAY.load(deps.storage)?;

    let activated_at = env
        .block
        .time
        .plus_seconds(activation_delay.into())
        .seconds();

    let root_index = DISTRIBUTION_ROOTS_COUNT
        .may_load(deps.storage)?
        .unwrap_or(0);

    let new_root = DistributionRoot {
        root,
        activated_at,
        rewards_calculation_end_timestamp,
        disabled: false,
    };
    DISTRIBUTION_ROOTS.save(deps.storage, root_index, &new_root)?;

    CURR_REWARDS_CALCULATION_END_TIMESTAMP
        .save(deps.storage, &rewards_calculation_end_timestamp)?;

    DISTRIBUTION_ROOTS_COUNT.save(deps.storage, &(root_index + 1))?;

    let event = Event::new("DistributionRootSubmitted")
        .add_attribute("root_index", root_index.to_string())
        .add_attribute("root", format!("{:?}", new_root.root))
        .add_attribute(
            "rewards_calculation_end_timestamp",
            new_root.rewards_calculation_end_timestamp.to_string(),
        )
        .add_attribute("activated_at", new_root.activated_at.to_string());

    Ok(Response::new().add_event(event))
}

pub fn disable_root(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    root_index: u64,
) -> Result<Response, ContractError> {
    only_rewards_updater(deps.as_ref(), &info)?;

    let roots_length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;
    if root_index >= roots_length {
        return Err(ContractError::InvalidRootIndex {});
    }

    let mut root: DistributionRoot = DISTRIBUTION_ROOTS
        .load(deps.storage, root_index)
        .map_err(|_| ContractError::RootNotExist {})?;

    if root.disabled {
        return Err(ContractError::AlreadyDisabled {});
    }
    if env.block.time.seconds() >= root.activated_at {
        return Err(ContractError::AlreadyActivated {});
    }

    root.disabled = true;
    DISTRIBUTION_ROOTS.save(deps.storage, root_index, &root)?;

    let event =
        Event::new("DistributionRootDisabled").add_attribute("root_index", root_index.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_claimer_for(
    deps: DepsMut,
    info: MessageInfo,
    claimer: Addr,
) -> Result<Response, ContractError> {
    let earner = info.sender;
    let prev_claimer = CLAIMER_FOR
        .may_load(deps.storage, earner.clone())?
        .unwrap_or(Addr::unchecked(""));

    CLAIMER_FOR.save(deps.storage, earner.clone(), &claimer)?;

    let event = Event::new("ClaimerForSet")
        .add_attribute("earner", earner.to_string())
        .add_attribute("previous_claimer", prev_claimer.to_string())
        .add_attribute("new_claimer", claimer.to_string());

    Ok(Response::new().add_event(event))
}

pub fn check_claim(env: Env, deps: Deps, claim: RewardsMerkleClaim) -> Result<bool, ContractError> {
    let root = DISTRIBUTION_ROOTS
        .may_load(deps.storage, claim.root_index.into())?
        .ok_or(ContractError::RootNotExist {})?;

    check_claim_internal(env, &claim, &root)?;

    Ok(true)
}

pub fn set_activation_delay(
    deps: DepsMut,
    info: MessageInfo,
    new_activation_delay: u32,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let res = set_activation_delay_internal(deps, new_activation_delay)?;
    Ok(res)
}

pub fn set_rewards_updater(
    deps: DepsMut,
    info: MessageInfo,
    new_updater: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let res = set_rewards_updater_internal(deps, new_updater)?;
    Ok(res)
}

pub fn set_rewards_for_all_submitter(
    deps: DepsMut,
    info: MessageInfo,
    submitter: Addr,
    new_value: bool,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let prev_value = REWARDS_FOR_ALL_SUBMITTER
        .may_load(deps.storage, submitter.clone())?
        .unwrap_or(false);
    REWARDS_FOR_ALL_SUBMITTER.save(deps.storage, submitter.clone(), &new_value)?;

    Ok(Response::new()
        .add_attribute("method", "set_rewards_for_all_submitter")
        .add_attribute("submitter", submitter.to_string())
        .add_attribute("previous_value", prev_value.to_string())
        .add_attribute("new_value", new_value.to_string()))
}

pub fn set_global_operator_commission(
    deps: DepsMut,
    info: MessageInfo,
    new_commission_bips: u16,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let res = set_global_operator_commission_internal(deps, new_commission_bips)?;
    Ok(res)
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

    let event = Event::new("TransferOwnership")
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string());

    Ok(Response::new().add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CalculateEarnerLeafHash {
            earner,
            earner_token_root,
        } => to_json_binary(&query_calculate_earner_leaf_hash(
            deps,
            earner,
            earner_token_root,
        )?),
        QueryMsg::CalculateTokenLeafHash {
            token,
            cumulative_earnings,
        } => to_json_binary(&query_calculate_token_leaf_hash(
            deps,
            token,
            cumulative_earnings,
        )?),

        QueryMsg::OperatorCommissionBips { operator, bvs } => {
            to_json_binary(&query_operator_commission_bips(deps, operator, bvs)?)
        }

        QueryMsg::GetDistributionRootsLength {} => {
            to_json_binary(&query_distribution_roots_length(deps)?)
        }

        QueryMsg::GetDistributionRootAtIndex { index } => {
            to_json_binary(&query_distribution_root_at_index(deps, index)?)
        }

        QueryMsg::GetCurrentDistributionRoot {} => {
            to_json_binary(&query_current_distribution_root(deps)?)
        }

        QueryMsg::GetCurrentClaimableDistributionRoot {} => {
            to_json_binary(&query_current_claimable_distribution_root(deps, env)?)
        }

        QueryMsg::GetRootIndexFromHash { root_hash } => {
            to_json_binary(&query_root_index_from_hash(deps, root_hash)?)
        }

        QueryMsg::CalculateDomainSeparator {
            chain_id,
            contract_addr,
        } => to_json_binary(&query_calculate_domain_separator(
            deps,
            chain_id,
            contract_addr,
        )?),

        QueryMsg::MerkleizeLeaves { leaves } => {
            let binary_leaves: Vec<Binary> = leaves
                .iter()
                .map(|leaf_str| Binary::from_base64(leaf_str))
                .collect::<Result<Vec<Binary>, _>>()?;

            to_json_binary(&query_merkleize_leaves(binary_leaves)?)
        }

        QueryMsg::CheckClaim { claim } => {
            let earner = deps.api.addr_validate(&claim.earner_leaf.earner)?;

            let earner_token_root_binary =
                Binary::from_base64(&claim.earner_leaf.earner_token_root)?;

            let earner_tree_merkle_leaf = EarnerTreeMerkleLeaf {
                earner,
                earner_token_root: earner_token_root_binary,
            };

            let params = RewardsMerkleClaim {
                root_index: claim.root_index,
                earner_index: claim.earner_index,
                earner_tree_proof: claim.earner_tree_proof,
                earner_leaf: earner_tree_merkle_leaf,
                token_indices: claim.token_indices,
                token_tree_proofs: claim.token_tree_proofs,
                token_leaves: claim.token_leaves,
            };

            to_json_binary(&query_check_claim(deps, env, params)?)
        }
    }
}

fn query_calculate_earner_leaf_hash(
    _deps: Deps,
    earner: String,
    earner_token_root: String,
) -> StdResult<CalculateEarnerLeafHashResponse> {
    let earner_addr: Addr = Addr::unchecked(earner);
    let earner_token_root_binary = Binary::from_base64(&earner_token_root)?;

    let leaf = EarnerTreeMerkleLeaf {
        earner: earner_addr,
        earner_token_root: earner_token_root_binary,
    };

    let hash = calculate_earner_leaf_hash(&leaf);
    let hash_binary = Binary::from(hash);

    Ok(CalculateEarnerLeafHashResponse { hash_binary })
}

fn query_calculate_token_leaf_hash(
    _deps: Deps,
    token: String,
    cumulative_earnings: Uint128,
) -> StdResult<CalculateTokenLeafHashResponse> {
    let token_addr: Addr = Addr::unchecked(token);

    let leaf = TokenTreeMerkleLeaf {
        token: token_addr,
        cumulative_earnings,
    };

    let hash = calculate_token_leaf_hash(&leaf);
    let hash_binary = Binary::from(hash);

    Ok(CalculateTokenLeafHashResponse { hash_binary })
}

fn query_operator_commission_bips(
    deps: Deps,
    _operator: String,
    _bvs: String,
) -> StdResult<OperatorCommissionBipsResponse> {
    let commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS.load(deps.storage)?;

    Ok(OperatorCommissionBipsResponse { commission_bips })
}

fn query_distribution_roots_length(deps: Deps) -> StdResult<GetDistributionRootsLengthResponse> {
    let roots_length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    Ok(GetDistributionRootsLengthResponse { roots_length })
}

fn query_distribution_root_at_index(
    deps: Deps,
    index: String,
) -> StdResult<GetDistributionRootAtIndexResponse> {
    let index: u64 = index
        .parse()
        .map_err(|_| StdError::generic_err("Invalid index format"))?;

    let root: DistributionRoot = DISTRIBUTION_ROOTS
        .may_load(deps.storage, index)?
        .ok_or_else(|| StdError::generic_err("Root not exist"))?;

    Ok(GetDistributionRootAtIndexResponse { root })
}

fn query_current_distribution_root(deps: Deps) -> StdResult<GetCurrentDistributionRootResponse> {
    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if !root.disabled {
                return Ok(GetCurrentDistributionRootResponse { root });
            }
        }
    }

    Err(StdError::generic_err("No enabled distribution root found"))
}

fn query_current_claimable_distribution_root(
    deps: Deps,
    env: Env,
) -> StdResult<GetCurrentClaimableDistributionRootResponse> {
    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if !root.disabled && env.block.time.seconds() >= root.activated_at {
                return Ok(GetCurrentClaimableDistributionRootResponse { root });
            }
        }
    }

    Err(StdError::generic_err("No claimable root found"))
}

fn query_root_index_from_hash(
    deps: Deps,
    root_hash: String,
) -> StdResult<GetRootIndexFromHashResponse> {
    let root_hash_bytes =
        HexBinary::from_hex(&root_hash).map_err(|_| StdError::generic_err("Invalid hex format"))?;

    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if root.root.as_slice() == root_hash_bytes.as_slice() {
                return Ok(GetRootIndexFromHashResponse {
                    root_index: i as u32,
                });
            }
        }
    }

    Err(StdError::generic_err("Root not found"))
}

fn query_calculate_domain_separator(
    _deps: Deps,
    chain_id: String,
    contract_addr: String,
) -> StdResult<CalculateDomainSeparatorResponse> {
    let contract_address = Addr::unchecked(contract_addr);
    let domain_separator = calculate_domain_separator(&chain_id, &contract_address);
    let domain_separator_binary = Binary::from(domain_separator);

    Ok(CalculateDomainSeparatorResponse {
        domain_separator_binary,
    })
}

fn query_merkleize_leaves(leaves: Vec<Binary>) -> StdResult<MerkleizeLeavesResponse> {
    let leaf_hashes: Vec<Vec<u8>> = leaves.iter().map(|leaf| leaf.to_vec()).collect();

    if !leaf_hashes.len().is_power_of_two() {
        return Err(StdError::generic_err("Invalid number of leaves"));
    }

    let root_hash = merkleize_sha256(leaf_hashes);
    let root_hash_binary = Binary::from(root_hash);

    Ok(MerkleizeLeavesResponse { root_hash_binary })
}

pub fn query_check_claim(
    deps: Deps,
    env: Env,
    claim: RewardsMerkleClaim,
) -> StdResult<CheckClaimResponse> {
    let check_claim =
        check_claim(env, deps, claim).map_err(|err| StdError::generic_err(format!("{:?}", err)))?;

    Ok(CheckClaimResponse { check_claim })
}

fn only_rewards_updater(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let rewards_updater = REWARDS_UPDATER.load(deps.storage)?;

    if info.sender != rewards_updater {
        return Err(ContractError::NotRewardsUpdater {});
    }
    Ok(())
}

fn only_rewards_for_all_submitter(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let is_submitter = REWARDS_FOR_ALL_SUBMITTER
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(false);
    if !is_submitter {
        return Err(ContractError::ValidCreateRewardsForAllSubmission {});
    }
    Ok(())
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn validate_rewards_submission(
    deps: &Deps,
    submission: &RewardsSubmission,
    env: &Env,
) -> Result<Response, ContractError> {
    if submission.strategies_and_multipliers.is_empty() {
        return Err(ContractError::NoStrategiesSet {});
    }
    if submission.amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }
    if submission.amount > MAX_REWARDS_AMOUNT.into() {
        return Err(ContractError::AmountTooLarge {});
    }

    let max_rewards_duration = MAX_REWARDS_DURATION.load(deps.storage)?;
    if submission.duration > max_rewards_duration {
        return Err(ContractError::ExceedsMaxRewardsDuration {});
    }

    let calc_interval_seconds = CALCULATION_INTERVAL_SECONDS.load(deps.storage)?;
    if submission.duration % calc_interval_seconds != 0 {
        return Err(ContractError::DurationMustBeMultipleOfCalcIntervalSec {});
    }

    if submission.start_timestamp.seconds() % calc_interval_seconds != 0 {
        return Err(ContractError::TimeMustBeMultipleOfCalcIntervalSec {});
    }

    let max_retroactive_length = MAX_RETROACTIVE_LENGTH.load(deps.storage)?;
    let genesis_rewards_timestamp = GENESIS_REWARDS_TIMESTAMP.load(deps.storage)?;
    if env.block.time.seconds() - max_retroactive_length > submission.start_timestamp.seconds()
        || submission.start_timestamp.seconds() < genesis_rewards_timestamp
    {
        return Err(ContractError::StartTimestampTooFarInPast {});
    }

    let max_future_length = MAX_FUTURE_LENGTH.load(deps.storage)?;
    if submission.start_timestamp.seconds() > env.block.time.seconds() + max_future_length {
        return Err(ContractError::StartTimestampTooFarInFuture {});
    }

    let mut current_address = Addr::unchecked("");

    let strategy_manager = STRATEGY_MANAGER.load(deps.storage)?;

    for strategy_multiplier in &submission.strategies_and_multipliers {
        let strategy = &strategy_multiplier.strategy;

        let whitelisted_response: StrategyWhitelistedResponse = deps.querier.query_wasm_smart(
            strategy_manager.clone(),
            &StrategyManagerQueryMsg::IsStrategyWhitelisted {
                strategy: strategy.to_string(),
            },
        )?;

        let whitelisted = whitelisted_response.is_whitelisted;

        if !whitelisted {
            return Err(ContractError::InvalidStrategyConsidered {});
        }

        if current_address >= *strategy {
            return Err(ContractError::StrategiesMuseBeHandleDuplicates {});
        }
        current_address = strategy.clone();
    }

    Ok(Response::new().add_attribute("action", "validate_rewards_submission"))
}

fn check_claim_internal(
    env: Env,
    claim: &RewardsMerkleClaim,
    root: &DistributionRoot,
) -> Result<(), ContractError> {
    if root.disabled {
        return Err(ContractError::RootDisabled {});
    }

    if env.block.time.seconds() < root.activated_at {
        return Err(ContractError::RootNotActivatedYet {});
    }

    if claim.token_indices.len() != claim.token_tree_proofs.len() {
        return Err(ContractError::TokenIndicesAndProofsMismatch {});
    }

    if claim.token_tree_proofs.len() != claim.token_leaves.len() {
        return Err(ContractError::TokenProofsAndLeavesMismatch {});
    }

    verify_earner_claim_proof(
        root.root.clone(),
        claim.earner_index,
        &claim.earner_tree_proof,
        &claim.earner_leaf,
    )?;

    for i in 0..claim.token_indices.len() {
        verify_token_claim_proof(
            claim.earner_leaf.earner_token_root.clone(),
            claim.token_indices[i],
            &claim.token_tree_proofs[i],
            &claim.token_leaves[i],
        )?;
    }

    Ok(())
}

fn verify_token_claim_proof(
    earner_token_root: Binary,
    token_leaf_index: u32,
    token_proof: &[u8],
    token_leaf: &TokenTreeMerkleLeaf,
) -> Result<(), ContractError> {
    if token_leaf_index >= (1 << (token_proof.len() / 32)) {
        return Err(ContractError::InvalidTokenLeafIndex {});
    }

    let token_leaf_hash = calculate_token_leaf_hash(token_leaf);

    let is_valid_proof = verify_inclusion_sha256(
        token_proof,
        earner_token_root.as_slice(),
        &token_leaf_hash,
        token_leaf_index as u64,
    );

    if !is_valid_proof {
        return Err(ContractError::InvalidTokenClaimProof {});
    }

    Ok(())
}

fn verify_earner_claim_proof(
    root: Binary,
    earner_leaf_index: u32,
    earner_proof: &[u8],
    earner_leaf: &EarnerTreeMerkleLeaf,
) -> Result<(), ContractError> {
    if earner_leaf_index >= (1 << (earner_proof.len() / 32)) {
        return Err(ContractError::InvalidEarnerLeafIndex {});
    }

    let earner_leaf_hash = calculate_earner_leaf_hash(earner_leaf);

    let is_valid_proof = verify_inclusion_sha256(
        earner_proof,
        root.as_slice(),
        &earner_leaf_hash,
        earner_leaf_index as u64,
    );

    if !is_valid_proof {
        return Err(ContractError::InvalidEarnerClaimProof {});
    }

    Ok(())
}

fn set_activation_delay_internal(
    deps: DepsMut,
    new_activation_delay: u32,
) -> Result<Response, ContractError> {
    let current_activation_delay = ACTIVATION_DELAY.may_load(deps.storage)?.unwrap_or(0);

    let event = Event::new("ActivationDelaySet")
        .add_attribute("old_activation_delay", current_activation_delay.to_string())
        .add_attribute("new_activation_delay", new_activation_delay.to_string());

    ACTIVATION_DELAY.save(deps.storage, &new_activation_delay)?;

    Ok(Response::new().add_event(event))
}

fn set_global_operator_commission_internal(
    deps: DepsMut,
    new_commission_bips: u16,
) -> Result<Response, ContractError> {
    let current_commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS
        .may_load(deps.storage)?
        .unwrap_or(0);

    GLOBAL_OPERATOR_COMMISSION_BIPS.save(deps.storage, &new_commission_bips)?;

    let event = Event::new("GlobalCommissionBipsSet")
        .add_attribute("old_commission_bips", current_commission_bips.to_string())
        .add_attribute("new_commission_bips", new_commission_bips.to_string());

    Ok(Response::new().add_event(event))
}

fn set_rewards_updater_internal(
    deps: DepsMut,
    new_updater: Addr,
) -> Result<Response, ContractError> {
    REWARDS_UPDATER.save(deps.storage, &new_updater)?;

    let event = Event::new("SetRewardsUpdater")
        .add_attribute("method", "set_rewards_updater")
        .add_attribute("new_updater", new_updater.to_string());

    Ok(Response::new().add_event(event))
}

fn token_balance(querier: &QuerierWrapper, token: &Addr, account: &Addr) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::Balance {
            address: account.to_string(),
        })?,
    }))?;
    Ok(res.balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::DistributionRoot;
    use crate::utils::{
        sha256, ExecuteEarnerTreeMerkleLeaf, ExecuteRewardsMerkleClaim, StrategyAndMultiplier,
    };
    use base64::{engine::general_purpose, Engine as _};
    use bvs_base::roles::{PAUSER, UNPAUSER};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_dependencies_with_balances, mock_env, MockApi,
        MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, coins, from_json, Addr, Binary, ContractResult, OwnedDeps, SystemError, SystemResult,
        Timestamp, WasmQuery,
    };

    type OwnedDepsType = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let initial_owner = deps.api.addr_make("initial_owner").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();
        let rewards_updater = deps.api.addr_make("rewards_updater").to_string();

        let msg = InstantiateMsg {
            initial_owner: initial_owner.clone(),
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            delegation_manager: delegation_manager.clone(),
            strategy_manager: strategy_manager.clone(),
            rewards_updater: rewards_updater.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
            activation_delay: 60,
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, initial_owner.to_string());
        assert_eq!(res.attributes[2].key, "rewards_updater");
        assert_eq!(res.attributes[2].value, rewards_updater.to_string());
        assert_eq!(res.attributes[3].key, "activation_delay");
        assert_eq!(res.attributes[3].value, "60");

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, Addr::unchecked(initial_owner));

        let stored_calculation_interval = CALCULATION_INTERVAL_SECONDS.load(&deps.storage).unwrap();
        assert_eq!(stored_calculation_interval, 86_400);

        let stored_max_rewards_duration = MAX_REWARDS_DURATION.load(&deps.storage).unwrap();
        assert_eq!(stored_max_rewards_duration, 30 * 86_400);

        let stored_max_retroactive_length = MAX_RETROACTIVE_LENGTH.load(&deps.storage).unwrap();
        assert_eq!(stored_max_retroactive_length, 5 * 86_400);

        let stored_max_future_length = MAX_FUTURE_LENGTH.load(&deps.storage).unwrap();
        assert_eq!(stored_max_future_length, 10 * 86_400);

        let stored_genesis_rewards_timestamp =
            GENESIS_REWARDS_TIMESTAMP.load(&deps.storage).unwrap();
        assert_eq!(
            stored_genesis_rewards_timestamp,
            msg.genesis_rewards_timestamp
        );

        let stored_delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(
            stored_delegation_manager,
            Addr::unchecked(delegation_manager)
        );

        let stored_strategy_manager = STRATEGY_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(stored_strategy_manager, Addr::unchecked(strategy_manager));

        let stored_activation_delay = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay, 60);

        let stored_rewards_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_rewards_updater, Addr::unchecked(rewards_updater));

        let stored_paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(stored_paused_state, 0);
    }

    fn instantiate_contract() -> (
        OwnedDepsType,
        Env,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("initial_owner").to_string();
        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();
        let rewards_updater = deps.api.addr_make("rewards_updater").to_string();

        let owner_info = message_info(&Addr::unchecked(initial_owner.clone()), &[]);
        let pauser_info = message_info(&Addr::unchecked(pauser.clone()), &[]);
        let unpauser_info = message_info(&Addr::unchecked(unpauser.clone()), &[]);
        let strategy_manager_info = message_info(&Addr::unchecked(strategy_manager.clone()), &[]);
        let delegation_manager_info =
            message_info(&Addr::unchecked(delegation_manager.clone()), &[]);
        let rewards_updater_info = message_info(&Addr::unchecked(rewards_updater.clone()), &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.clone(),
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            delegation_manager: delegation_manager.clone(),
            strategy_manager: strategy_manager.clone(),
            rewards_updater: rewards_updater.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
            activation_delay: 60, // 1 minute
        };

        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        (
            deps,
            env,
            owner_info,
            pauser_info,
            unpauser_info,
            strategy_manager_info,
            delegation_manager_info,
            rewards_updater_info,
        )
    }

    #[test]
    fn test_only_rewards_updater_success() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");

        REWARDS_UPDATER
            .save(&mut deps.storage, &(rewards_updater_addr))
            .unwrap();

        let info = message_info(&Addr::unchecked(rewards_updater_addr), &[]);
        let result = only_rewards_updater(deps.as_ref(), &info);

        assert!(result.is_ok());
    }

    #[test]
    fn test_only_rewards_updater_failure() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let rewards_updater_addr = deps.api.addr_make("rewards_updater");

        REWARDS_UPDATER
            .save(&mut deps.storage, &rewards_updater_addr)
            .unwrap();

        let info = message_info(&Addr::unchecked("not_rewards_updater"), &[]);
        let result = only_rewards_updater(deps.as_ref(), &info);

        assert_eq!(result, Err(ContractError::NotRewardsUpdater {}));
    }

    #[test]
    fn test_only_rewards_for_all_submitter() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let valid_submitter = deps.api.addr_make("valid_submitter");
        REWARDS_FOR_ALL_SUBMITTER
            .save(&mut deps.storage, valid_submitter.clone(), &true)
            .unwrap();

        let info = message_info(&Addr::unchecked(valid_submitter), &[]);
        let result = only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert!(result.is_ok());

        let invalid_submitter = deps.api.addr_make("invalid_submitter");
        REWARDS_FOR_ALL_SUBMITTER
            .save(&mut deps.storage, invalid_submitter.clone(), &false)
            .unwrap();

        let info = message_info(&Addr::unchecked("invalid_submitter"), &[]);
        let result = only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert_eq!(
            result,
            Err(ContractError::ValidCreateRewardsForAllSubmission {})
        );

        let info = message_info(&Addr::unchecked("unset_submitter"), &[]);
        let result = only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert_eq!(
            result,
            Err(ContractError::ValidCreateRewardsForAllSubmission {})
        );
    }

    #[test]
    fn test_only_owner() {
        let (
            deps,
            _env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let result = only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

        let info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = only_owner(deps.as_ref(), &info);
        assert_eq!(result, Err(ContractError::Unauthorized {}));
    }

    #[test]
    fn test_validate_rewards_submission() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let calc_interval = 86_400; // 1 day

        let block_time = mock_env().block.time.seconds();

        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);

        let valid_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: deps.api.addr_make("strategy1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: deps.api.addr_make("token"),
        };

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if Addr::unchecked(contract_addr) == deps.api.addr_make("strategy_manager") =>
            {
                let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyManagerQueryMsg::IsStrategyWhitelisted { strategy } => {
                        if strategy == deps.api.addr_make("strategy1").to_string() {
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&StrategyWhitelistedResponse {
                                    is_whitelisted: true,
                                })
                                .unwrap(),
                            ))
                        } else {
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&StrategyWhitelistedResponse {
                                    is_whitelisted: false,
                                })
                                .unwrap(),
                            ))
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

        let result = validate_rewards_submission(&deps.as_ref(), &valid_submission, &mock_env());
        assert!(result.is_ok());

        let no_strategy_submission = RewardsSubmission {
            strategies_and_multipliers: vec![],
            amount: Uint128::new(100),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };

        let result =
            validate_rewards_submission(&deps.as_ref(), &no_strategy_submission, &mock_env());
        assert!(matches!(result, Err(ContractError::NoStrategiesSet {})));

        let zero_amount_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy1"),
                multiplier: 1,
            }],
            amount: Uint128::zero(),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };

        let result =
            validate_rewards_submission(&deps.as_ref(), &zero_amount_submission, &mock_env());
        assert!(matches!(result, Err(ContractError::AmountCannotBeZero {})));

        let exceeds_duration_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: 30 * calc_interval + 1,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };

        let result =
            validate_rewards_submission(&deps.as_ref(), &exceeds_duration_submission, &mock_env());
        assert!(matches!(
            result,
            Err(ContractError::ExceedsMaxRewardsDuration {})
        ));
    }

    #[test]
    fn test_create_bvs_rewards_submission() {
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let calc_interval = 86_400; // 1 day

        let block_time = mock_env().block.time.seconds();

        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);

        let submission = vec![RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: deps.api.addr_make("strategy1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval, // 1 day
            start_timestamp: aligned_start_timestamp,
            token: deps.api.addr_make("token"),
        }];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if Addr::unchecked(contract_addr) == deps.api.addr_make("strategy_manager") =>
            {
                let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyManagerQueryMsg::IsStrategyWhitelisted { strategy } => {
                        let response = if strategy == deps.api.addr_make("strategy1").to_string() {
                            StrategyWhitelistedResponse {
                                is_whitelisted: true,
                            }
                        } else {
                            StrategyWhitelistedResponse {
                                is_whitelisted: false,
                            }
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
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

        let msg = ExecuteMsg::CreateBvsRewardsSubmission {
            rewards_submissions: submission,
        };

        let result = execute(deps.as_mut(), env.clone(), owner_info.clone(), msg);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "BVSRewardsSubmissionCreated");
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "sender");
        assert_eq!(
            event.attributes[0].value,
            deps.api.addr_make("initial_owner").to_string()
        );
        assert_eq!(event.attributes[1].key, "nonce");
        assert_eq!(event.attributes[1].value, "0");
        assert_eq!(event.attributes[2].key, "rewards_submission_hash");
        assert_eq!(
            event.attributes[2].value,
            "FWMKFRHYNewOAaP1Ol9hEq89dlsUbU9m3PehWbAwIi8="
        );
        assert_eq!(event.attributes[3].key, "token");
        assert_eq!(
            event.attributes[3].value,
            deps.api.addr_make("token").to_string()
        );
        assert_eq!(event.attributes[4].key, "amount");
        assert_eq!(event.attributes[4].value, "100");
    }

    #[test]
    fn test_create_rewards_for_all_submission() {
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let calc_interval = 86_400; // 1 day

        let block_time = mock_env().block.time.seconds();

        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);

        let submission = vec![RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: deps.api.addr_make("strategy1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval, // 1 day
            start_timestamp: aligned_start_timestamp,
            token: deps.api.addr_make("token"),
        }];

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if Addr::unchecked(contract_addr) == deps.api.addr_make("strategy_manager") =>
            {
                let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyManagerQueryMsg::IsStrategyWhitelisted { strategy } => {
                        let response = if strategy == deps.api.addr_make("strategy1").to_string() {
                            StrategyWhitelistedResponse {
                                is_whitelisted: true,
                            }
                        } else {
                            StrategyWhitelistedResponse {
                                is_whitelisted: false,
                            }
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
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

        let msg_set_submitter = ExecuteMsg::SetRewardsForAllSubmitter {
            submitter: deps.api.addr_make("submitter").to_string(),
            new_value: true,
        };

        let _ = execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            msg_set_submitter,
        );

        let msg = ExecuteMsg::CreateRewardsForAllSubmission {
            rewards_submissions: submission,
        };

        let submmiter_info = message_info(&deps.api.addr_make("submitter"), &[]);

        let result = execute(deps.as_mut(), env.clone(), submmiter_info.clone(), msg);

        if let Err(err) = &result {
            println!("Error: {:?}", err);
        }

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "RewardsSubmissionForAllCreated");
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "sender");
        assert_eq!(
            event.attributes[0].value,
            deps.api.addr_make("submitter").to_string()
        );
        assert_eq!(event.attributes[1].key, "nonce");
        assert_eq!(event.attributes[1].value, "0");
        assert_eq!(event.attributes[2].key, "rewards_submission_hash");
        assert_eq!(
            event.attributes[2].value,
            "6iTJDz8b/ym1GayJcb5UVJB1h+3Pab9z07oOboL8kfU="
        );
        assert_eq!(event.attributes[3].key, "token");
        assert_eq!(
            event.attributes[3].value,
            deps.api.addr_make("token").to_string()
        );
        assert_eq!(event.attributes[4].key, "amount");
        assert_eq!(event.attributes[4].value, "100");
    }

    #[test]
    fn test_transfer_ownership() {
        let (
            mut deps,
            _env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let new_owner = deps.api.addr_make("new_owner");

        let result = transfer_ownership(deps.as_mut(), owner_info, new_owner.clone());

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "TransferOwnership");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "transfer_ownership");
        assert_eq!(event.attributes[1].key, "new_owner");
        assert_eq!(event.attributes[1].value, new_owner.to_string());

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized_caller"), &[]);

        let result = transfer_ownership(deps.as_mut(), info_unauthorized, new_owner.clone());

        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);
    }

    #[test]
    fn test_set_activation_delay_internal() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let initial_activation_delay: u32 = 60; // 1 minute
        ACTIVATION_DELAY
            .save(&mut deps.storage, &initial_activation_delay)
            .unwrap();

        let new_activation_delay: u32 = 120; // 2 minutes

        let result = set_activation_delay_internal(deps.as_mut(), new_activation_delay);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "ActivationDelaySet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_activation_delay");
        assert_eq!(
            event.attributes[0].value,
            initial_activation_delay.to_string()
        );
        assert_eq!(event.attributes[1].key, "new_activation_delay");
        assert_eq!(event.attributes[1].value, new_activation_delay.to_string());

        let stored_activation_delay = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay, new_activation_delay);
    }

    #[test]
    fn test_set_activation_delay() {
        let (
            mut deps,
            _env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let initial_activation_delay: u32 = 60; // 1 minute
        ACTIVATION_DELAY
            .save(&mut deps.storage, &initial_activation_delay)
            .unwrap();

        let new_activation_delay: u32 = 120; // 2 minutes

        let result = set_activation_delay(deps.as_mut(), owner_info.clone(), new_activation_delay);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "ActivationDelaySet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_activation_delay");
        assert_eq!(
            event.attributes[0].value,
            initial_activation_delay.to_string()
        );
        assert_eq!(event.attributes[1].key, "new_activation_delay");
        assert_eq!(event.attributes[1].value, new_activation_delay.to_string());

        let stored_activation_delay = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay, new_activation_delay);

        let unauthorized_info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = set_activation_delay(deps.as_mut(), unauthorized_info, new_activation_delay);

        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }

        let stored_activation_delay_after_unauthorized_attempt =
            ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(
            stored_activation_delay_after_unauthorized_attempt,
            new_activation_delay
        );
    }

    #[test]
    fn test_set_rewards_updater_internal() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let new_updater = deps.api.addr_make("new_updater");

        let result = set_rewards_updater_internal(deps.as_mut(), new_updater.clone());

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "SetRewardsUpdater");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_rewards_updater");
        assert_eq!(event.attributes[1].key, "new_updater");
        assert_eq!(event.attributes[1].value, new_updater.to_string());

        let stored_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_updater, new_updater);
    }

    #[test]
    fn test_set_rewards_updater() {
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let new_updater = deps.api.addr_make("new_updater");

        let msg = ExecuteMsg::SetRewardsUpdater {
            new_updater: new_updater.to_string(),
        };

        let result = execute(deps.as_mut(), env, owner_info, msg);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "SetRewardsUpdater");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_rewards_updater");
        assert_eq!(event.attributes[1].key, "new_updater");
        assert_eq!(event.attributes[1].value, new_updater.to_string());

        let stored_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_updater, new_updater);

        let unauthorized_info = message_info(&Addr::unchecked("not_owner"), &[]);
        let result = set_rewards_updater(
            deps.as_mut(),
            unauthorized_info,
            Addr::unchecked("another_updater"),
        );

        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }

        let stored_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_updater, new_updater);
    }

    #[test]
    fn test_set_rewards_for_all_submitter() {
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let submitter = deps.api.addr_make("submitter");
        let initial_value = false;

        REWARDS_FOR_ALL_SUBMITTER
            .save(&mut deps.storage, submitter.clone(), &initial_value)
            .unwrap();

        let msg = ExecuteMsg::SetRewardsForAllSubmitter {
            submitter: submitter.to_string(),
            new_value: true,
        };

        let result = execute(deps.as_mut(), env.clone(), owner_info, msg.clone());

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.attributes.len(), 4);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(
            response.attributes[0].value,
            "set_rewards_for_all_submitter"
        );
        assert_eq!(response.attributes[1].key, "submitter");
        assert_eq!(response.attributes[1].value, submitter.to_string());
        assert_eq!(response.attributes[2].key, "previous_value");
        assert_eq!(response.attributes[2].value, initial_value.to_string());
        assert_eq!(response.attributes[3].key, "new_value");
        assert_eq!(response.attributes[3].value, "true");

        let stored_value = REWARDS_FOR_ALL_SUBMITTER
            .load(&deps.storage, submitter.clone())
            .unwrap();
        assert!(stored_value);

        let unauthorized_info = message_info(&Addr::unchecked("not_owner"), &[]);

        let result = execute(deps.as_mut(), env, unauthorized_info, msg);

        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }

        let stored_value = REWARDS_FOR_ALL_SUBMITTER
            .load(&deps.storage, submitter)
            .unwrap();

        assert!(stored_value);
    }

    #[test]
    fn test_set_global_operator_commission_internal() {
        let (
            mut deps,
            _env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let initial_commission_bips = 100;
        GLOBAL_OPERATOR_COMMISSION_BIPS
            .save(&mut deps.storage, &initial_commission_bips)
            .unwrap();

        let new_commission_bips = 150;

        let result = set_global_operator_commission_internal(deps.as_mut(), new_commission_bips);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "GlobalCommissionBipsSet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_commission_bips");
        assert_eq!(
            event.attributes[0].value,
            initial_commission_bips.to_string()
        );
        assert_eq!(event.attributes[1].key, "new_commission_bips");
        assert_eq!(event.attributes[1].value, new_commission_bips.to_string());

        let stored_commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS.load(&deps.storage).unwrap();
        assert_eq!(stored_commission_bips, new_commission_bips);
    }

    #[test]
    fn test_set_global_operator_commission() {
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let initial_commission_bips = 100;
        GLOBAL_OPERATOR_COMMISSION_BIPS
            .save(&mut deps.storage, &initial_commission_bips)
            .unwrap();

        let new_commission_bips = 150;

        let msg = ExecuteMsg::SetGlobalOperatorCommission {
            new_commission_bips,
        };

        let result = execute(deps.as_mut(), env.clone(), owner_info.clone(), msg.clone());

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "GlobalCommissionBipsSet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_commission_bips");
        assert_eq!(
            event.attributes[0].value,
            initial_commission_bips.to_string()
        );
        assert_eq!(event.attributes[1].key, "new_commission_bips");
        assert_eq!(event.attributes[1].value, new_commission_bips.to_string());

        let stored_commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS.load(&deps.storage).unwrap();
        assert_eq!(stored_commission_bips, new_commission_bips);

        let info_not_owner = message_info(&Addr::unchecked("info_not_owner"), &[]);

        let result = execute(deps.as_mut(), env, info_not_owner, msg);
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }
    }

    #[test]
    fn test_calculate_token_leaf_hash() {
        let (
            deps,
            env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let msg = QueryMsg::CalculateTokenLeafHash {
            token: deps.api.addr_make("token_a").to_string(),
            cumulative_earnings: Uint128::new(100),
        };

        let result = query(deps.as_ref(), env, msg);

        assert!(result.is_ok());
    }

    #[test]
    fn test_token_leaf_merkle_tree_construction() {
        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        let leaves = [
            Binary::from(hash_a.clone()),
            Binary::from(hash_b.clone()),
            Binary::from(hash_c.clone()),
            Binary::from(hash_d.clone()),
        ];

        let msg = QueryMsg::MerkleizeLeaves {
            leaves: leaves.iter().map(|leaf| leaf.to_base64()).collect(),
        };

        let deps = mock_dependencies();
        let env = mock_env();

        let res: MerkleizeLeavesResponse =
            from_json(query(deps.as_ref(), env, msg).unwrap()).unwrap();

        let merkle_root = res.root_hash_binary.to_vec();

        // Expected parent hash & Expected root hash
        let leaves_ab = vec![hash_a.clone(), hash_b.clone()];
        let parent_ab = merkleize_sha256(leaves_ab.clone());

        let leaves_cd = vec![hash_c.clone(), hash_d.clone()];
        let parent_cd = merkleize_sha256(leaves_cd.clone());

        let parent_hash = vec![parent_ab.clone(), parent_cd.clone()];
        let expected_root_hash = merkleize_sha256(parent_hash.clone());

        assert!(!merkle_root.is_empty(), "Merkle root should not be empty");
        assert_eq!(merkle_root, expected_root_hash);

        assert_eq!(
            parent_ab,
            sha256(&[hash_a.as_slice(), hash_b.as_slice()].concat()),
            "Parent AB hash is incorrect"
        );
        assert_eq!(
            parent_cd,
            sha256(&[hash_c.as_slice(), hash_d.as_slice()].concat()),
            "Parent CD hash is incorrect"
        );

        println!("Hash A: {:?}", hash_a);
        println!("Hash B: {:?}", hash_b);
        println!("Parent AB: {:?}", parent_ab);
        println!("Hash C: {:?}", hash_c);
        println!("Hash D: {:?}", hash_d);
        println!("Parent CD: {:?}", parent_cd);
        println!("Merkle Root: {:?}", merkle_root);
    }

    #[test]
    fn test_earner_leaf_merkle_tree_construction() {
        let token_leaves_sets = vec![
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a1"),
                    cumulative_earnings: Uint128::new(100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a2"),
                    cumulative_earnings: Uint128::new(200),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a3"),
                    cumulative_earnings: Uint128::new(300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a4"),
                    cumulative_earnings: Uint128::new(400),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b1"),
                    cumulative_earnings: Uint128::new(500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b2"),
                    cumulative_earnings: Uint128::new(600),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b3"),
                    cumulative_earnings: Uint128::new(700),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b4"),
                    cumulative_earnings: Uint128::new(800),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c1"),
                    cumulative_earnings: Uint128::new(900),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c2"),
                    cumulative_earnings: Uint128::new(1000),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c3"),
                    cumulative_earnings: Uint128::new(1100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c4"),
                    cumulative_earnings: Uint128::new(1200),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d1"),
                    cumulative_earnings: Uint128::new(1300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d2"),
                    cumulative_earnings: Uint128::new(1400),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d3"),
                    cumulative_earnings: Uint128::new(1500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d4"),
                    cumulative_earnings: Uint128::new(1600),
                },
            ],
        ];

        // Calculate Merkle roots for each set of token leaves
        let mut merkle_roots = Vec::new();

        for leaves in token_leaves_sets {
            let mut leaf_hashes = Vec::new();
            for leaf in leaves {
                leaf_hashes.push(calculate_token_leaf_hash(&leaf));
            }
            let merkle_root = merkleize_sha256(leaf_hashes);
            merkle_roots.push(merkle_root.clone());

            println!("Merkle Root: {:?}", merkle_root);
        }

        // Assertions & Print root hash for calculate_earner_leaf_hash
        for (i, merkle_root) in merkle_roots.iter().enumerate() {
            assert!(
                !merkle_root.is_empty(),
                "Merkle root for tree {} should not be empty",
                i + 1
            );
            println!("Merkle Root for Tree {}: {:?}", i + 1, merkle_root);
        }

        let tree1_root_hash = [
            48, 187, 24, 98, 230, 203, 235, 218, 90, 43, 190, 153, 209, 248, 126, 128, 198, 194,
            113, 131, 32, 46, 106, 102, 115, 45, 214, 230, 122, 67, 222, 244,
        ];
        let tree2_root_hash = [
            31, 173, 229, 179, 199, 27, 21, 153, 215, 61, 227, 184, 156, 136, 11, 226, 144, 224,
            214, 117, 192, 110, 116, 32, 123, 117, 254, 131, 59, 205, 178, 221,
        ];
        let tree3_root_hash = [
            241, 77, 172, 5, 228, 0, 249, 31, 159, 211, 176, 37, 20, 123, 30, 159, 62, 148, 250,
            97, 101, 206, 14, 35, 211, 217, 181, 123, 237, 149, 14, 220,
        ];
        let tree4_root_hash = [
            114, 34, 142, 99, 115, 93, 244, 227, 187, 171, 41, 53, 218, 109, 87, 55, 75, 87, 46,
            220, 50, 151, 15, 77, 78, 255, 183, 253, 198, 47, 244, 132,
        ];

        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: Binary::from(tree1_root_hash.to_vec()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: Binary::from(tree2_root_hash.to_vec()),
        };
        let earner3 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner3"),
            earner_token_root: Binary::from(tree3_root_hash.to_vec()),
        };
        let earner4 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner4"),
            earner_token_root: Binary::from(tree4_root_hash.to_vec()),
        };

        // Calculate earner leaf hashes
        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);
        let earner_leaf_hash3 = calculate_earner_leaf_hash(&earner3);
        let earner_leaf_hash4 = calculate_earner_leaf_hash(&earner4);

        let leaves = vec![
            earner_leaf_hash1.clone(),
            earner_leaf_hash2.clone(),
            earner_leaf_hash3.clone(),
            earner_leaf_hash4.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        // Expected parent hash & Expected root hash
        let leaves_1_2 = vec![earner_leaf_hash1.clone(), earner_leaf_hash2.clone()];
        let parent_1_2 = merkleize_sha256(leaves_1_2.clone());

        let leaves_3_4 = vec![earner_leaf_hash3.clone(), earner_leaf_hash4.clone()];
        let parent_3_4 = merkleize_sha256(leaves_3_4.clone());

        let parent_hash = vec![parent_1_2.clone(), parent_3_4.clone()];
        let expected_root_hash = merkleize_sha256(parent_hash.clone());

        assert!(!merkle_root.is_empty(), "Merkle root should not be empty");
        assert_eq!(merkle_root, expected_root_hash);

        assert_eq!(
            parent_1_2,
            sha256(&[earner_leaf_hash1.as_slice(), earner_leaf_hash2.as_slice()].concat()),
            "Parent 1 2 hash is incorrect"
        );
        assert_eq!(
            parent_3_4,
            sha256(&[earner_leaf_hash3.as_slice(), earner_leaf_hash4.as_slice()].concat()),
            "Parent 3 4 hash is incorrect"
        );

        println!("earner_leaf_hash1: {:?}", earner_leaf_hash1);
        println!("earner_leaf_hash2: {:?}", earner_leaf_hash2);
        println!("parent_1_2: {:?}", parent_1_2);
        println!("earner_leaf_hash3: {:?}", earner_leaf_hash3);
        println!("earner_leaf_hash4: {:?}", earner_leaf_hash4);
        println!("parent_3_4: {:?}", parent_3_4);
        println!("Merkle Root: {:?}", merkle_root);
    }

    #[test]
    fn test_verify_inclusion_proof() {
        let token_leaves_sets = vec![
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a1"),
                    cumulative_earnings: Uint128::new(100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a2"),
                    cumulative_earnings: Uint128::new(200),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a3"),
                    cumulative_earnings: Uint128::new(300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a4"),
                    cumulative_earnings: Uint128::new(400),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b1"),
                    cumulative_earnings: Uint128::new(500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b2"),
                    cumulative_earnings: Uint128::new(600),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b3"),
                    cumulative_earnings: Uint128::new(700),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b4"),
                    cumulative_earnings: Uint128::new(800),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c1"),
                    cumulative_earnings: Uint128::new(900),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c2"),
                    cumulative_earnings: Uint128::new(1000),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c3"),
                    cumulative_earnings: Uint128::new(1100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c4"),
                    cumulative_earnings: Uint128::new(1200),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d1"),
                    cumulative_earnings: Uint128::new(1300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d2"),
                    cumulative_earnings: Uint128::new(1400),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d3"),
                    cumulative_earnings: Uint128::new(1500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d4"),
                    cumulative_earnings: Uint128::new(1600),
                },
            ],
        ];

        let mut merkle_roots = Vec::new();

        for leaves in &token_leaves_sets {
            let mut leaf_hashes = Vec::new();
            for leaf in leaves {
                leaf_hashes.push(calculate_token_leaf_hash(leaf));
            }
            let merkle_root = merkleize_sha256(leaf_hashes.clone());
            merkle_roots.push(merkle_root.clone());

            println!("Merkle Root: {:?}", merkle_root);
        }

        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: Binary::from(merkle_roots[0].clone()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: Binary::from(merkle_roots[1].clone()),
        };
        let earner3 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner3"),
            earner_token_root: Binary::from(merkle_roots[2].clone()),
        };
        let earner4 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner4"),
            earner_token_root: Binary::from(merkle_roots[3].clone()),
        };

        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);
        let earner_leaf_hash3 = calculate_earner_leaf_hash(&earner3);
        let earner_leaf_hash4 = calculate_earner_leaf_hash(&earner4);

        let leaves = vec![
            earner_leaf_hash1.clone(),
            earner_leaf_hash2.clone(),
            earner_leaf_hash3.clone(),
            earner_leaf_hash4.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        let leaves_3_4 = vec![earner_leaf_hash3.clone(), earner_leaf_hash4.clone()];
        let parent_3_4 = merkleize_sha256(leaves_3_4.clone());

        let leaves_1_2 = vec![earner_leaf_hash1.clone(), earner_leaf_hash2.clone()];
        let parent_1_2 = merkleize_sha256(leaves_1_2.clone());

        // Generate proof for earner1 leaf
        let proof1 = [earner_leaf_hash2.clone(), parent_3_4.clone()];
        let proof2 = [earner_leaf_hash1.clone(), parent_3_4.clone()];
        let proof3 = [earner_leaf_hash4.clone(), parent_1_2.clone()];
        let proof4 = [earner_leaf_hash3.clone(), parent_1_2.clone()];

        assert!(verify_inclusion_sha256(
            &proof1.concat(),
            &merkle_root,
            &earner_leaf_hash1,
            0
        ));

        assert!(verify_inclusion_sha256(
            &proof2.concat(),
            &merkle_root,
            &earner_leaf_hash2,
            1
        ));

        assert!(verify_inclusion_sha256(
            &proof3.concat(),
            &merkle_root,
            &earner_leaf_hash3,
            2
        ));

        assert!(verify_inclusion_sha256(
            &proof4.concat(),
            &merkle_root,
            &earner_leaf_hash4,
            3
        ));
    }

    #[test]
    fn test_verify_earner_claim_proof() {
        let token_leaves_sets = vec![
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a1"),
                    cumulative_earnings: Uint128::new(100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a2"),
                    cumulative_earnings: Uint128::new(200),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a3"),
                    cumulative_earnings: Uint128::new(300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_a4"),
                    cumulative_earnings: Uint128::new(400),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b1"),
                    cumulative_earnings: Uint128::new(500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b2"),
                    cumulative_earnings: Uint128::new(600),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b3"),
                    cumulative_earnings: Uint128::new(700),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_b4"),
                    cumulative_earnings: Uint128::new(800),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c1"),
                    cumulative_earnings: Uint128::new(900),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c2"),
                    cumulative_earnings: Uint128::new(1000),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c3"),
                    cumulative_earnings: Uint128::new(1100),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_c4"),
                    cumulative_earnings: Uint128::new(1200),
                },
            ],
            vec![
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d1"),
                    cumulative_earnings: Uint128::new(1300),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d2"),
                    cumulative_earnings: Uint128::new(1400),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d3"),
                    cumulative_earnings: Uint128::new(1500),
                },
                TokenTreeMerkleLeaf {
                    token: Addr::unchecked("token_d4"),
                    cumulative_earnings: Uint128::new(1600),
                },
            ],
        ];

        // Calculate Merkle roots for each set of token leaves
        let mut merkle_roots = Vec::new();
        for leaves in &token_leaves_sets {
            let mut leaf_hashes = Vec::new();
            for leaf in leaves {
                leaf_hashes.push(calculate_token_leaf_hash(leaf));
            }
            let merkle_root = merkleize_sha256(leaf_hashes.clone());
            merkle_roots.push(merkle_root.clone());
        }

        // Setup earner leaves
        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: Binary::from(merkle_roots[0].clone()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: Binary::from(merkle_roots[1].clone()),
        };
        let earner3 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner3"),
            earner_token_root: Binary::from(merkle_roots[2].clone()),
        };
        let earner4 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner4"),
            earner_token_root: Binary::from(merkle_roots[3].clone()),
        };

        // Calculate earner leaf hashes
        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);
        let earner_leaf_hash3 = calculate_earner_leaf_hash(&earner3);
        let earner_leaf_hash4 = calculate_earner_leaf_hash(&earner4);

        let leaves = vec![
            earner_leaf_hash1.clone(),
            earner_leaf_hash2.clone(),
            earner_leaf_hash3.clone(),
            earner_leaf_hash4.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        // Create proofs for earner leaf nodes
        let proof1 = [
            earner_leaf_hash2.clone(),
            sha256(&[earner_leaf_hash3.clone(), earner_leaf_hash4.clone()].concat()),
        ];
        let proof2 = [
            earner_leaf_hash1.clone(),
            sha256(&[earner_leaf_hash3.clone(), earner_leaf_hash4.clone()].concat()),
        ];
        let proof3 = [
            earner_leaf_hash4.clone(),
            sha256(&[earner_leaf_hash1.clone(), earner_leaf_hash2.clone()].concat()),
        ];
        let proof4 = [
            earner_leaf_hash3.clone(),
            sha256(&[earner_leaf_hash1.clone(), earner_leaf_hash2.clone()].concat()),
        ];

        // Verify proofs using _verify_earner_claim_proof function
        assert!(verify_earner_claim_proof(
            Binary::from(merkle_root.clone()),
            0,
            &proof1.concat(),
            &earner1
        )
        .is_ok());
        assert!(verify_earner_claim_proof(
            Binary::from(merkle_root.clone()),
            1,
            &proof2.concat(),
            &earner2
        )
        .is_ok());
        assert!(verify_earner_claim_proof(
            Binary::from(merkle_root.clone()),
            2,
            &proof3.concat(),
            &earner3
        )
        .is_ok());
        assert!(verify_earner_claim_proof(
            Binary::from(merkle_root.clone()),
            3,
            &proof4.concat(),
            &earner4
        )
        .is_ok());
    }

    #[test]
    fn test_verify_token_claim_proof() {
        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        // Calculate hashes for each leaf
        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        // Create the Merkle tree and calculate root
        let leaves = vec![
            hash_a.clone(),
            hash_b.clone(),
            hash_c.clone(),
            hash_d.clone(),
        ];
        let merkle_root = merkleize_sha256(leaves.clone());

        // Calculate parent hashes
        let parent_ab = merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]);
        let parent_cd = merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]);

        // Create proofs for each leaf
        let proof_a = [hash_b.clone(), parent_cd.clone()];
        let proof_b = [hash_a.clone(), parent_cd.clone()];
        let proof_c = [hash_d.clone(), parent_ab.clone()];
        let proof_d = [hash_c.clone(), parent_ab.clone()];

        // Verify proofs using _verify_token_claim_proof function
        assert!(verify_token_claim_proof(
            Binary::from(merkle_root.clone()),
            0,
            &proof_a.concat(),
            &leaf_a
        )
        .is_ok());
        assert!(verify_token_claim_proof(
            Binary::from(merkle_root.clone()),
            1,
            &proof_b.concat(),
            &leaf_b
        )
        .is_ok());
        assert!(verify_token_claim_proof(
            Binary::from(merkle_root.clone()),
            2,
            &proof_c.concat(),
            &leaf_c
        )
        .is_ok());
        assert!(verify_token_claim_proof(
            Binary::from(merkle_root.clone()),
            3,
            &proof_d.concat(),
            &leaf_d
        )
        .is_ok());
    }

    #[test]
    fn test_verify_whole_claim_proof() {
        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        let leaf_a1 = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a1"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b1 = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b1"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c1 = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c1"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d1 = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d1"),
            cumulative_earnings: Uint128::new(400),
        };

        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        let hash_a1 = calculate_token_leaf_hash(&leaf_a1);
        let hash_b1 = calculate_token_leaf_hash(&leaf_b1);
        let hash_c1 = calculate_token_leaf_hash(&leaf_c1);
        let hash_d1 = calculate_token_leaf_hash(&leaf_d1);

        let leaves_a_b = vec![hash_a.clone(), hash_b.clone()];
        let parent_a_b = merkleize_sha256(leaves_a_b.clone());

        let leaves_c_d = vec![hash_c.clone(), hash_d.clone()];
        let parent_c_d = merkleize_sha256(leaves_c_d.clone());

        let root_a_b_c_d = vec![parent_a_b.clone(), parent_c_d.clone()];
        let root_a_b_c_d = merkleize_sha256(root_a_b_c_d.clone());

        let leaves_a1_b1 = vec![hash_a1.clone(), hash_b1.clone()];
        let parent_a1_b1 = merkleize_sha256(leaves_a1_b1.clone());

        let leaves_c1_d1 = vec![hash_c1.clone(), hash_d1.clone()];
        let parent_c1_d1 = merkleize_sha256(leaves_c1_d1.clone());

        let root_a1_b1_c1_d1 = vec![parent_a1_b1.clone(), parent_c1_d1.clone()];
        let root_a1_b1_c1_d1 = merkleize_sha256(root_a1_b1_c1_d1.clone());

        let earner1 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner1"),
            earner_token_root: Binary::from(root_a_b_c_d.clone()),
        };
        let earner2 = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner2"),
            earner_token_root: Binary::from(root_a1_b1_c1_d1.clone()),
        };

        let earner_leaf_hash1 = calculate_earner_leaf_hash(&earner1);
        let earner_leaf_hash2 = calculate_earner_leaf_hash(&earner2);

        let leaves = vec![earner_leaf_hash1.clone(), earner_leaf_hash2.clone()];
        let earner_root = merkleize_sha256(leaves.clone());

        let proof_a = [earner_leaf_hash2.clone()];
        let proof_b = [hash_b.clone(), parent_c_d.clone()];

        assert!(verify_earner_claim_proof(
            Binary::from(earner_root.clone()),
            0,
            &proof_a.concat(),
            &earner1
        )
        .is_ok());
        assert!(verify_token_claim_proof(
            Binary::from(root_a_b_c_d.clone()),
            0,
            &proof_b.concat(),
            &leaf_a
        )
        .is_ok());
    }

    #[test]
    fn test_check_claim() {
        let env = mock_env();

        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        let token_leaves = vec![
            hash_a.clone(),
            hash_b.clone(),
            hash_c.clone(),
            hash_d.clone(),
        ];
        let token_root = merkleize_sha256(token_leaves.clone());

        let earner_leaf = EarnerTreeMerkleLeaf {
            earner: Addr::unchecked("earner"),
            earner_token_root: Binary::from(token_root.clone()),
        };

        let earner_leaf_hash = calculate_earner_leaf_hash(&earner_leaf);

        let earner_leaves = vec![earner_leaf_hash.clone()];
        let earner_root = merkleize_sha256(earner_leaves.clone());

        let distribution_root = DistributionRoot {
            root: Binary::from(earner_root.clone()),
            rewards_calculation_end_timestamp: 500,
            activated_at: 500,
            disabled: false,
        };

        let claim = RewardsMerkleClaim {
            root_index: 0,
            earner_index: 0,
            earner_tree_proof: vec![],
            earner_leaf,
            token_indices: vec![0, 1, 2, 3],
            token_tree_proofs: vec![
                [
                    hash_b.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_a.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_d.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
                [
                    hash_c.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
            ],
            token_leaves: vec![
                leaf_a.clone(),
                leaf_b.clone(),
                leaf_c.clone(),
                leaf_d.clone(),
            ],
        };

        assert!(check_claim_internal(env, &claim, &distribution_root).is_ok());
    }

    #[test]
    fn test_submit_root() {
        let (
            mut deps,
            env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            rewards_updater_info,
        ) = instantiate_contract();

        CURR_REWARDS_CALCULATION_END_TIMESTAMP
            .save(&mut deps.storage, &1000)
            .unwrap();
        ACTIVATION_DELAY.save(&mut deps.storage, &60u32).unwrap();

        let root = Binary::from(b"valid_root".to_vec());
        let root_base64 = root.to_base64();
        let rewards_calculation_end_timestamp = 1100;

        let msg = ExecuteMsg::SubmitRoot {
            root: root_base64.clone(),
            rewards_calculation_end_timestamp,
        };

        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg,
        );

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "DistributionRootSubmitted");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "root_index");
        assert_eq!(event.attributes[0].value, "0");
        assert_eq!(event.attributes[1].key, "root");
        assert_eq!(event.attributes[1].value, format!("{:?}", root));
        assert_eq!(event.attributes[2].key, "rewards_calculation_end_timestamp");
        assert_eq!(
            event.attributes[2].value,
            rewards_calculation_end_timestamp.to_string()
        );
        assert_eq!(event.attributes[3].key, "activated_at");
        assert_eq!(
            event.attributes[3].value,
            (env.block.time.seconds() + 60).to_string()
        );

        let past_timestamp = 500;
        let msg = ExecuteMsg::SubmitRoot {
            root: root_base64.clone(),
            rewards_calculation_end_timestamp: past_timestamp,
        };

        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::InvalidTimestamp {});
        }

        let future_timestamp = env.block.time.seconds() + 100;
        let msg = ExecuteMsg::SubmitRoot {
            root: root_base64.clone(),
            rewards_calculation_end_timestamp: future_timestamp,
        };

        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::TimestampInFuture {});
        }

        let unauthorized_info = message_info(&Addr::unchecked("not_rewards_updater"), &[]);
        let msg = ExecuteMsg::SubmitRoot {
            root: root_base64,
            rewards_calculation_end_timestamp,
        };

        let result = execute(deps.as_mut(), env.clone(), unauthorized_info, msg);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::NotRewardsUpdater {});
        }
    }

    fn setup_test_environment() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies_with_balances(&[
            ("token_a", &coins(1000, "token_a")),
            ("token_b", &coins(1000, "token_b")),
            ("token_c", &coins(1000, "token_c")),
            ("token_d", &coins(1000, "token_d")),
        ]);

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } => {
                if contract_addr == "token_a"
                    || contract_addr == "token_b"
                    || contract_addr == "token_c"
                    || contract_addr == "token_d"
                {
                    let balance_response = cw20::BalanceResponse {
                        balance: Uint128::new(1000),
                    };
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&balance_response).unwrap(),
                    ))
                } else {
                    SystemResult::Err(cosmwasm_std::SystemError::NoSuchContract {
                        addr: contract_addr.clone(),
                    })
                }
            }
            _ => SystemResult::Err(cosmwasm_std::SystemError::Unknown {}),
        });

        let env = mock_env();
        let info = message_info(&deps.api.addr_make("claimer"), &[]);
        (deps, env, info)
    }

    #[test]
    fn test_process_claim() {
        let (mut deps, env, _info) = setup_test_environment();

        let initial_owner = deps.api.addr_make("initial_owner").to_string();
        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();
        let rewards_updater = deps.api.addr_make("rewards_updater").to_string();

        let owner_info = message_info(&Addr::unchecked(initial_owner.clone()), &[]);
        message_info(&Addr::unchecked(delegation_manager.clone()), &[]);

        let msg = InstantiateMsg {
            initial_owner: initial_owner.clone(),
            calculation_interval_seconds: 86_400, // 1 day
            max_rewards_duration: 30 * 86_400,    // 30 days
            max_retroactive_length: 5 * 86_400,   // 5 days
            max_future_length: 10 * 86_400,       // 10 days
            genesis_rewards_timestamp: env.block.time.seconds() / 86_400 * 86_400,
            delegation_manager: delegation_manager.clone(),
            strategy_manager: strategy_manager.clone(),
            rewards_updater: rewards_updater.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
            activation_delay: 60, // 1 minute
        };

        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        let info = message_info(&deps.api.addr_make("claimer"), &[]);

        let leaf_a = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_a"),
            cumulative_earnings: Uint128::new(100),
        };

        let leaf_b = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_b"),
            cumulative_earnings: Uint128::new(200),
        };

        let leaf_c = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_c"),
            cumulative_earnings: Uint128::new(300),
        };

        let leaf_d = TokenTreeMerkleLeaf {
            token: Addr::unchecked("token_d"),
            cumulative_earnings: Uint128::new(400),
        };

        let hash_a = calculate_token_leaf_hash(&leaf_a);
        let hash_b = calculate_token_leaf_hash(&leaf_b);
        let hash_c = calculate_token_leaf_hash(&leaf_c);
        let hash_d = calculate_token_leaf_hash(&leaf_d);

        let token_leaves = vec![
            hash_a.clone(),
            hash_b.clone(),
            hash_c.clone(),
            hash_d.clone(),
        ];
        let token_root = merkleize_sha256(token_leaves.clone());

        let earner_leaf = EarnerTreeMerkleLeaf {
            earner: deps.api.addr_make("earner"),
            earner_token_root: Binary::from(token_root.clone()),
        };

        let earner_leaf_hash = calculate_earner_leaf_hash(&earner_leaf);

        let earner_leaves = vec![earner_leaf_hash.clone()];
        let earner_root = merkleize_sha256(earner_leaves.clone());

        let distribution_root = DistributionRoot {
            root: Binary::from(earner_root.clone()),
            rewards_calculation_end_timestamp: 500,
            activated_at: 500,
            disabled: false,
        };

        let claim = RewardsMerkleClaim {
            root_index: 0,
            earner_index: 0,
            earner_tree_proof: vec![],
            earner_leaf,
            token_indices: vec![0, 1, 2, 3],
            token_tree_proofs: vec![
                [
                    hash_b.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_a.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_d.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
                [
                    hash_c.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
            ],
            token_leaves: vec![
                leaf_a.clone(),
                leaf_b.clone(),
                leaf_c.clone(),
                leaf_d.clone(),
            ],
        };

        DISTRIBUTION_ROOTS
            .save(&mut deps.storage, 0, &distribution_root)
            .unwrap();

        CLAIMER_FOR
            .save(
                &mut deps.storage,
                deps.api.addr_make("earner"),
                &deps.api.addr_make("claimer"),
            )
            .unwrap();

        let recipient = deps.api.addr_make("recipient");

        let earner_leaf = ExecuteEarnerTreeMerkleLeaf {
            earner: deps.api.addr_make("earner").to_string(),
            earner_token_root: general_purpose::STANDARD.encode(token_root),
        };

        let exeute_claim = ExecuteRewardsMerkleClaim {
            root_index: 0,
            earner_index: 0,
            earner_tree_proof: vec![],
            earner_leaf,
            token_indices: vec![0, 1, 2, 3],
            token_tree_proofs: vec![
                [
                    hash_b.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_a.clone(),
                    merkleize_sha256(vec![hash_c.clone(), hash_d.clone()]),
                ]
                .concat(),
                [
                    hash_d.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
                [
                    hash_c.clone(),
                    merkleize_sha256(vec![hash_a.clone(), hash_b.clone()]),
                ]
                .concat(),
            ],
            token_leaves: vec![
                leaf_a.clone(),
                leaf_b.clone(),
                leaf_c.clone(),
                leaf_d.clone(),
            ],
        };

        let msg = ExecuteMsg::ProcessClaim {
            claim: exeute_claim.clone(),
            recipient: recipient.to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        println!("Error: {:?}", result);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.messages.len(), 4);
        assert_eq!(response.events.len(), 4);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "RewardsClaimed");
        assert_eq!(event.attributes.len(), 6);
        assert_eq!(event.attributes[0].key, "root");
        assert_eq!(
            event.attributes[0].value,
            format!("{:?}", distribution_root.root)
        );
        assert_eq!(event.attributes[1].key, "earner");
        assert_eq!(
            event.attributes[1].value,
            deps.api.addr_make("earner").to_string()
        );
        assert_eq!(event.attributes[2].key, "claimer");
        assert_eq!(
            event.attributes[2].value,
            deps.api.addr_make("claimer").to_string()
        );
        assert_eq!(event.attributes[3].key, "recipient");
        assert_eq!(
            event.attributes[3].value,
            deps.api.addr_make("recipient").to_string()
        );
        assert_eq!(event.attributes[4].key, "token");
        assert_eq!(event.attributes[4].value, "token_a");
        assert_eq!(event.attributes[5].key, "amount");
        assert_eq!(event.attributes[5].value, "100");

        // Test for unauthorized claimer
        let unauthorized_info = message_info(&Addr::unchecked("unauthorized_claimer"), &[]);
        let result = process_claim(
            deps.as_mut(),
            env.clone(),
            unauthorized_info,
            claim.clone(),
            recipient.clone(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::UnauthorizedClaimer {});
        }

        // Test for already claimed amount
        CUMULATIVE_CLAIMED
            .save(
                &mut deps.storage,
                (Addr::unchecked("earner"), "token_a".to_string()),
                &Uint128::new(100),
            )
            .unwrap();

        let result = process_claim(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            claim.clone(),
            recipient.clone(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::CumulativeEarningsTooLow {});
        }

        // Test for disabled root
        let disabled_root = DistributionRoot {
            root: Binary::from(earner_root.clone()),
            rewards_calculation_end_timestamp: 500,
            activated_at: env.block.time.seconds() - 100,
            disabled: true,
        };

        DISTRIBUTION_ROOTS
            .save(&mut deps.storage, 0, &disabled_root)
            .unwrap();

        let result = process_claim(deps.as_mut(), env.clone(), info.clone(), claim, recipient);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::RootDisabled {});
        }
    }

    #[test]
    fn test_disable_root() {
        let (
            mut deps,
            env,
            _owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let rewards_updater = deps.api.addr_make("rewards_updater");
        let rewards_updater_info = message_info(&rewards_updater, &[]);

        REWARDS_UPDATER
            .save(&mut deps.storage, &deps.api.addr_make("rewards_updater"))
            .unwrap();

        let valid_root = DistributionRoot {
            root: Binary::from(b"valid_root".to_vec()),
            rewards_calculation_end_timestamp: 500,
            activated_at: env.block.time.seconds() + 1000, // Future activation
            disabled: false,
        };

        DISTRIBUTION_ROOTS
            .save(&mut deps.storage, 0, &valid_root)
            .unwrap();
        DISTRIBUTION_ROOTS_COUNT
            .save(&mut deps.storage, &1u64)
            .unwrap();

        let msg = ExecuteMsg::DisableRoot { root_index: 0 };

        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg.clone(),
        );
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);

        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "DistributionRootDisabled");
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(event.attributes[0].key, "root_index");
        assert_eq!(event.attributes[0].value, "0");

        let stored_root = DISTRIBUTION_ROOTS.load(&deps.storage, 0).unwrap();
        assert!(stored_root.disabled);

        // Test disabling an already disabled root
        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg.clone(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::AlreadyDisabled {});
        }

        // Prepare an activated root
        let activated_root = DistributionRoot {
            root: Binary::from(b"activated_root".to_vec()),
            rewards_calculation_end_timestamp: 500,
            activated_at: env.block.time.seconds() - 1000, // Past activation
            disabled: false,
        };

        DISTRIBUTION_ROOTS
            .save(&mut deps.storage, 1, &activated_root)
            .unwrap();
        DISTRIBUTION_ROOTS_COUNT
            .save(&mut deps.storage, &2u64)
            .unwrap();

        let msg_activated = ExecuteMsg::DisableRoot { root_index: 1 };

        // Test disabling an activated root
        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg_activated,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::AlreadyActivated {});
        }

        // Test with an invalid root index
        let msg_invalid_index = ExecuteMsg::DisableRoot { root_index: 3 };

        let result = execute(
            deps.as_mut(),
            env.clone(),
            rewards_updater_info.clone(),
            msg_invalid_index,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::InvalidRootIndex {});
        }

        // Test unauthorized caller
        let unauthorized_info = message_info(&Addr::unchecked("not_rewards_updater"), &[]);
        let result = execute(deps.as_mut(), env, unauthorized_info, msg);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::NotRewardsUpdater {});
        }
    }

    #[test]
    fn test_pause() {
        let (
            mut deps,
            env,
            _owner_info,
            pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

        let pause_msg = ExecuteMsg::Pause {};
        let res = execute(deps.as_mut(), env.clone(), pauser_info.clone(), pause_msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action", "PAUSED")]);

        let paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_state, 1);
    }

    #[test]
    fn test_unpause() {
        let (
            mut deps,
            env,
            _owner_info,
            _pauser_info,
            unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

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
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

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
        let (
            mut deps,
            env,
            owner_info,
            _pauser_info,
            _unpauser_info,
            _strategy_manager_info,
            _delegation_manager_info,
            _rewards_updater_info,
        ) = instantiate_contract();

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
