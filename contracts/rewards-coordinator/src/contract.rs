use crate::{
    error::ContractError,
    strategy_manager,
    msg::{InstantiateMsg, DistributionRoot, QueryMsg},
    state::{OWNER, REWARDS_UPDATER, CALCULATION_INTERVAL_SECONDS, REWARDS_FOR_ALL_SUBMITTER, IS_AVS_REWARDS_SUBMISSION_HASH, CLAIMER_FOR, DISTRIBUTION_ROOTS,
        MAX_REWARDS_DURATION, MAX_RETROACTIVE_LENGTH, MAX_FUTURE_LENGTH, GENESIS_REWARDS_TIMESTAMP, DELEGATION_MANAGER, STRATEGY_MANAGER, ACTIVATION_DELAY,
        GLOBAL_OPERATOR_COMMISSION_BIPS, SUBMISSION_NONCE, DISTRIBUTION_ROOTS_COUNT, CURR_REWARDS_CALCULATION_END_TIMESTAMP, CUMULATIVE_CLAIMED
    },
    utils::{RewardsSubmission, calculate_rewards_submission_hash, TokenTreeMerkleLeaf, calculate_token_leaf_hash,
        verify_inclusion_sha256, EarnerTreeMerkleLeaf, calculate_earner_leaf_hash, RewardsMerkleClaim, calculate_domain_separator
    }
};
use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo, Response, Addr, Event, CosmosMsg, WasmMsg, to_json_binary, Binary, Uint64, Uint128,
    HexBinary
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use strategy_manager::QueryMsg as StrategyQueryMsg;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
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

    if msg.genesis_rewards_timestamp.u64() % msg.calculation_interval_seconds.u64() != 0 {
        return Err(ContractError::InvalidGenesisTimestamp {});
    }

    if msg.calculation_interval_seconds.u64() % SNAPSHOT_CADENCE != 0 {
        return Err(ContractError::InvalidCalculationInterval {});
    }

    let owner = msg.initial_owner.clone();

    OWNER.save(deps.storage, &owner)?;

    CALCULATION_INTERVAL_SECONDS.save(deps.storage, &msg.calculation_interval_seconds.u64())?;
    MAX_REWARDS_DURATION.save(deps.storage, &msg.max_rewards_duration.u64())?;
    MAX_RETROACTIVE_LENGTH.save(deps.storage, &msg.max_retroactive_length.u64())?;
    MAX_FUTURE_LENGTH.save(deps.storage, &msg.max_future_length.u64())?;
    GENESIS_REWARDS_TIMESTAMP.save(deps.storage, &msg.genesis_rewards_timestamp.u64())?;
    DELEGATION_MANAGER.save(deps.storage, &msg.delegation_manager)?;
    STRATEGY_MANAGER.save(deps.storage, &msg.strategy_manager)?;
    
    _set_rewards_updater(deps.branch(), msg.rewards_updater.clone())?;
    _set_activation_delay(deps.branch(), msg.activation_delay)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner.to_string())
        .add_attribute("rewards_updater", msg.rewards_updater.to_string())
        .add_attribute("activation_delay", msg.activation_delay.to_string()))
}

fn _only_rewards_updater(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let rewards_updater = REWARDS_UPDATER.load(deps.storage)?;

    if info.sender != rewards_updater {
        return Err(ContractError::NotRewardsUpdater {});
    }
    Ok(())
}

fn _only_rewards_for_all_submitter(deps: Deps, info: &MessageInfo) ->  Result<(), ContractError> {
    let is_submitter = REWARDS_FOR_ALL_SUBMITTER.may_load(deps.storage, info.sender.clone())?.unwrap_or(false);
    if !is_submitter {
        return Err(ContractError::ValidCreateRewardsForAllSubmission {});
    } 
    Ok(())
}

fn _only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn create_avs_rewards_submission(
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

        let rewards_submission_hash = calculate_rewards_submission_hash(&info.sender, nonce, &submission);

        _validate_rewards_submission(&deps.as_ref(), &submission, &env)?;

        IS_AVS_REWARDS_SUBMISSION_HASH.save(
            deps.storage,
            (info.sender.clone(), rewards_submission_hash.to_vec()),
            &true,
        )?;

        SUBMISSION_NONCE.save(deps.storage, info.sender.clone(), &(nonce + 1))?;

        let event = Event::new("AVSRewardsSubmissionCreated")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("nonce", nonce.to_string())
            .add_attribute("rewards_submission_hash", rewards_submission_hash.to_string())
            .add_attribute("token", submission.token.to_string())
            .add_attribute("amount", submission.amount.to_string());

        response = response.add_event(event);

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
    }

    Ok(response)
}

