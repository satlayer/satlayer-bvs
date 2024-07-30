use crate::{
    error::ContractError,
    strategy_base,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        StrategyManagerState, STRATEGY_MANAGER_STATE, STRATEGY_WHITELIST, STRATEGY_WHITELISTER, OWNER,
        STAKER_STRATEGY_SHARES, STAKER_STRATEGY_LIST, MAX_STAKER_STRATEGY_LIST_LENGTH, THIRD_PARTY_TRANSFERS_FORBIDDEN, NONCES
    },
    utils::{calculate_digest_hash, recover, DigestHashParams, DepositWithSignatureParams},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, SubMsg,
    Uint64, Binary
};
use strategy_base::ExecuteMsg as StrategyExecuteMsg;
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
        ExecuteMsg::DepositIntoStrategy { strategy, token, amount } => {
            let staker = info.sender.clone();
            let response = deposit_into_strategy(deps, env, info, staker, strategy.clone(), token.clone(), amount)?;
            Ok(response)
        },
        ExecuteMsg::SetThirdPartyTransfersForbidden { strategy, value } => set_third_party_transfers_forbidden(deps, info, strategy, value),
        ExecuteMsg::DepositIntoStrategyWithSignature { strategy, token, amount, staker, public_key, expiry, signature } => {
            let params = DepositWithSignatureParams {
                strategy,
                token,
                amount,
                staker,
                public_key,
                expiry,
                signature,
            };
            deposit_into_strategy_with_signature(deps, env, info, params)
        },
        ExecuteMsg::RemoveShares { staker, strategy, shares } => remove_shares(deps, info, staker, strategy, shares),
        ExecuteMsg::WithdrawSharesAsTokens { recipient, strategy, shares, token } => withdraw_shares_as_tokens(deps, info, recipient, strategy, shares, token),
    }
}

fn only_strategy_whitelister(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let whitelister: Addr = STRATEGY_WHITELISTER.load(deps.storage)?;
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

pub fn deposit_into_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let response = _deposit_into_strategy(deps, env, info, staker, strategy, token, amount)?;
    Ok(response)
}

fn _deposit_into_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy)?;

    let transfer_msg = create_transfer_msg(&info, &token, &strategy, amount)?;
    let deposit_msg = create_deposit_msg(&strategy, amount)?;

    let new_shares = Uint128::new(50); // TODO: Replace this with actual logic to get new shares

    let deposit_response = Response::new()
        .add_message(transfer_msg)
        .add_message(deposit_msg)
        .add_attribute("method", "deposit_into_strategy")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("amount", amount.to_string())
        .add_attribute("new_shares", new_shares.to_string());

    let add_shares_response = add_shares(deps, info.clone(), staker.clone(), token.clone(), strategy.clone(), new_shares)?;

    Ok(deposit_response.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&add_shares_response)?,
        funds: vec![],
    }))))
}

fn _query_new_shares_from_response(response: &Response) -> Result<Uint128, ContractError> {
    for attr in &response.attributes {
        if attr.key == "new_shares" {
            return Uint128::from_str(&attr.value).map_err(|_| ContractError::InvalidShares {});
        }
    }
    Err(ContractError::InvalidShares {})
}

pub fn deposit_into_strategy_with_signature(
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
        public_key: params.public_key.clone(),
        strategy: params.strategy.clone(),
        token: params.token.clone(),
        amount: params.amount.u128(),
        nonce,
        expiry: params.expiry.u64(),
        chain_id,
        contract_addr: env.contract.address.clone(),
    };

    let struct_hash = calculate_digest_hash(&digest_params);

    if !recover(&struct_hash, &params.signature, &params.public_key)? {
        return Err(ContractError::InvalidSignature {});
    }

    // Increment the nonce for the staker
    NONCES.save(deps.storage, &params.staker, &(nonce + 1))?;

    // Call the original deposit_into_strategy function and capture its response
    let deposit_res = _deposit_into_strategy(deps, env.clone(), info, params.strategy.clone(), params.strategy.clone(), params.token.clone(), params.amount)?;

    // Create a new Response with the required method attribute and append original messages and attributes
    let mut res = Response::new()
        .add_attribute("method", "deposit_into_strategy_with_signature")
        .add_attribute("strategy", params.strategy.to_string())
        .add_attribute("amount", params.amount.to_string())
        .add_attribute("new_shares", "50"); // Adjust this to your actual logic

    // Append original messages
    res.messages = deposit_res.messages;

    // Return the final response
    Ok(res)
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
    token: Addr,
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
        .add_attribute("token", token.to_string())
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposits { staker } => to_json_binary(&get_deposits(deps, staker)?),
        QueryMsg::StakerStrategyListLength { staker } => to_json_binary(&staker_strategy_list_length(deps, staker)?),
        QueryMsg::GetStakerStrategyShares { staker, strategy } => to_json_binary(&get_staker_strategy_shares(deps, staker, strategy)?),
    }
}

