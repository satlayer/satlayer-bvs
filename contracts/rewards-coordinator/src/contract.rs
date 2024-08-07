use crate::{
    error::ContractError,
    strategy_manager,
    msg::{InstantiateMsg, DistributionRoot, QueryMsg},
    state::{OWNER, REWARDS_UPDATER, CALCULATION_INTERVAL_SECONDS, REWARDS_FOR_ALL_SUBMITTER, IS_AVS_REWARDS_SUBMISSION_HASH, CLAIMER_FOR, DISTRIBUTION_ROOTS,
        MAX_REWARDS_DURATION, MAX_RETROACTIVE_LENGTH, MAX_FUTURE_LENGTH, GENESIS_REWARDS_TIMESTAMP, DELEGATION_MANAGER, STRATEGY_MANAGER, ACTIVATION_DELAY,
        GLOBAL_OPERATOR_COMMISSION_BIPS, SUBMISSION_NONCE, DISTRIBUTION_ROOTS_COUNT, CURR_REWARDS_CALCULATION_END_TIMESTAMP, CUMULATIVE_CLAIMED
    },
    utils::{RewardsSubmission, calculate_rewards_submission_hash, TokenTreeMerkleLeaf, calculate_token_leaf_hash,
        verify_inclusion_keccak, EarnerTreeMerkleLeaf, calculate_earner_leaf_hash, RewardsMerkleClaim, calculate_domain_separator
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
    let is_submitter = REWARDS_FOR_ALL_SUBMITTER.load(deps.storage, info.sender.clone())?;
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