pub fn create_rewards_for_all_submission(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rewards_submissions: Vec<RewardsSubmission>,
) -> Result<Response, ContractError> {
    _only_rewards_for_all_submitter(deps.as_ref(), &info)?;

    let mut response = Response::new();
    for submission in rewards_submissions {
        let nonce = SUBMISSION_NONCE.may_load(deps.storage, info.sender.clone())?.unwrap_or_default();

        let rewards_submission_hash = calculate_rewards_submission_hash(&info.sender, nonce, &submission);

        _validate_rewards_submission(&deps.as_ref(), &submission, &env)?;

        IS_AVS_REWARDS_SUBMISSION_HASH.save(
            deps.storage,
            (info.sender.clone(), rewards_submission_hash.to_vec()),
            &true,
        )?;

        SUBMISSION_NONCE.save(deps.storage, info.sender.clone(), &(nonce + 1))?;

        let event = Event::new("RewardsSubmissionForAllCreated")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("nonce", nonce.to_string())
            .add_attribute("rewards_submission_hash", rewards_submission_hash.to_string())
            .add_attribute("token", submission.token.to_string())
            .add_attribute("amount", submission.amount.to_string());

        response = response.add_event(event);

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
    }

    Ok(response)
}

fn _validate_rewards_submission(
    deps: &Deps,
    submission: &RewardsSubmission,
    env: &Env
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
        || submission.start_timestamp.seconds() < genesis_rewards_timestamp {
        return Err(ContractError::StartTimeStampTooFarInPase {});
    }

    let max_future_length = MAX_FUTURE_LENGTH.load(deps.storage)?;
    if submission.start_timestamp.seconds() > env.block.time.seconds() + max_future_length {
        return Err(ContractError::StartTimeStampTooFarInFuture {});
    }

    let mut current_address = Addr::unchecked("");
    for strategy_multiplier in &submission.strategies_and_multipliers {
        let strategy = &strategy_multiplier.strategy;

        let is_strategy_whitelisted: bool = deps.querier.query_wasm_smart(
            strategy,
            &StrategyQueryMsg::IsStrategyWhitelisted {
                strategy: strategy.clone(),
            },
        )?;
        
        if !is_strategy_whitelisted {
            return Err(ContractError::InvaildStrategyConsidered {});
        }
        if  current_address >= *strategy {
            return Err(ContractError::StrategiesMuseBeHandleDuplicates {});
        }
        current_address = strategy.clone();
    }
    Ok(Response::new().add_attribute("action", "validate_rewards_submission"))
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

    _check_claim(env.clone(), &claim, &root)?;

    let earner = claim.earner_leaf.earner.clone();
    let mut claimer = CLAIMER_FOR.may_load(deps.storage, earner.clone())?.unwrap_or_else(|| earner.clone());

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

        if token_leaf.cumulative_earnings <= curr_cumulative_claimed.into() {
            return Err(ContractError::CumulativeEarningsTooLow {});
        }

        let claim_amount = token_leaf.cumulative_earnings.u128() - curr_cumulative_claimed;

        CUMULATIVE_CLAIMED.save(
            deps.storage,
            (earner.clone(), token.to_string()),
            &token_leaf.cumulative_earnings.u128(),
        )?;

        // Prepare a transfer message for the token claim
        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.clone().into(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount: claim_amount.into(),
            })?,
            funds: vec![],
        });

        // Add the transfer message to the response
        response = response.add_message(transfer_msg);

        // Record an event for the rewards claim
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
    rewards_calculation_end_timestamp: Uint64,
) -> Result<Response, ContractError> {
    _only_rewards_updater(deps.as_ref(), &info)?;

    let curr_rewards_calculation_end_timestamp = CURR_REWARDS_CALCULATION_END_TIMESTAMP
        .may_load(deps.storage)?
        .unwrap_or(Uint64::zero());

    if rewards_calculation_end_timestamp <= curr_rewards_calculation_end_timestamp {
        return Err(ContractError::InvalidTimestamp {});
    }
    if rewards_calculation_end_timestamp.u64() >= env.block.time.seconds() {
        return Err(ContractError::TimestampInFuture {});
    }

    let activation_delay = ACTIVATION_DELAY.load(deps.storage)?;

    let activated_at = env.block.time.plus_seconds(activation_delay.into()).seconds();

    let root_index = DISTRIBUTION_ROOTS_COUNT.may_load(deps.storage)?.unwrap_or(0);

    let new_root = DistributionRoot {
        root,
        activated_at: Uint64::new(activated_at),
        rewards_calculation_end_timestamp,
        disabled: false,
    };
    DISTRIBUTION_ROOTS.save(deps.storage, root_index, &new_root)?;

    CURR_REWARDS_CALCULATION_END_TIMESTAMP.save(deps.storage, &rewards_calculation_end_timestamp)?;

    DISTRIBUTION_ROOTS_COUNT.save(deps.storage, &(root_index + 1))?;

    let event = Event::new("DistributionRootSubmitted")
        .add_attribute("root_index", root_index.to_string())
        .add_attribute("root", format!("{:?}", new_root.root))
        .add_attribute("rewards_calculation_end_timestamp", new_root.rewards_calculation_end_timestamp.to_string())
        .add_attribute("activated_at", new_root.activated_at.to_string());

    Ok(Response::new().add_event(event))
}

