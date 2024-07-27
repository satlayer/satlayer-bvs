use crate::{
    error::ContractError,
    strategybase,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{
        StrategyManagerState, STRATEGY_MANAGER_STATE, STRATEGY_WHITELIST, STRATEGY_WHITELISTER, OWNER,
        STAKER_STRATEGY_SHARES, STAKER_STRATEGY_LIST, MAX_STAKER_STRATEGY_LIST_LENGTH
    },
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, SubMsg, StdError,
};
use strategybase::ExecuteMsg as StrategyExecuteMsg;
use std::str::FromStr;

use cw20::Cw20ExecuteMsg;
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = StrategyManagerState {
        delegation_manager: msg.delegation_manager,
        slasher: msg.slasher,
    };

    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;
    STRATEGY_WHITELISTER.save(deps.storage, &msg.initial_strategy_whitelister)?;
    OWNER.save(deps.storage, &msg.initial_owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("delegation_manager", state.delegation_manager.to_string())
        .add_attribute("slasher", state.slasher.to_string())
        .add_attribute("strategy_whitelister", msg.initial_strategy_whitelister.to_string())
        .add_attribute("owner", msg.initial_owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddStrategiesToWhitelist { strategies } => add_strategies_to_whitelist(deps, info, strategies),
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => remove_strategies_from_whitelist(deps, info, strategies),
        ExecuteMsg::SetStrategyWhitelister { new_strategy_whitelister } => set_strategy_whitelister(deps, info, new_strategy_whitelister),
        ExecuteMsg::DepositIntoStrategy { strategy, token, amount } => deposit_into_strategy(deps, env, info, strategy, token, amount),
    }
}

fn only_strategy_whitelister(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let whitelister = STRATEGY_WHITELISTER.load(deps.storage)?;
    if info.sender != whitelister {
        return Err(ContractError::Unauthorized {});
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

fn only_strategies_whitelisted_for_deposit(deps: Deps, strategy: &Addr) -> Result<(), ContractError> {
    let whitelist = STRATEGY_WHITELIST.may_load(deps.storage, strategy)?.unwrap_or(false);
    if !whitelist {
        return Err(ContractError::StrategyNotWhitelisted {});
    }
    Ok(())
}

fn add_strategies_to_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    only_strategy_whitelister(deps.as_ref(), &info)?;

    for strategy in strategies {
        STRATEGY_WHITELIST.save(deps.storage, &strategy, &true)?;
    }

    Ok(Response::new()
        .add_attribute("method", "add_strategies_to_whitelist"))
}

fn remove_strategies_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    only_strategy_whitelister(deps.as_ref(), &info)?;

    for strategy in strategies {
        STRATEGY_WHITELIST.save(deps.storage, &strategy, &false)?;
    }

    Ok(Response::new()
        .add_attribute("method", "remove_strategies_from_whitelist"))
}

fn set_strategy_whitelister(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_whitelister: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    STRATEGY_WHITELISTER.save(deps.storage, &new_strategy_whitelister)?;

    Ok(Response::new()
        .add_attribute("method", "set_strategy_whitelister")
        .add_attribute("new_strategy_whitelister", new_strategy_whitelister.to_string()))
}

fn deposit_into_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy)?;

    let transfer_msg = create_transfer_msg(&info, &token, &strategy, amount)?;
    let deposit_msg = create_deposit_msg(&strategy, amount)?;

    let deposit_response = Response::new()
        .add_message(transfer_msg)
        .add_message(deposit_msg)
        .add_attribute("method", "deposit_into_strategy")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("amount", amount.to_string());

    let new_shares = query_new_shares_from_response(&deposit_response)?;

    let add_shares_response = add_shares(deps, info.sender.clone(), strategy.clone(), new_shares)?;

    // TODO:
    // delegation.increaseDelegatedShares(staker, strategy, shares);

    Ok(deposit_response.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&add_shares_response)?,
        funds: vec![],
    }))))
}

fn query_new_shares_from_response(response: &Response) -> StdResult<Uint128> {
    for attr in &response.attributes {
        if attr.key == "new_shares" {
            return Uint128::from_str(&attr.value).map_err(|_| StdError::generic_err("Failed to parse new_shares"));
        }
    }
    Err(StdError::generic_err("new_shares attribute not found"))
}

fn create_transfer_msg(
    info: &MessageInfo,
    token: &Addr,
    strategy: &Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: strategy.to_string(),
            amount,
        })?,
        funds: vec![],
    }))
}

fn create_deposit_msg(
    strategy: &Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::Deposit { amount })?,
        funds: vec![],
    }))
}

fn add_shares(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    if shares.is_zero() {
        return Err(ContractError::InvalidShares {});
    }

    let mut strategy_list = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    let current_shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or_else(Uint128::zero);

    if current_shares.is_zero() {
        if strategy_list.len() >= MAX_STAKER_STRATEGY_LIST_LENGTH {
            return Err(ContractError::MaxStrategyListLengthExceeded {});
        }
        strategy_list.push(strategy.clone());
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
    }

    let new_shares = current_shares + shares;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &new_shares)?;

    Ok(Response::new()
        .add_attribute("method", "add_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string()))
}
