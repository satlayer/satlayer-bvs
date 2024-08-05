use crate::{
    error::ContractError,
    strategy_manager,
    msg::InstantiateMsg,
    state::{OWNER, REWARDS_UPDATER, CALCULATION_INTERVAL_SECONDS, REWARDS_FOR_ALL_SUBMITTER, IS_AVS_REWARDS_SUBMISSION_HASH, 
        MAX_REWARDS_DURATION, MAX_RETROACTIVE_LENGTH, MAX_FUTURE_LENGTH, GENESIS_REWARDS_TIMESTAMP, DELEGATION_MANAGER, STRATEGY_MANAGER, ACTIVATION_DELAY,
        GLOBAL_OPERATOR_COMMISSION_BIPS, SUBMISSION_NONCE
    },
    utils::{RewardsSubmission, calculate_rewards_submission_hash}
};
use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo, Response, Addr, Event, CosmosMsg, WasmMsg, to_json_binary
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use strategy_manager::QueryMsg as StrategyQueryMsg;


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