pub fn disable_root(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    root_index: u64,
) -> Result<Response, ContractError> {
    _only_rewards_updater(deps.as_ref(), &info)?;

    let roots_length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;
    if root_index >= roots_length {
        return Err(ContractError::InvalidRootIndex {});
    }

    let mut root: DistributionRoot = DISTRIBUTION_ROOTS.load(deps.storage, root_index)
        .map_err(|_| ContractError::RootNotExist {})?;

    if root.disabled {
        return Err(ContractError::AlreadyDisabled {});
    }
    if  env.block.time.seconds() >= root.activated_at.into() {
        return Err(ContractError::AlreadyActivated {});
    }

    root.disabled = true;
    DISTRIBUTION_ROOTS.save(deps.storage, root_index, &root)?;

    let event = Event::new("DistributionRootDisabled")
        .add_attribute("root_index", root_index.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_claimer_for(
    deps: DepsMut,
    info: MessageInfo,
    claimer: Addr,
) -> Result<Response, ContractError> {
    let earner = info.sender;  
    let prev_claimer = CLAIMER_FOR.may_load(deps.storage, earner.clone())?.unwrap_or(Addr::unchecked(""));

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

    _check_claim(env, &claim, &root)?;

    Ok(true)
}


fn _check_claim(env: Env, claim: &RewardsMerkleClaim, root: &DistributionRoot) -> Result<(), ContractError> {
    if root.disabled {
        return Err(ContractError::RootDisabled {});
    }

    if env.block.time.seconds() < root.activated_at.into() {
        return Err(ContractError::RootNotActivatedYet {});
    }

    if claim.token_indices.len() != claim.token_tree_proofs.len() {
        return Err(ContractError::TokenIndicesAndProofsMismatch {});
    }

    if claim.token_tree_proofs.len() != claim.token_leaves.len() {
        return Err(ContractError::TokenProofsAndLeavesMismatch {});
    }

    _verify_earner_claim_proof(
        root.root.clone(),
        claim.earner_index,
        &claim.earner_tree_proof,
        &claim.earner_leaf,
    )?;

    for i in 0..claim.token_indices.len() {
        _verify_token_claim_proof(
            claim.earner_leaf.earner_token_root.clone(),
            claim.token_indices[i],
            &claim.token_tree_proofs[i],
            &claim.token_leaves[i],
        )?;
    }

    Ok(())
}

fn _verify_token_claim_proof(
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

fn _verify_earner_claim_proof(
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

pub fn set_activation_delay(
    deps: DepsMut,
    info: MessageInfo,
    new_activation_delay: u32,    
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    let res = _set_activation_delay(deps, new_activation_delay)?;
    Ok(res)
}

fn _set_activation_delay(
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

pub fn set_rewards_updater(
    deps: DepsMut,
    info: MessageInfo,
    new_updater: Addr,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    let res = _set_rewards_updater(deps, new_updater)?;
    Ok(res)
}

fn _set_rewards_updater(
    deps: DepsMut,
    new_updater: Addr,
) -> Result<Response, ContractError> {
    REWARDS_UPDATER.save(deps.storage, &new_updater)?;

    let event = Event::new("SetRewardsUpdater")
        .add_attribute("method", "set_rewards_updater")
        .add_attribute("new_updater", new_updater.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_rewards_for_all_submitter(
    deps: DepsMut,
    info: MessageInfo,
    submitter: Addr,
    new_value: bool,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    let prev_value = REWARDS_FOR_ALL_SUBMITTER.may_load(deps.storage, submitter.clone())?.unwrap_or(false);
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
    _only_owner(deps.as_ref(), &info)?;

    let res = _set_global_operator_commission(deps, new_commission_bips)?;
    Ok(res)
}

fn _set_global_operator_commission(
    deps: DepsMut,
    new_commission_bips: u16,
) -> Result<Response, ContractError> {
    let current_commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS.may_load(deps.storage)?.unwrap_or(0);

    GLOBAL_OPERATOR_COMMISSION_BIPS.save(deps.storage, &new_commission_bips)?;

    let event = Event::new("GlobalCommissionBipsSet")
        .add_attribute("old_commission_bips", current_commission_bips.to_string())
        .add_attribute("new_commission_bips", new_commission_bips.to_string());

    Ok(Response::new()
        .add_event(event))
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

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CalculateEarnerLeafHash { earner, earner_token_root } => {
            query_calculate_earner_leaf_hash(deps, earner, earner_token_root)
        }
        QueryMsg::CalculateTokenLeafHash { token, cumulative_earnings } => {
            query_calculate_token_leaf_hash(deps, token, cumulative_earnings)
        }
        QueryMsg::QueryOperatorCommissionBips { operator, avs } => {
            query_operator_commission_bips(deps, operator, avs)
        }
        QueryMsg::GetDistributionRootsLength { } => {
            query_distribution_roots_length(deps)
        }
        QueryMsg::GetDistributionRootAtIndex { index } => {
            query_get_distribution_root_at_index(deps, index)
        }
        QueryMsg::GetCurrentDistributionRoot {} => { 
            query_get_current_distribution_root(deps)
        }
        QueryMsg::GetCurrentClaimableDistributionRoot {} => {
            query_get_current_claimable_distribution_root(deps, env)
        }
        QueryMsg::GetRootIndexFromHash { root_hash } => {
            query_get_root_index_from_hash(deps, root_hash)
        }
        QueryMsg::CalculateDomainSeparator { chain_id, contract_addr } => {
            query_calculate_domain_separator(deps, chain_id, contract_addr)
        }
    }
}

fn query_calculate_earner_leaf_hash(
    _deps: Deps,
    earner: String,
    earner_token_root: String,
) -> Result<Binary, ContractError> {
    let earner_addr: Addr = Addr::unchecked(earner);
    let earner_token_root_binary = Binary::from_base64(&earner_token_root)?;

    let leaf = EarnerTreeMerkleLeaf {
        earner: earner_addr,
        earner_token_root: earner_token_root_binary,
    };

    let hash = calculate_earner_leaf_hash(&leaf);

    Ok(to_json_binary(&hash)?)
}

fn query_calculate_token_leaf_hash(
    _deps: Deps,
    token: String,
    cumulative_earnings: String,
) -> Result<Binary, ContractError> {
    let token_addr: Addr = Addr::unchecked(token);

    let cumulative_earnings_amount: Uint128 = cumulative_earnings
        .parse::<u128>()
        .map(Uint128::from)
        .map_err(|_| ContractError::InvalidCumulativeEarnings {})?;

    let leaf = TokenTreeMerkleLeaf {
        token: token_addr,
        cumulative_earnings: cumulative_earnings_amount,
    };

    let hash = calculate_token_leaf_hash(&leaf);

    Ok(to_json_binary(&hash)?)
}

fn query_operator_commission_bips(
    deps: Deps,
    _operator: String, 
    _avs: String, 
) -> Result<Binary, ContractError> {
    let commission_bips = GLOBAL_OPERATOR_COMMISSION_BIPS.load(deps.storage)?;

    Ok(to_json_binary(&commission_bips)?)
}

fn query_distribution_roots_length(deps: Deps) -> Result<Binary, ContractError> {
    let roots_length: u64 = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    let roots_length_as_uint64 = Uint64::from(roots_length);

    Ok(to_json_binary(&roots_length_as_uint64)?)
}

fn query_get_distribution_root_at_index(
    deps: Deps,
    index: String,
) -> Result<Binary, ContractError> {
    let index: u64 = index.parse().map_err(|_| ContractError::InvalidIndexFormat {})?;

    let root: DistributionRoot = DISTRIBUTION_ROOTS
        .may_load(deps.storage, index)?
        .ok_or(ContractError::RootNotExist {})?;

    Ok(to_json_binary(&root)?)
}

fn query_get_current_distribution_root(deps: Deps) -> Result<Binary, ContractError> {
    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if !root.disabled {
                return Ok(to_json_binary(&root)?);
            }
        }
    }

    Err(ContractError::NoActiveRootFound {})
}

fn query_get_current_claimable_distribution_root(deps: Deps, env: Env) -> Result<Binary, ContractError> {
    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if !root.disabled && env.block.time.seconds() >= root.activated_at.u64() {
                return Ok(to_json_binary(&root)?);
            }
        }
    }

    Err(ContractError::NoClaimableRootFound {})
}

fn query_get_root_index_from_hash(deps: Deps, root_hash: String) -> Result<Binary, ContractError> {
    let root_hash_bytes = HexBinary::from_hex(&root_hash)
        .map_err(|_| ContractError::InvalidHexFormat {})?;

    let length = DISTRIBUTION_ROOTS_COUNT.load(deps.storage)?;

    for i in (0..length).rev() {
        if let Some(root) = DISTRIBUTION_ROOTS.may_load(deps.storage, i)? {
            if root.root.as_slice() == root_hash_bytes.as_slice() {
                return Ok(to_json_binary(&(i as u32))?);
            }
        }
    }

    Err(ContractError::RootNotFound {})
}

fn query_calculate_domain_separator(
    _deps: Deps,
    chain_id: String,
    contract_addr: String,
) -> Result<Binary, ContractError> {
    let contract_address = Addr::unchecked(contract_addr);
    let domain_separator = calculate_domain_separator(&chain_id, &contract_address);

    Ok(to_json_binary(&domain_separator)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{from_json, Addr, SystemResult, ContractResult, WasmQuery, SystemError, Timestamp};
    use crate::utils::StrategyAndMultiplier;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            calculation_interval_seconds: Uint64::new(86_400), // 1 day
            max_rewards_duration: Uint64::new(30 * 86_400),   // 30 days
            max_retroactive_length: Uint64::new(5 * 86_400),  // 5 days
            max_future_length: Uint64::new(10 * 86_400),      // 10 days
            genesis_rewards_timestamp: Uint64::new(env.block.time.seconds() / 86_400 * 86_400), 
            delegation_manager: Addr::unchecked("delegation_manager"),
            strategy_manager: Addr::unchecked("strategy_manager"),
            rewards_updater: Addr::unchecked("rewards_updater"),
            activation_delay: 60,  // 1 minute
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, "owner");
        assert_eq!(res.attributes[2].key, "rewards_updater");
        assert_eq!(res.attributes[2].value, "rewards_updater");

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, Addr::unchecked("owner"));

        let stored_calculation_interval = CALCULATION_INTERVAL_SECONDS.load(&deps.storage).unwrap();
        assert_eq!(stored_calculation_interval, 86_400);

        let stored_max_rewards_duration = MAX_REWARDS_DURATION.load(&deps.storage).unwrap();
        assert_eq!(stored_max_rewards_duration, 30 * 86_400);

        let stored_max_retroactive_length = MAX_RETROACTIVE_LENGTH.load(&deps.storage).unwrap();
        assert_eq!(stored_max_retroactive_length, 5 * 86_400);

        let stored_max_future_length = MAX_FUTURE_LENGTH.load(&deps.storage).unwrap();
        assert_eq!(stored_max_future_length, 10 * 86_400);

        let stored_genesis_rewards_timestamp = GENESIS_REWARDS_TIMESTAMP.load(&deps.storage).unwrap();
        assert_eq!(stored_genesis_rewards_timestamp, msg.genesis_rewards_timestamp.u64());

        let stored_delegation_manager = DELEGATION_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(stored_delegation_manager, Addr::unchecked("delegation_manager"));

        let stored_strategy_manager = STRATEGY_MANAGER.load(&deps.storage).unwrap();
        assert_eq!(stored_strategy_manager, Addr::unchecked("strategy_manager"));
        
        let stored_activation_delay = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay, 60);

        let stored_rewards_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_rewards_updater, Addr::unchecked("rewards_updater"));
    }

    #[test]
    fn test_only_rewards_updater_success() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = Addr::unchecked("rewards_updater");
        REWARDS_UPDATER.save(&mut deps.storage, &rewards_updater_addr).unwrap();

        let info = message_info(&Addr::unchecked("rewards_updater"), &[]);
        let result = _only_rewards_updater(deps.as_ref(), &info);

        assert!(result.is_ok());
    }

    #[test]
    fn test_only_rewards_updater_failure() {
        let mut deps = mock_dependencies();

        let rewards_updater_addr = Addr::unchecked("rewards_updater");
        REWARDS_UPDATER.save(&mut deps.storage, &rewards_updater_addr).unwrap();

        let info = message_info(&Addr::unchecked("not_rewards_updater"), &[]);
        let result = _only_rewards_updater(deps.as_ref(), &info);

        assert_eq!(result, Err(ContractError::NotRewardsUpdater {}));
    }

    #[test]
    fn test_only_rewards_for_all_submitter() {
        let mut deps = mock_dependencies();
    
        let valid_submitter = Addr::unchecked("valid_submitter");
        REWARDS_FOR_ALL_SUBMITTER.save(&mut deps.storage, valid_submitter.clone(), &true).unwrap();
    
        let info = message_info(&Addr::unchecked("valid_submitter"), &[]);
        let result = _only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert!(result.is_ok());
    
        let invalid_submitter = Addr::unchecked("invalid_submitter");
        REWARDS_FOR_ALL_SUBMITTER.save(&mut deps.storage, invalid_submitter.clone(), &false).unwrap();
    
        let info = message_info(&Addr::unchecked("invalid_submitter"), &[]);
        let result = _only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert_eq!(result, Err(ContractError::ValidCreateRewardsForAllSubmission {}));
    
        let info = message_info(&Addr::unchecked("unset_submitter"), &[]);
        let result = _only_rewards_for_all_submitter(deps.as_ref(), &info);
        assert_eq!(result, Err(ContractError::ValidCreateRewardsForAllSubmission {}));
    }    

    #[test]
    fn test_only_owner() {
    let mut deps = mock_dependencies();

    let owner_addr = Addr::unchecked("owner");
    OWNER.save(&mut deps.storage, &owner_addr).unwrap();

    let info = message_info(&Addr::unchecked("owner"), &[]);
    let result = _only_owner(deps.as_ref(), &info);
    assert!(result.is_ok());

    let info = message_info(&Addr::unchecked("not_owner"), &[]);
    let result = _only_owner(deps.as_ref(), &info);
    assert_eq!(result, Err(ContractError::Unauthorized {}));
    }

    #[test]
    fn test_validate_rewards_submission() {
        let mut deps = mock_dependencies();
        let calc_interval = 86_400; // 1 day
    
        CALCULATION_INTERVAL_SECONDS.save(&mut deps.storage, &calc_interval).unwrap();
        MAX_REWARDS_DURATION.save(&mut deps.storage, &(30 * calc_interval)).unwrap(); // 30 days
        MAX_RETROACTIVE_LENGTH.save(&mut deps.storage, &(5 * calc_interval)).unwrap(); // 5 days
        MAX_FUTURE_LENGTH.save(&mut deps.storage, &(10 * calc_interval)).unwrap(); // 10 days
    
        let block_time = mock_env().block.time.seconds();
        let genesis_time = block_time - (2 * calc_interval);
        GENESIS_REWARDS_TIMESTAMP.save(&mut deps.storage, &genesis_time).unwrap();
    
        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);
    
        let valid_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy_1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };
    
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if contract_addr == "strategy_1" => {
                let msg: StrategyQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyQueryMsg::IsStrategyWhitelisted { .. } => {
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                    },
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
    
        let result = _validate_rewards_submission(&deps.as_ref(), &valid_submission, &mock_env());
        assert!(result.is_ok());
    
        let no_strategy_submission = RewardsSubmission {
            strategies_and_multipliers: vec![],
            amount: Uint128::new(100),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };
    
        let result = _validate_rewards_submission(&deps.as_ref(), &no_strategy_submission, &mock_env());
        assert!(matches!(result, Err(ContractError::NoStrategiesSet {})));
    
        let zero_amount_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy_1"),
                multiplier: 1,
            }],
            amount: Uint128::zero(),
            duration: calc_interval,
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };
    
        let result = _validate_rewards_submission(&deps.as_ref(), &zero_amount_submission, &mock_env());
        assert!(matches!(result, Err(ContractError::AmountCannotBeZero {})));
    
        let exceeds_duration_submission = RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy_1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: 30 * calc_interval + 1, 
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        };
    
        let result = _validate_rewards_submission(&deps.as_ref(), &exceeds_duration_submission, &mock_env());
        assert!(matches!(result, Err(ContractError::ExceedsMaxRewardsDuration {})));
    }    

    #[test]
    fn test_create_avs_rewards_submission() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let calc_interval = 86_400; // 1 day
        CALCULATION_INTERVAL_SECONDS.save(&mut deps.storage, &calc_interval).unwrap(); 
        MAX_REWARDS_DURATION.save(&mut deps.storage, &(30 * calc_interval)).unwrap(); // 30 days
        MAX_RETROACTIVE_LENGTH.save(&mut deps.storage, &(5 * calc_interval)).unwrap(); // 5 days
        MAX_FUTURE_LENGTH.save(&mut deps.storage, &(10 * calc_interval)).unwrap(); // 10 days
        OWNER.save(&mut deps.storage, &Addr::unchecked("creator")).unwrap();
        REWARDS_UPDATER.save(&mut deps.storage, &Addr::unchecked("creator")).unwrap();

        let block_time = mock_env().block.time.seconds();
        let genesis_time = block_time - (2 * calc_interval); 
        GENESIS_REWARDS_TIMESTAMP.save(&mut deps.storage, &genesis_time).unwrap();

        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);

        let submission = vec![RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy_1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval, // 1 day
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        }];
    
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if contract_addr == "strategy_1" => {
                let msg: StrategyQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyQueryMsg::IsStrategyWhitelisted { .. } => {
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap())) 
                    },
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

        let result = create_avs_rewards_submission(deps.as_mut(), env, info, submission);
    
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.events.len(), 1);
    
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "AVSRewardsSubmissionCreated");
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "sender");
        assert_eq!(event.attributes[0].value, "creator");
        assert_eq!(event.attributes[1].key, "nonce");
        assert_eq!(event.attributes[1].value, "0");
        assert_eq!(event.attributes[2].key, "rewards_submission_hash");
        assert_eq!(event.attributes[2].value, "D+8YcM6Ak/8lhnToScMD6sXz0OP1Xi2Z076xuazagXs=");
        assert_eq!(event.attributes[3].key, "token");
        assert_eq!(event.attributes[3].value, "token");
        assert_eq!(event.attributes[4].key, "amount");
        assert_eq!(event.attributes[4].value, "100");
    }

    #[test]
    fn test_create_rewards_for_all_submission() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("valid_submitter"), &[]);
    
        let calc_interval = 86_400; // 1 day
        CALCULATION_INTERVAL_SECONDS.save(&mut deps.storage, &calc_interval).unwrap();
        MAX_REWARDS_DURATION.save(&mut deps.storage, &(30 * calc_interval)).unwrap(); // 30 days
        MAX_RETROACTIVE_LENGTH.save(&mut deps.storage, &(5 * calc_interval)).unwrap(); // 5 days
        MAX_FUTURE_LENGTH.save(&mut deps.storage, &(10 * calc_interval)).unwrap(); // 10 days
        OWNER.save(&mut deps.storage, &Addr::unchecked("creator")).unwrap();
        REWARDS_FOR_ALL_SUBMITTER.save(&mut deps.storage, Addr::unchecked("valid_submitter"), &true).unwrap();
    
        let block_time = mock_env().block.time.seconds();
        let genesis_time = block_time - (2 * calc_interval);
        GENESIS_REWARDS_TIMESTAMP
            .save(&mut deps.storage, &genesis_time)
            .unwrap();
    
        let aligned_start_time = block_time - (block_time % calc_interval);
        let aligned_start_timestamp = Timestamp::from_seconds(aligned_start_time);
    
        let submission = vec![RewardsSubmission {
            strategies_and_multipliers: vec![StrategyAndMultiplier {
                strategy: Addr::unchecked("strategy_1"),
                multiplier: 1,
            }],
            amount: Uint128::new(100),
            duration: calc_interval, // 1 day
            start_timestamp: aligned_start_timestamp,
            token: Addr::unchecked("token"),
        }];
    
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if contract_addr == "strategy_1" => {
                let msg: StrategyQueryMsg = from_json(msg).unwrap();
                match msg {
                    StrategyQueryMsg::IsStrategyWhitelisted { .. } => {
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
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
    
        let result = create_rewards_for_all_submission(deps.as_mut(), env, info, submission);
    
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.events.len(), 1);
    
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "RewardsSubmissionForAllCreated");
        assert_eq!(event.attributes.len(), 5);
        assert_eq!(event.attributes[0].key, "sender");
        assert_eq!(event.attributes[0].value, "valid_submitter");
        assert_eq!(event.attributes[1].key, "nonce");
        assert_eq!(event.attributes[1].value, "0");
        assert_eq!(event.attributes[2].key, "rewards_submission_hash");
        assert_eq!(event.attributes[2].value, "vyIVPmInZg5TBPxQlVNL3GBbMgJgauDMSZXhaaUuCyE=");
        assert_eq!(event.attributes[3].key, "token");
        assert_eq!(event.attributes[3].value, "token");
        assert_eq!(event.attributes[4].key, "amount");
        assert_eq!(event.attributes[4].value, "100");
    }

    #[test]
    fn test_transfer_ownership() {
        let mut deps = mock_dependencies();
        let info = message_info(&Addr::unchecked("current_owner"), &[]);

        OWNER.save(&mut deps.storage, &info.sender).unwrap();

        let new_owner = Addr::unchecked("new_owner");

        let result = transfer_ownership(deps.as_mut(), info, new_owner.clone());

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
        let mut deps = mock_dependencies();
    
        let initial_activation_delay: u32 = 60; // 1 minute
        ACTIVATION_DELAY.save(&mut deps.storage, &initial_activation_delay).unwrap();
    
        let new_activation_delay: u32 = 120; // 2 minutes
    
        let result = _set_activation_delay(deps.as_mut(), new_activation_delay);
    
        assert!(result.is_ok());
    
        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);
    
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "ActivationDelaySet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_activation_delay");
        assert_eq!(event.attributes[0].value, initial_activation_delay.to_string());
        assert_eq!(event.attributes[1].key, "new_activation_delay");
        assert_eq!(event.attributes[1].value, new_activation_delay.to_string());
    
        let stored_activation_delay = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay, new_activation_delay);
    }

    #[test]
    fn test_set_activation_delay() {
        let mut deps = mock_dependencies();
    
        let owner_addr = Addr::unchecked("owner");
        OWNER.save(&mut deps.storage, &owner_addr).unwrap();
    
        let initial_activation_delay: u32 = 60; // 1 minute
        ACTIVATION_DELAY.save(&mut deps.storage, &initial_activation_delay).unwrap();
    
        let new_activation_delay: u32 = 120; // 2 minutes
    
        let info = message_info(&owner_addr, &[]);
    
        let result = set_activation_delay(deps.as_mut(), info.clone(), new_activation_delay);
    
        assert!(result.is_ok());
    
        let response = result.unwrap();
        assert_eq!(response.events.len(), 1);
    
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, "ActivationDelaySet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_activation_delay");
        assert_eq!(event.attributes[0].value, initial_activation_delay.to_string());
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
    
        let stored_activation_delay_after_unauthorized_attempt = ACTIVATION_DELAY.load(&deps.storage).unwrap();
        assert_eq!(stored_activation_delay_after_unauthorized_attempt, new_activation_delay);
    }

    #[test]
    fn test_set_rewards_updater_internal() {
        let mut deps = mock_dependencies();
    
        let initial_updater = Addr::unchecked("initial_updater");
        REWARDS_UPDATER.save(&mut deps.storage, &initial_updater).unwrap();
    
        let new_updater = Addr::unchecked("new_updater");
    
        let result = _set_rewards_updater(deps.as_mut(), new_updater.clone());
    
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
        let mut deps = mock_dependencies();
    
        let owner = Addr::unchecked("owner");
        OWNER.save(&mut deps.storage, &owner).unwrap();
    
        let initial_updater = Addr::unchecked("initial_updater");
        REWARDS_UPDATER.save(&mut deps.storage, &initial_updater).unwrap();
    
        let new_updater = Addr::unchecked("new_updater");
    
        let info = message_info(&Addr::unchecked("owner"), &[]);
        let result = set_rewards_updater(deps.as_mut(), info, new_updater.clone());
    
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
        let result = set_rewards_updater(deps.as_mut(), unauthorized_info, Addr::unchecked("another_updater"));
    
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err, ContractError::Unauthorized {});
        }
    
        let stored_updater = REWARDS_UPDATER.load(&deps.storage).unwrap();
        assert_eq!(stored_updater, new_updater);
    }

}