fn get_staker_strategy_shares(deps: Deps, staker: Addr, strategy: Addr) -> StdResult<Uint128> {
    let shares = STAKER_STRATEGY_SHARES.load(deps.storage, (&staker, &strategy))?;
    Ok(shares)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{Addr, from_json};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use sha2::{Sha256, Digest};
    use ripemd::Ripemd160;
    use bech32::{self, ToBase32, Variant};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "delegation_manager");
        assert_eq!(res.attributes[1].value, "delegation_manager");
        assert_eq!(res.attributes[2].key, "slasher");
        assert_eq!(res.attributes[2].value, "slasher");
        assert_eq!(res.attributes[3].key, "strategy_whitelister");
        assert_eq!(res.attributes[3].value, "whitelister");
        assert_eq!(res.attributes[4].key, "owner");
        assert_eq!(res.attributes[4].value, "owner");

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, Addr::unchecked("owner"));

        let strategy_manager_state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(strategy_manager_state.delegation_manager, Addr::unchecked("delegation_manager"));
        assert_eq!(strategy_manager_state.slasher, Addr::unchecked("slasher"));

        let strategy_whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        assert_eq!(strategy_whitelister, Addr::unchecked("whitelister"));
    }

    #[test]
    fn test_only_strategy_whitelister() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        // Instantiate the contract with the whitelister
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        // Test with the correct whitelister
        let result = only_strategy_whitelister(deps.as_ref(), &info_whitelister);
        assert!(result.is_ok());

        // Test with an unauthorized user
        let result = only_strategy_whitelister(deps.as_ref(), &info_unauthorized);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_only_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_owner = message_info(&Addr::unchecked("owner"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
        

        // Instantiate the contract with the owner
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        // Test with the correct owner
        let result = only_owner(deps.as_ref(), &info_owner);
        assert!(result.is_ok());

        // Test with an unauthorized user
        let result = only_owner(deps.as_ref(), &info_unauthorized);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }
    #[test]
    fn test_only_delegation_manager() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        // Instantiate the contract with the delegation manager
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        // Test with the correct delegation manager
        let result = only_delegation_manager(deps.as_ref(), &info_delegation_manager);
        assert!(result.is_ok());

        // Test with an unauthorized user
        let result = only_delegation_manager(deps.as_ref(), &info_unauthorized);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_only_strategies_whitelisted_for_deposit() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        // Add a strategy to the whitelist
        let strategy = Addr::unchecked("strategy");
        STRATEGY_WHITELIST.save(&mut deps.storage, &strategy, &true).unwrap();

        // Test with a whitelisted strategy
        let result = only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy);
        assert!(result.is_ok());

        // Test with a non-whitelisted strategy
        let non_whitelisted_strategy = Addr::unchecked("non_whitelisted_strategy");
        let result = only_strategies_whitelisted_for_deposit(deps.as_ref(), &non_whitelisted_strategy);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::StrategyNotWhitelisted {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }
    
    #[test]
    fn test_add_strategies_to_whitelist() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        // Instantiate the contract with the whitelister
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Test adding strategies with the correct whitelister
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let forbidden_values = vec![true, false];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 5); 
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "add_strategies_to_whitelist");
        assert_eq!(res.attributes[1].key, "strategy_added");
        assert_eq!(res.attributes[1].value, "strategy1");
        assert_eq!(res.attributes[2].key, "third_party_transfers_forbidden");
        assert_eq!(res.attributes[2].value, "true");
        assert_eq!(res.attributes[3].key, "strategy_added");
        assert_eq!(res.attributes[3].value, "strategy2");
        assert_eq!(res.attributes[4].key, "third_party_transfers_forbidden");
        assert_eq!(res.attributes[4].value, "false");
    
        let is_whitelisted = STRATEGY_WHITELIST.load(&deps.storage, &strategies[0]).unwrap();
        assert!(is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[0]).unwrap();
        assert!(forbidden);
    
        let is_whitelisted = STRATEGY_WHITELIST.load(&deps.storage, &strategies[1]).unwrap();
        assert!(is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[1]).unwrap();
        assert!(!forbidden);
    
        // Test with an unauthorized user
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test with mismatched strategies and forbidden_values length
        let strategies = vec![Addr::unchecked("strategy3")];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies,
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidInput {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_remove_strategies_from_whitelist() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        // Instantiate the contract with the whitelister
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Add strategies to the whitelist
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let forbidden_values = vec![true, false];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        // Test removing strategies with the correct whitelister
        let msg = ExecuteMsg::RemoveStrategiesFromWhitelist {
            strategies: strategies.clone(),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "remove_strategies_from_whitelist");
        assert_eq!(res.attributes[1].key, "strategy_removed");
        assert_eq!(res.attributes[1].value, "strategy1");
        assert_eq!(res.attributes[2].key, "third_party_transfers_forbidden");
        assert_eq!(res.attributes[2].value, "false");
        assert_eq!(res.attributes[3].key, "strategy_removed");
        assert_eq!(res.attributes[3].value, "strategy2");
        assert_eq!(res.attributes[4].key, "third_party_transfers_forbidden");
        assert_eq!(res.attributes[4].value, "false");
    
        let is_whitelisted = STRATEGY_WHITELIST.load(&deps.storage, &strategies[0]).unwrap();
        assert!(!is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[0]).unwrap();
        assert!(!forbidden);
    
        let is_whitelisted = STRATEGY_WHITELIST.load(&deps.storage, &strategies[1]).unwrap();
        assert!(!is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[1]).unwrap();
        assert!(!forbidden);
    
        // Test with an unauthorized user
        let msg = ExecuteMsg::RemoveStrategiesFromWhitelist {
            strategies: strategies.clone(),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_strategy_whitelister() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_owner = message_info(&Addr::unchecked("owner"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        // Instantiate the contract with the owner
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Test setting new strategy whitelister with the correct owner
        let new_whitelister = Addr::unchecked("new_whitelister");
        let msg = ExecuteMsg::SetStrategyWhitelister {
            new_strategy_whitelister: new_whitelister.clone(),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_owner.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "set_strategy_whitelister");
        assert_eq!(res.attributes[1].key, "new_strategy_whitelister");
        assert_eq!(res.attributes[1].value, new_whitelister.to_string());
    
        let whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        assert_eq!(whitelister, new_whitelister);
    
        // Test with an unauthorized user
        let msg = ExecuteMsg::SetStrategyWhitelister {
            new_strategy_whitelister: Addr::unchecked("another_whitelister"),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_third_party_transfers_forbidden() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        // Instantiate the contract with the whitelister
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Whitelist a strategy
        let strategy = Addr::unchecked("strategy1");
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        // Test setting third party transfers forbidden with the correct whitelister
        let msg = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value: true,
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "set_third_party_transfers_forbidden");
        assert_eq!(res.attributes[1].key, "strategy");
        assert_eq!(res.attributes[1].value, strategy.to_string());
        assert_eq!(res.attributes[2].key, "value");
        assert_eq!(res.attributes[2].value, "true");
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategy).unwrap();
        assert!(forbidden);
    
        // Test with an unauthorized user
        let msg = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value: false,
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_create_transfer_msg() {
        let info = message_info(&Addr::unchecked("sender"), &[]);
        let token = Addr::unchecked("token");
        let strategy = Addr::unchecked("strategy");
        let amount = Uint128::new(100);
    
        // Call the create_transfer_msg function
        let msg_result = create_transfer_msg(&info, &token, &strategy, amount);
    
        // Assert that the result is Ok
        assert!(msg_result.is_ok());
    
        // Extract the CosmosMsg from the result
        let cosmos_msg = msg_result.unwrap();
    
        // Check that the CosmosMsg is a Wasm Execute message
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = cosmos_msg {
            // Assert that the contract address matches the token address
            assert_eq!(contract_addr, token.to_string());
    
            // Deserialize the message and assert that it matches the expected Cw20ExecuteMsg
            let expected_msg = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: strategy.to_string(),
                amount,
            };
            let actual_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_create_deposit_msg() {
        let strategy = Addr::unchecked("strategy");
        let amount = Uint128::new(100);
    
        // Call the create_deposit_msg function
        let msg_result = create_deposit_msg(&strategy, amount);
    
        // Assert that the result is Ok
        assert!(msg_result.is_ok());
    
        // Extract the CosmosMsg from the result
        let cosmos_msg = msg_result.unwrap();
    
        // Check that the CosmosMsg is a Wasm Execute message
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = cosmos_msg {
            // Assert that the contract address matches the strategy address
            assert_eq!(contract_addr, strategy.to_string());
    
            // Deserialize the message and assert that it matches the expected StrategyExecuteMsg::Deposit message
            let expected_msg = StrategyExecuteMsg::Deposit { amount };
            let actual_msg: StrategyExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_deposit_into_strategy_via_delegation_manager() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
    
        // Instantiate the contract with the whitelister and delegation manager
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Whitelist a strategy
        let strategy = Addr::unchecked("strategy1");
        let token = Addr::unchecked("token1");
        let amount = Uint128::new(100);
    
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        // Test deposit into strategy with whitelisted strategy via delegation manager
        let msg = ExecuteMsg::DepositIntoStrategy {
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit_into_strategy");
        assert_eq!(res.attributes[1].key, "strategy");
        assert_eq!(res.attributes[1].value, strategy.to_string());
        assert_eq!(res.attributes[2].key, "amount");
        assert_eq!(res.attributes[2].value, amount.to_string());
        assert_eq!(res.attributes[3].key, "new_shares");
        assert_eq!(res.attributes[3].value, "50");
    
        // Verify the transfer and deposit messages
        assert_eq!(res.messages.len(), 3);
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = &res.messages[0].msg {
            assert_eq!(contract_addr, &token.to_string());
            let expected_msg = Cw20ExecuteMsg::TransferFrom {
                owner: info_delegation_manager.sender.to_string(), // Use the correct delegation manager address
                recipient: strategy.to_string(),
                amount,
            };
            let actual_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        if let CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) = &res.messages[1].msg {
            assert_eq!(contract_addr, &strategy.to_string());
            let expected_msg = StrategyExecuteMsg::Deposit { amount };
            let actual_msg: StrategyExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        // Test deposit into strategy with non-whitelisted strategy
        let non_whitelisted_strategy = Addr::unchecked("non_whitelisted_strategy");
        let msg = ExecuteMsg::DepositIntoStrategy {
            strategy: non_whitelisted_strategy.clone(),
            token: token.clone(),
            amount,
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::StrategyNotWhitelisted {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }        

    #[test]
    fn test_get_deposits() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Add some strategies and shares for a staker
        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");
    
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &vec![strategy1.clone(), strategy2.clone()]).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100)).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200)).unwrap();
    
        // Query deposits for the staker
        let query_msg = QueryMsg::GetDeposits { staker: staker.clone() };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let (strategies, shares): (Vec<Addr>, Vec<Uint128>) = from_json(bin).unwrap();
    
        assert_eq!(strategies.len(), 2);
        assert_eq!(shares.len(), 2);
        assert_eq!(strategies[0], strategy1);
        assert_eq!(shares[0], Uint128::new(100));
        assert_eq!(strategies[1], strategy2);
        assert_eq!(shares[1], Uint128::new(200));
    
        // Test with a staker that has no deposits
        let new_staker = Addr::unchecked("new_staker");
        let query_msg = QueryMsg::GetDeposits { staker: new_staker.clone() };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let (strategies, shares): (Vec<Addr>, Vec<Uint128>) = from_json(bin).unwrap();
    
        assert_eq!(strategies.len(), 0);
        assert_eq!(shares.len(), 0);
    }

    #[test]
    fn test_staker_strategy_list_length() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Add some strategies for a staker
        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");
    
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &vec![strategy1.clone(), strategy2.clone()]).unwrap();
    
        // Query the strategy list length for the staker
        let query_msg = QueryMsg::StakerStrategyListLength { staker: staker.clone() };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let length: Uint64 = from_json(bin).unwrap();
    
        assert_eq!(length, Uint64::new(2));
    
        // Test with a staker that has no strategies
        let new_staker = Addr::unchecked("new_staker");
        let query_msg = QueryMsg::StakerStrategyListLength { staker: new_staker.clone() };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let length: Uint64 = from_json(bin).unwrap();
    
        assert_eq!(length, Uint64::new(0));
    }

    #[test]
    fn test_add_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        // Instantiate the contract with the delegation manager
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Set up initial data
        let token = Addr::unchecked("token");
        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);
    
        // Test adding shares with the correct delegation manager
        let res = add_shares(deps.as_mut(), info_delegation_manager.clone(), staker.clone(), token.clone(), strategy.clone(), shares).unwrap();
    
        // Verify the response
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "add_shares");
        assert_eq!(res.attributes[1].key, "staker");
        assert_eq!(res.attributes[1].value, staker.to_string());
        assert_eq!(res.attributes[2].key, "token");
        assert_eq!(res.attributes[2].value, token.to_string());
        assert_eq!(res.attributes[3].key, "strategy");
        assert_eq!(res.attributes[3].value, strategy.to_string());
        assert_eq!(res.attributes[4].key, "shares");
        assert_eq!(res.attributes[4].value, shares.to_string());
    
        // Verify the shares were added correctly
        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy)).unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);
    
        // Verify the strategy was added to the staker's strategy list
        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);
    
        // Test adding more shares to the same strategy
        let additional_shares = Uint128::new(50);
        let res = add_shares(deps.as_mut(), info_delegation_manager.clone(), staker.clone(), token.clone(), strategy.clone(), additional_shares).unwrap();
    
        // Verify the response
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "add_shares");
        assert_eq!(res.attributes[1].key, "staker");
        assert_eq!(res.attributes[1].value, staker.to_string());
        assert_eq!(res.attributes[2].key, "token");
        assert_eq!(res.attributes[2].value, token.to_string());
        assert_eq!(res.attributes[3].key, "strategy");
        assert_eq!(res.attributes[3].value, strategy.to_string());
        assert_eq!(res.attributes[4].key, "shares");
        assert_eq!(res.attributes[4].value, additional_shares.to_string());
    
        // Verify the shares were added correctly
        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy)).unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);
    
        // Test with an unauthorized user
        let result = add_shares(deps.as_mut(), info_unauthorized.clone(), staker.clone(), token.clone(), strategy.clone(), shares);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test with zero shares
        let result = add_shares(deps.as_mut(), info_delegation_manager.clone(), staker.clone(), token.clone(), strategy.clone(), Uint128::zero());
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test exceeding the max strategy list length
        let mut strategy_list = Vec::with_capacity(MAX_STAKER_STRATEGY_LIST_LENGTH);
        for i in 0..MAX_STAKER_STRATEGY_LIST_LENGTH {
            strategy_list.push(Addr::unchecked(format!("strategy{}", i)));
        }
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &strategy_list).unwrap();
    
        let new_strategy = Addr::unchecked("new_strategy");
        let result = add_shares(deps.as_mut(), info_delegation_manager, staker.clone(), token.clone(), new_strategy.clone(), shares);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MaxStrategyListLengthExceeded {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_remove_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Add some strategies and shares for a staker
        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");
    
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &vec![strategy1.clone(), strategy2.clone()]).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100)).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200)).unwrap();
    
        // Test removing shares with the correct delegation manager
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy1.clone(),
            shares: Uint128::new(50),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "remove_shares");
        assert_eq!(res.attributes[1].key, "staker");
        assert_eq!(res.attributes[1].value, staker.to_string());
        assert_eq!(res.attributes[2].key, "strategy");
        assert_eq!(res.attributes[2].value, strategy1.to_string());
        assert_eq!(res.attributes[3].key, "shares");
        assert_eq!(res.attributes[3].value, "50");
    
        // Verify the shares were updated correctly
        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy1)).unwrap();
        println!("Stored shares after removal: {}", stored_shares);
        assert_eq!(stored_shares, Uint128::new(50));
    
        // Test removing shares with an unauthorized user
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy2.clone(),
            shares: Uint128::new(50),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test removing more shares than available
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy1.clone(),
            shares: Uint128::new(60),
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test removing all shares, which should remove the strategy from the staker's list
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy1.clone(),
            shares: Uint128::new(50),
        };
    
        execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        println!("Strategy list after removal: {:?}", strategy_list);
        assert_eq!(strategy_list.len(), 1);
        assert!(!strategy_list.contains(&strategy1));
        assert!(strategy_list.contains(&strategy2));
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
        params: DigestHashParams,
        secret_key: &SecretKey,
    ) -> Binary {
        let params = DigestHashParams {
            staker: params.staker,
            strategy: params.strategy,
            public_key: params.public_key,
            token: params.token,
            amount: params.amount,
            nonce: params.nonce,
            expiry: params.expiry,
            chain_id: params.chain_id,
            contract_addr: params.contract_addr,
        };
    
        let message_bytes = calculate_digest_hash(&params);
    
        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&message_bytes).expect("32 bytes");
        let signature = secp.sign_ecdsa(&message, secret_key);
        let signature_bytes = signature.serialize_compact().to_vec();
        
        Binary::from(signature_bytes)
    }    

    #[test]
    fn test_deposit_into_strategy_with_signature() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        // Whitelist a strategy
        let strategy = Addr::unchecked("strategy1");
        let token = Addr::unchecked("token1");
        let amount = 100;
    
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (staker, secret_key, public_key_bytes) = generate_osmosis_public_key_from_private_key(private_key_hex);
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let nonce = 0;
        let chain_id = env.block.chain_id.clone();
        let contract_addr = env.contract.address.clone();

        let public_key = Binary::from(public_key_bytes);

        let params = DigestHashParams {
            staker: staker.clone(),
            public_key: public_key.clone(),
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
            nonce,
            expiry,
            chain_id: chain_id.to_string(),
            contract_addr: contract_addr.clone(),
        };

        let signature = mock_signature_with_message(params, &secret_key);
    
        // Test deposit into strategy with signature via delegation manager
        let msg = ExecuteMsg::DepositIntoStrategyWithSignature {
            strategy: strategy.clone(),
            token: token.clone(),
            amount: Uint128::from(amount),
            staker: staker.clone(),
            public_key,
            expiry: Uint64::from(expiry),
            signature,
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit_into_strategy_with_signature");
        assert_eq!(res.attributes[1].key, "strategy");
        assert_eq!(res.attributes[1].value, strategy.to_string());
        assert_eq!(res.attributes[2].key, "amount");
        assert_eq!(res.attributes[2].value, amount.to_string());
        assert_eq!(res.attributes[3].key, "new_shares");
        assert_eq!(res.attributes[3].value, "50"); // Mock value used in the function
    
        // Verify the transfer and deposit messages
        assert_eq!(res.messages.len(), 3);
        if let SubMsg { msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }), .. } = &res.messages[0] {
            assert_eq!(contract_addr, &token.to_string());
            let expected_msg = Cw20ExecuteMsg::TransferFrom {
                owner: info_delegation_manager.sender.to_string(),
                recipient: strategy.to_string(),
                amount: Uint128::from(amount),
            };
            let actual_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        if let SubMsg { msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }), .. } = &res.messages[1] {
            assert_eq!(contract_addr, &strategy.to_string());
            let expected_msg = StrategyExecuteMsg::Deposit { amount: Uint128::from(amount) };
            let actual_msg: StrategyExecuteMsg = from_json(msg).unwrap();
            assert_eq!(actual_msg, expected_msg);
        } else {
            panic!("Unexpected message type");
        }
    
        // Verify nonce was incremented
        let stored_nonce = NONCES.load(&deps.storage, &staker).unwrap();
        println!("Stored nonce after deposit: {}", stored_nonce);
        assert_eq!(stored_nonce, 1);
    }    
}