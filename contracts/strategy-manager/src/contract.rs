use crate::{
    error::ContractError,
    strategybase,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        StrategyManagerState, STRATEGY_MANAGER_STATE, STRATEGY_WHITELIST, STRATEGY_WHITELISTER, OWNER,
        STAKER_STRATEGY_SHARES, STAKER_STRATEGY_LIST, MAX_STAKER_STRATEGY_LIST_LENGTH, THIRD_PARTY_TRANSFERS_FORBIDDEN, NONCES
    },
    utils::{calculate_digest_hash, recover, DigestHashParams, DepositWithSignatureParams},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, SubMsg, StdError,
    Uint64, Binary
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
        ExecuteMsg::AddStrategiesToWhitelist { strategies, third_party_transfers_forbidden_values } => {
            add_strategies_to_whitelist(deps, info, strategies, third_party_transfers_forbidden_values)
        }
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => remove_strategies_from_whitelist(deps, info, strategies),
        ExecuteMsg::SetStrategyWhitelister { new_strategy_whitelister } => set_strategy_whitelister(deps, info, new_strategy_whitelister),
        ExecuteMsg::DepositIntoStrategy { strategy, token, amount } => deposit_into_strategy(deps, env, info, strategy, token, amount),
        ExecuteMsg::SetThirdPartyTransfersForbidden { strategy, value } => set_third_party_transfers_forbidden(deps, info, strategy, value),
        ExecuteMsg::DepositIntoStrategyWithSignature { strategy, token, amount, staker, expiry, signature } => {
            let params = DepositWithSignatureParams {
                strategy,
                token,
                amount,
                staker,
                expiry,
                signature,
            };
            deposit_into_strategy_with_signature(deps, env, info, params)
        },
        ExecuteMsg::RemoveShares { staker, strategy, shares } => remove_shares(deps, info, staker, strategy, shares),
        ExecuteMsg::WithdrawSharesAsTokens { recipient, strategy, shares, token } => withdraw_shares_as_tokens(deps, info, recipient, strategy, shares, token),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposits { staker } => to_json_binary(&get_deposits(deps, staker)?),
        QueryMsg::StakerStrategyListLength { staker } => to_json_binary(&staker_strategy_list_length(deps, staker)?),
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

fn only_delegation_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    if info.sender != state.delegation_manager {
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
    third_party_transfers_forbidden_values: Vec<bool>,
) -> Result<Response, ContractError> {
    // Ensure only the strategy whitelister can call this function
    only_strategy_whitelister(deps.as_ref(), &info)?;

    // Check if the length of strategies matches the length of third_party_transfers_forbidden_values
    if strategies.len() != third_party_transfers_forbidden_values.len() {
        return Err(ContractError::InvalidInput {});
    }

    // Initialize response
    let mut response = Response::new()
        .add_attribute("method", "add_strategies_to_whitelist");

    // Iterate over strategies and third_party_transfers_forbidden_values
    for (i, strategy) in strategies.iter().enumerate() {
        let forbidden_value = third_party_transfers_forbidden_values[i];

        // Check if the strategy is already whitelisted
        let is_whitelisted = STRATEGY_WHITELIST.may_load(deps.storage, strategy)?.unwrap_or(false);
        if !is_whitelisted {
            // Save strategy to whitelist
            STRATEGY_WHITELIST.save(deps.storage, strategy, &true)?;
            
            // Save third party transfers forbidden value
            THIRD_PARTY_TRANSFERS_FORBIDDEN.save(deps.storage, strategy, &forbidden_value)?;
            
            // Add event attributes
            response = response
                .add_attribute("strategy_added", strategy.to_string())
                .add_attribute("third_party_transfers_forbidden", forbidden_value.to_string());
        }
    }

    Ok(response)
}

fn remove_strategies_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    // Ensure only the strategy whitelister can call this function
    only_strategy_whitelister(deps.as_ref(), &info)?;

    // Initialize response
    let mut response = Response::new()
        .add_attribute("method", "remove_strategies_from_whitelist");

    // Iterate over strategies
    for strategy in strategies {
        // Check if the strategy is already whitelisted
        let is_whitelisted = STRATEGY_WHITELIST.may_load(deps.storage, &strategy)?.unwrap_or(false);
        if is_whitelisted {
            // Remove strategy from whitelist
            STRATEGY_WHITELIST.save(deps.storage, &strategy, &false)?;

            // Set third party transfers forbidden value to false
            THIRD_PARTY_TRANSFERS_FORBIDDEN.save(deps.storage, &strategy, &false)?;

            // Add event attributes
            response = response
                .add_attribute("strategy_removed", strategy.to_string())
                .add_attribute("third_party_transfers_forbidden", "false".to_string());
        }
    }

    Ok(response)
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

fn set_third_party_transfers_forbidden(
    deps: DepsMut,
    info: MessageInfo,
    strategy: Addr,
    value: bool,
) -> Result<Response, ContractError> {
    only_strategy_whitelister(deps.as_ref(), &info)?;

    THIRD_PARTY_TRANSFERS_FORBIDDEN.save(deps.storage, &strategy, &value)?;

    Ok(Response::new()
        .add_attribute("method", "set_third_party_transfers_forbidden")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("value", value.to_string()))
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

    let add_shares_response = add_shares(deps, info.clone(), info.sender.clone(), strategy.clone(), new_shares)?;

    Ok(deposit_response.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&add_shares_response)?,
        funds: vec![],
    }))))
}

fn deposit_into_strategy_with_signature(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: DepositWithSignatureParams,
) -> Result<Response, ContractError> {
    let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.may_load(deps.storage, &params.strategy)?.unwrap_or(false);
    if forbidden {
        return Err(ContractError::Unauthorized {});
    }

    let current_time: Uint64 = env.block.time.seconds().into();
    if params.expiry < current_time {
        return Err(ContractError::SignatureExpired {});
    }

    // Get the current nonce for the staker
    let nonce = NONCES.may_load(deps.storage, &params.staker)?.unwrap_or(0);

    let chain_id = env.block.chain_id.clone();

    let digest_params = DigestHashParams {
        staker: params.staker.clone(),
        strategy: params.strategy.clone(),
        token: params.token.clone(),
        amount: params.amount.u128(),
        nonce,
        expiry: params.expiry.u64(),
        chain_id,
        contract_addr: env.contract.address.clone(),
    };

    let struct_hash = calculate_digest_hash(&digest_params);

    let signature_bytes = hex::decode(&params.signature).map_err(|_| ContractError::InvalidSignature {})?;
    
    if !recover(&struct_hash, &signature_bytes, &params.staker)? {
        return Err(ContractError::InvalidSignature {});
    }

    // Increment the nonce for the staker
    NONCES.save(deps.storage, &params.staker, &(nonce + 1))?;

    deposit_into_strategy(deps, env, info, params.strategy, params.token, params.amount)
        .map(|mut res| {
            res.attributes.push(("method".to_string(), "deposit_into_strategy_with_signature".to_string()).into());
            res
    })
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
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_delegation_manager(deps.as_ref(), &info)?;

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

fn remove_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_delegation_manager(deps.as_ref(), &info)?;

    // Get the current shares for the staker and strategy
    let mut current_shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or_else(Uint128::zero);

    if shares > current_shares {
        return Err(ContractError::InvalidShares {});
    }

    // Subtract the shares
    current_shares = current_shares.checked_sub(shares).map_err(|_| ContractError::InvalidShares {})?;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &current_shares)?;

    // If the shares are zero, remove the strategy from the staker's list
    if current_shares.is_zero() {
        remove_strategy_from_staker_strategy_list(deps, staker.clone(), strategy.clone())?;
    }

    Ok(Response::new()
        .add_attribute("method", "remove_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string()))
}

fn remove_strategy_from_staker_strategy_list(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
) -> Result<(), ContractError> {
    let mut strategy_list = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    if let Some(pos) = strategy_list.iter().position(|x| *x == strategy) {
        strategy_list.swap_remove(pos);
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
        Ok(())
    } else {
        Err(ContractError::StrategyNotFound {})
    }
}

fn withdraw_shares_as_tokens(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Addr,
    strategy: Addr,
    shares: Uint128,
    token: Addr,
) -> Result<Response, ContractError> {
    // Ensure only the delegation manager can call this function
    only_delegation_manager(deps.as_ref(), &info)?;

    // Create the message to call the withdraw function on the strategy
    let withdraw_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::Withdraw {
            recipient: recipient.clone(),
            amount_shares: shares,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(withdraw_msg)
        .add_attribute("method", "withdraw_shares_as_tokens")
        .add_attribute("recipient", recipient.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("token", token.to_string()))
}

pub fn get_deposits(deps: Deps, staker: Addr) -> StdResult<(Vec<Addr>, Vec<Uint128>)> {
    // Load the staker's strategy list
    let strategies = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    // Initialize a vector to hold the shares for each strategy
    let mut shares = Vec::with_capacity(strategies.len());

    // Iterate over each strategy and fetch the corresponding shares
    for strategy in &strategies {
        let share = STAKER_STRATEGY_SHARES.may_load(deps.storage, (&staker, strategy))?
            .unwrap_or_else(Uint128::zero);
        shares.push(share);
    }

    // Return the strategies and their corresponding shares
    Ok((strategies, shares))
}

pub fn staker_strategy_list_length(deps: Deps, staker: Addr) -> StdResult<Uint64> {
    let strategies = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);
    Ok(Uint64::new(strategies.len() as u64))
}
