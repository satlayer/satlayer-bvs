use crate::{
    error::ContractError,
    strategy_base,
    delegation_manager,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        StrategyManagerState, STRATEGY_MANAGER_STATE, STRATEGY_IS_WHITELISTED_FOR_DEPOSIT, STRATEGY_WHITELISTER, OWNER,
        STAKER_STRATEGY_SHARES, STAKER_STRATEGY_LIST, MAX_STAKER_STRATEGY_LIST_LENGTH, THIRD_PARTY_TRANSFERS_FORBIDDEN, NONCES
    },
    utils::{calculate_digest_hash, recover, DigestHashParams, DepositWithSignatureParams, DEPOSIT_TYPEHASH, DOMAIN_TYPEHASH, DOMAIN_NAME},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
    Uint64, Binary, WasmQuery, Event, QuerierWrapper, QueryRequest
};
use strategy_base::{ExecuteMsg as StrategyExecuteMsg, QueryMsg as StrategyQueryMsg, StrategyState};
use delegation_manager::ExecuteMsg as DelegationExecuteMsg;

use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse};

use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SHARES_OFFSET: Uint128 = Uint128::new(1_000);
const BALANCE_OFFSET: Uint128 = Uint128::new(1_000);

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
            add_strategies_to_deposit_whitelist(deps, info, strategies, third_party_transfers_forbidden_values)
        }
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => remove_strategies_from_deposit_whitelist(deps, info, strategies),
        ExecuteMsg::SetStrategyWhitelister { new_strategy_whitelister } => set_strategy_whitelister(deps, info, new_strategy_whitelister),
        ExecuteMsg::DepositIntoStrategy { strategy, token, amount } => {
            let staker = info.sender.clone();
            deposit_into_strategy(deps, info, staker, strategy.clone(), token.clone(), amount)
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
            
            let response = deposit_into_strategy_with_signature(deps, env, info, params)?;
        
            Ok(response)
        },
        ExecuteMsg::RemoveShares { staker, strategy, shares } => remove_shares(deps, info, staker, strategy, shares),
        ExecuteMsg::WithdrawSharesAsTokens { recipient, strategy, shares, token } => withdraw_shares_as_tokens(deps, info, recipient, strategy, shares, token),
        ExecuteMsg::AddShares { staker, token, strategy, shares } => add_shares(deps, info, staker, token, strategy, shares),
        ExecuteMsg::SetDelegationManager { new_delegation_manager } => {set_delegation_manager(deps, info, new_delegation_manager)},
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr: Addr = Addr::unchecked(new_owner);
            transfer_ownership(deps, info, new_owner_addr)
        },
    }
}

fn _only_strategy_whitelister(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let whitelister: Addr = STRATEGY_WHITELISTER.load(deps.storage)?;
    if info.sender != whitelister {
        return Err(ContractError::Unauthorized {});
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

fn _only_delegation_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    if info.sender != state.delegation_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn _only_strategies_whitelisted_for_deposit(deps: Deps, strategy: &Addr) -> Result<(), ContractError> {
    let whitelist = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.may_load(deps.storage, strategy)?.unwrap_or(false);
    if !whitelist {
        return Err(ContractError::StrategyNotWhitelisted {});
    }
    Ok(())
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

pub fn set_delegation_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_delegation_manager: Addr,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    let mut state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    state.delegation_manager = new_delegation_manager.clone();
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;

    let event = Event::new("set_delegation_manager")
        .add_attribute("new_delegation_manager", new_delegation_manager.to_string());

    Ok(Response::new().add_event(event))
}

pub fn add_strategies_to_deposit_whitelist(
    mut deps: DepsMut,
    info: MessageInfo,
    strategies_to_whitelist: Vec<Addr>,
    third_party_transfers_forbidden_values: Vec<bool>,
) -> Result<Response, ContractError> {
    _only_strategy_whitelister(deps.as_ref(), &info)?;

    if strategies_to_whitelist.len() != third_party_transfers_forbidden_values.len() {
        return Err(ContractError::InvalidInput {});
    }

    let mut events = vec![];

    for (i, strategy) in strategies_to_whitelist.iter().enumerate() {
        let forbidden_value = third_party_transfers_forbidden_values[i];

        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.may_load(deps.storage, strategy)?.unwrap_or(false);

        if !is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, strategy, &true)?;
            set_third_party_transfers_forbidden(deps.branch(), info.clone(), strategy.clone(), forbidden_value)?;

            let event = Event::new("StrategyAddedToDepositWhitelist")
                .add_attribute("strategy", strategy.to_string())
                .add_attribute("third_party_transfers_forbidden", forbidden_value.to_string());
            events.push(event);
        }
    }

    let mut response = Response::new();
    for event in events {
        response = response.add_event(event);
    }

    Ok(response)
}

pub fn remove_strategies_from_deposit_whitelist(
    mut deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    _only_strategy_whitelister(deps.as_ref(), &info)?;

    let mut events = vec![];

    for strategy in strategies {
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.may_load(deps.storage, &strategy)?.unwrap_or(false);
        
        if is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, &strategy, &false)?;
            set_third_party_transfers_forbidden(deps.branch(), info.clone(), strategy.clone(), false)?;

            let event = Event::new("StrategyRemovedFromDepositWhitelist")
                .add_attribute("strategy", strategy.to_string());

            events.push(event);
        }
    }

    Ok(Response::new().add_events(events))
}

pub fn set_strategy_whitelister(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_whitelister: Addr,
) -> Result<Response, ContractError> {
    _only_owner(deps.as_ref(), &info)?;

    let strategy_whitelister = STRATEGY_WHITELISTER.load(deps.storage)?;

    STRATEGY_WHITELISTER.save(deps.storage, &new_strategy_whitelister)?;

    let event = Event::new("set_strategy_whitelister")
        .add_attribute("old_strategy_whitelister", strategy_whitelister.to_string())
        .add_attribute("new_strategy_whitelister", new_strategy_whitelister.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_third_party_transfers_forbidden(
    deps: DepsMut,
    info: MessageInfo,
    strategy: Addr,
    value: bool,
) -> Result<Response, ContractError> {
    _only_strategy_whitelister(deps.as_ref(), &info)?;

    THIRD_PARTY_TRANSFERS_FORBIDDEN.save(deps.storage, &strategy, &value)?;

    let event = Event::new("set_third_party_transfers_forbidden")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("value", value.to_string());

    Ok(Response::new().add_event(event))
}

pub fn deposit_into_strategy(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    _deposit_into_strategy(deps, info, staker, strategy, token, amount)
}

fn _deposit_into_strategy(
    mut deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    _only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy)?;

    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: strategy.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    let strategy_state: StrategyState = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyQueryMsg::GetStrategyState {})?,
    }))?;

    let balance = _token_balance(&deps.querier, &strategy_state.underlying_token, &strategy)?;
    let new_shares = _calculate_new_shares(strategy_state.total_shares, balance, amount);

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    let deposit_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::Deposit { amount })?,
        funds: vec![],
    });

    _add_shares(deps.branch(), staker.clone(), token.clone(), strategy.clone(), new_shares)?;

    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    let increase_delegated_shares_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.delegation_manager.to_string(),
        msg: to_json_binary(&DelegationExecuteMsg::IncreaseDelegatedShares {
            staker: staker.clone(),
            strategy: strategy.clone(),
            shares: new_shares,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_message(deposit_msg)
        .add_message(increase_delegated_shares_msg)
        .add_attribute("new_shares", new_shares.to_string()))
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

    NONCES.save(deps.storage, &params.staker, &(nonce + 1))?;

    let response = _deposit_into_strategy(deps, info, params.staker.clone(), params.strategy.clone(), params.token.clone(), params.amount)?;

    let new_shares = response.attributes.iter()
        .find(|attr| attr.key == "new_shares")
        .map(|attr| attr.value.clone())
        .ok_or_else(|| ContractError::AttributeNotFound {})?;

    Ok(Response::new()
        .add_attribute("method", "deposit_into_strategy_with_signature")
        .add_attribute("strategy", params.strategy.to_string())
        .add_attribute("amount", params.amount.to_string())
        .add_attribute("new_shares", new_shares))
}

pub fn add_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    token: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    _only_delegation_manager(deps.as_ref(), &info)?;

    _add_shares(deps, staker, token, strategy, shares)
}

fn _add_shares(
    deps: DepsMut,
    staker: Addr,
    token: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    if shares.is_zero() {
        return Err(ContractError::InvalidShares {});
    }

    let mut strategy_list = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?.unwrap_or_else(Vec::new);

    let current_shares = STAKER_STRATEGY_SHARES.may_load(deps.storage, (&staker, &strategy))?.unwrap_or_else(Uint128::zero);

    if current_shares.is_zero() {
        if strategy_list.len() >= MAX_STAKER_STRATEGY_LIST_LENGTH {
            return Err(ContractError::MaxStrategyListLengthExceeded {});
        }
        strategy_list.push(strategy.clone());
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
    }

    let new_shares = current_shares + shares;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &new_shares)?;

    let event = Event::new("add_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("token", token.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string());

    Ok(Response::new().add_event(event))
}

pub fn remove_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    _only_delegation_manager(deps.as_ref(), &info)?;
    let strategy_removed = _remove_shares(deps, staker.clone(), strategy.clone(), shares)?;

    let mut response = Response::new()
        .add_attribute("method", "remove_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string());

    // Add an additional attribute if the strategy was removed
    if strategy_removed {
        response = response.add_attribute("strategy_removed", "true");
    } else {
        response = response.add_attribute("strategy_removed", "false");
    }

    Ok(response)
}

fn _remove_shares(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<bool, ContractError> {
    if shares.is_zero() {
        return Err(ContractError::InvalidShares {});
    }

    let mut current_shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or_else(Uint128::zero);

    if shares > current_shares {
        return Err(ContractError::InvalidShares {});
    }

    // Subtract the shares
    current_shares = current_shares.checked_sub(shares).map_err(|_| ContractError::InvalidShares {})?;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &current_shares)?;

    // If no existing shares, remove the strategy from the staker's list
    if current_shares.is_zero() {
        _remove_strategy_from_staker_strategy_list(deps, staker.clone(), strategy.clone())?;
        return Ok(true);
    }

    Ok(false)
}

fn _remove_strategy_from_staker_strategy_list(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
) -> Result<(), ContractError> {
    let mut strategy_list = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?.unwrap_or_else(Vec::new);

    if let Some(pos) = strategy_list.iter().position(|x| *x == strategy) {
        strategy_list.swap_remove(pos);
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
        Ok(())
    } else {
        Err(ContractError::StrategyNotFound {})
    }
}

pub fn withdraw_shares_as_tokens(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Addr,
    strategy: Addr,
    shares: Uint128,
    token: Addr,
) -> Result<Response, ContractError> {
    _only_delegation_manager(deps.as_ref(), &info)?;

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

fn _calculate_new_shares(
    total_shares: Uint128,
    token_balance: Uint128,
    deposit_amount: Uint128,
) -> Uint128 {
    let virtual_share_amount = total_shares + SHARES_OFFSET;
    let virtual_token_balance = token_balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - deposit_amount;
    (deposit_amount * virtual_share_amount) / virtual_prior_token_balance
}

fn _token_balance(querier: &QuerierWrapper, token: &Addr, account: &Addr) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::Balance {
            address: account.to_string(),
        })?,
    }))?;
    Ok(res.balance)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposits { staker } => to_json_binary(&get_deposits(deps, staker)?),
        QueryMsg::StakerStrategyListLength { staker } => to_json_binary(&staker_strategy_list_length(deps, staker)?),
        QueryMsg::GetStakerStrategyShares { staker, strategy } => to_json_binary(&query_staker_strategy_shares(deps, staker, strategy)?),
        QueryMsg::IsThirdPartyTransfersForbidden { strategy } => to_json_binary(&query_is_third_party_transfers_forbidden(deps, strategy)?),
        QueryMsg::GetNonce { staker } => to_json_binary(&query_nonce(deps, staker)?),
        QueryMsg::GetStakerStrategyList { staker } => to_json_binary(&query_staker_strategy_list(deps, staker)?),
        QueryMsg::GetOwner {} => to_json_binary(&query_owner(deps)?),
        QueryMsg::IsStrategyWhitelisted { strategy } => to_json_binary(&query_is_strategy_whitelisted(deps, strategy)?),
        QueryMsg::CalculateDigestHash { digst_hash_params } => to_json_binary(&calculate_digest_hash(&digst_hash_params)),
        QueryMsg::GetStrategyWhitelister {} => to_json_binary(&query_strategy_whitelister(deps)?),
        QueryMsg::GetStrategyManagerState {} => to_json_binary(&query_strategy_manager_state(deps)?),
        QueryMsg::GetDepositTypehash {} => to_json_binary(&query_deposit_typehash()?),
        QueryMsg::GetDomainTypehash {} => to_json_binary(&query_domain_typehash()?),
        QueryMsg::GetDomainName {} => to_json_binary(&query_domain_name()?),
        QueryMsg::GetDelegationManager {} => to_json_binary(&query_delegation_manager(deps)?),
    }
}

fn query_staker_strategy_shares(deps: Deps, staker: Addr, strategy: Addr) -> StdResult<Uint128> {
    let shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or(Uint128::zero());
    Ok(shares)
}

fn query_is_third_party_transfers_forbidden(deps: Deps, strategy: Addr) -> StdResult<bool> {
    let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.may_load(deps.storage, &strategy)?.unwrap_or(false);
    Ok(forbidden)
}

fn query_nonce(deps: Deps, staker: Addr) -> StdResult<u64> {
    let nonce = NONCES.may_load(deps.storage, &staker)?.unwrap_or(0);
    Ok(nonce)
}

fn query_staker_strategy_list(deps: Deps, staker: Addr) -> StdResult<Vec<Addr>> {
    let strategies = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?.unwrap_or_else(Vec::new);
    Ok(strategies)
}

fn query_owner(deps: Deps) -> StdResult<Addr> {
    let owner = OWNER.load(deps.storage)?;
    Ok(owner)
}

fn query_is_strategy_whitelisted(deps: Deps, strategy: Addr) -> StdResult<bool> {
    let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.may_load(deps.storage, &strategy)?.unwrap_or(false);
    Ok(is_whitelisted)
}

fn query_strategy_whitelister(deps: Deps) -> StdResult<Addr> {
    let whitelister = STRATEGY_WHITELISTER.load(deps.storage)?;
    Ok(whitelister)
}

fn query_strategy_manager_state(deps: Deps) -> StdResult<StrategyManagerState> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    Ok(state)
}

fn query_deposit_typehash() -> StdResult<String> {
    let deposit_typehash_str = String::from_utf8_lossy(DEPOSIT_TYPEHASH).to_string();
    Ok(deposit_typehash_str)
}

fn query_domain_typehash() -> StdResult<String> {
    let domain_typehash_str = String::from_utf8_lossy(DOMAIN_TYPEHASH).to_string();
    Ok(domain_typehash_str)
}

fn query_domain_name() -> StdResult<String> {
    let domain_name_str = String::from_utf8_lossy(DOMAIN_NAME).to_string();
    Ok(domain_name_str)
}

fn query_delegation_manager(deps: Deps) -> StdResult<Addr> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    Ok(state.delegation_manager)
}

pub fn get_deposits(deps: Deps, staker: Addr) -> StdResult<(Vec<Addr>, Vec<Uint128>)> {
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

    Ok((strategies, shares))
}

pub fn staker_strategy_list_length(deps: Deps, staker: Addr) -> StdResult<Uint64> {
    let strategies = STAKER_STRATEGY_LIST.may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);
    Ok(Uint64::new(strategies.len() as u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{Addr, from_json, SystemResult, SystemError, ContractResult};
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

        let result = _only_strategy_whitelister(deps.as_ref(), &info_whitelister);
        assert!(result.is_ok());

        let result = _only_strategy_whitelister(deps.as_ref(), &info_unauthorized);
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

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let result = _only_owner(deps.as_ref(), &info_owner);
        assert!(result.is_ok());

        let result = _only_owner(deps.as_ref(), &info_unauthorized);
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

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let result = _only_delegation_manager(deps.as_ref(), &info_delegation_manager);
        assert!(result.is_ok());

        let result = _only_delegation_manager(deps.as_ref(), &info_unauthorized);
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

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let strategy = Addr::unchecked("strategy");
        STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(&mut deps.storage, &strategy, &true).unwrap();

        let result = _only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy);
        assert!(result.is_ok());

        let non_whitelisted_strategy = Addr::unchecked("non_whitelisted_strategy");
        let result = _only_strategies_whitelisted_for_deposit(deps.as_ref(), &non_whitelisted_strategy);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::StrategyNotWhitelisted {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }
    
    #[test]
    fn test_add_strategies_to_deposit_whitelist() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let forbidden_values = vec![true, false];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 2);
    
        for (i, strategy) in strategies.iter().enumerate() {
            let event = &events[i];
            assert_eq!(event.ty, "StrategyAddedToDepositWhitelist");
            assert_eq!(event.attributes.len(), 2);
            assert_eq!(event.attributes[0].key, "strategy");
            assert_eq!(event.attributes[0].value, strategy.to_string());
            assert_eq!(event.attributes[1].key, "third_party_transfers_forbidden");
            assert_eq!(event.attributes[1].value, forbidden_values[i].to_string());
        }
    
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.load(&deps.storage, &strategies[0]).unwrap();
        assert!(is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[0]).unwrap();
        assert!(forbidden);
    
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.load(&deps.storage, &strategies[1]).unwrap();
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
    fn test_remove_strategies_from_deposit_whitelist() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        let forbidden_values = vec![true, false];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        let msg = ExecuteMsg::RemoveStrategiesFromWhitelist {
            strategies: strategies.clone(),
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 2);
    
        for (i, strategy) in strategies.iter().enumerate() {
            let event = &events[i];
            assert_eq!(event.ty, "StrategyRemovedFromDepositWhitelist");
            assert_eq!(event.attributes.len(), 1);
            assert_eq!(event.attributes[0].key, "strategy");
            assert_eq!(event.attributes[0].value, strategy.to_string());
        }
    
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.load(&deps.storage, &strategies[0]).unwrap();
        assert!(!is_whitelisted);
    
        let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategies[0]).unwrap();
        assert!(!forbidden);
    
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.load(&deps.storage, &strategies[1]).unwrap();
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
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let old_whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        let new_whitelister = Addr::unchecked("new_whitelister");
    
        let res = set_strategy_whitelister(deps.as_mut(), info_owner.clone(),new_whitelister.clone(),).unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "set_strategy_whitelister");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "old_strategy_whitelister");
        assert_eq!(event.attributes[0].value, old_whitelister.to_string());
        assert_eq!(event.attributes[1].key, "new_strategy_whitelister");
        assert_eq!(event.attributes[1].value, new_whitelister.to_string());
    
        let stored_whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        assert_eq!(stored_whitelister, new_whitelister);
    
        let result = set_strategy_whitelister(
            deps.as_mut(),
            info_unauthorized.clone(),
            Addr::unchecked("another_whitelister"),
        );
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
        let info_strategy_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let strategy = Addr::unchecked("strategy1");
        let value = true;
    
        let msg = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value,
        };
    
        let res = execute(deps.as_mut(), env.clone(), info_strategy_whitelister.clone(), msg).unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "set_third_party_transfers_forbidden");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "strategy");
        assert_eq!(event.attributes[0].value, strategy.to_string());
        assert_eq!(event.attributes[1].key, "value");
        assert_eq!(event.attributes[1].value, value.to_string());
    
        let stored_value = THIRD_PARTY_TRANSFERS_FORBIDDEN.load(&deps.storage, &strategy).unwrap();
        assert_eq!(stored_value, value);
    
        // Test with an unauthorized user
        let exec_msg_unauthorized = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value,
        };
    
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), exec_msg_unauthorized);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }     

    #[test]
    fn test_deposit_into_strategy() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_whitelister = message_info(&Addr::unchecked("whitelister"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let strategy = Addr::unchecked("strategy1");
        let token = Addr::unchecked("token1");
        let amount = Uint128::new(100);
    
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };
    
        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();
    
        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();
    
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == strategy_for_closure.to_string() => {
                let strategy_query_msg: StrategyQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    StrategyQueryMsg::GetStrategyState {} => {
                        let strategy_state = StrategyState {
                            strategy_manager: Addr::unchecked("delegation_manager"),
                            underlying_token: token_for_closure.clone(),
                            total_shares: Uint128::new(1000),
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&strategy_state).unwrap()))
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            },
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == token_for_closure.to_string() => {
                let cw20_query_msg: Cw20QueryMsg = from_json(msg).unwrap();
                match cw20_query_msg {
                    Cw20QueryMsg::Balance { address: _ } => {
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1000) }).unwrap()))
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            },
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });
    
        let msg = ExecuteMsg::DepositIntoStrategy {
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
        };
    
        let res: Response = execute(deps.as_mut(), env.clone(), info_delegation_manager.clone(), msg).unwrap();
    
        assert_eq!(res.attributes.len(), 1); 
        assert_eq!(res.attributes[0].key, "new_shares");
        assert_eq!(res.attributes[0].value, "105"); 
    
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
    fn test_add_shares_internal() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let token = Addr::unchecked("token");
        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);

        let res = _add_shares(deps.as_mut(), staker.clone(), token.clone(), strategy.clone(), shares).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "token");
        assert_eq!(event.attributes[1].value, token.to_string());
        assert_eq!(event.attributes[2].key, "strategy");
        assert_eq!(event.attributes[2].value, strategy.to_string());
        assert_eq!(event.attributes[3].key, "shares");
        assert_eq!(event.attributes[3].value, shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy)).unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);

        let additional_shares = Uint128::new(50);
        let res = _add_shares(deps.as_mut(), staker.clone(), token.clone(), strategy.clone(), additional_shares).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "token");
        assert_eq!(event.attributes[1].value, token.to_string());
        assert_eq!(event.attributes[2].key, "strategy");
        assert_eq!(event.attributes[2].value, strategy.to_string());
        assert_eq!(event.attributes[3].key, "shares");
        assert_eq!(event.attributes[3].value, additional_shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy)).unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);

        // Test with zero shares
        let result = _add_shares(deps.as_mut(), staker.clone(), token.clone(), strategy.clone(), Uint128::zero());
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test exceeding the max strategy list length
        let mut strategy_list = Vec::new();
        for i in 0..MAX_STAKER_STRATEGY_LIST_LENGTH {
            strategy_list.push(Addr::unchecked(format!("strategy{}", i)));
        }
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &strategy_list).unwrap();

        let new_strategy = Addr::unchecked("new_strategy");
        let result = _add_shares(deps.as_mut(), staker.clone(), token.clone(), new_strategy.clone(), shares);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MaxStrategyListLengthExceeded {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_add_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_delegation_manager = message_info(&Addr::unchecked("delegation_manager"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let token = Addr::unchecked("token");
        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);
    
        let msg = ExecuteMsg::AddShares {
            staker: staker.clone(),
            token: token.clone(),
            strategy: strategy.clone(),
            shares,
        };
    
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        )
        .unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "token");
        assert_eq!(event.attributes[1].value, token.to_string());
        assert_eq!(event.attributes[2].key, "strategy");
        assert_eq!(event.attributes[2].value, strategy.to_string());
        assert_eq!(event.attributes[3].key, "shares");
        assert_eq!(event.attributes[3].value, shares.to_string());
    
        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);
    
        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);
    
        // Test adding more shares to the same strategy
        let additional_shares = Uint128::new(50);
        let exec_msg = ExecuteMsg::AddShares {
            staker: staker.clone(),
            token: token.clone(),
            strategy: strategy.clone(),
            shares: additional_shares,
        };
    
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            exec_msg,
        )
        .unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 4);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "token");
        assert_eq!(event.attributes[1].value, token.to_string());
        assert_eq!(event.attributes[2].key, "strategy");
        assert_eq!(event.attributes[2].value, strategy.to_string());
        assert_eq!(event.attributes[3].key, "shares");
        assert_eq!(event.attributes[3].value, additional_shares.to_string());
    
        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);
    
        // Test with an unauthorized user
        let exec_msg = ExecuteMsg::AddShares {
            staker: staker.clone(),
            token: token.clone(),
            strategy: strategy.clone(),
            shares,
        };
    
        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_unauthorized.clone(),
            exec_msg,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test with zero shares
        let exec_msg = ExecuteMsg::AddShares {
            staker: staker.clone(),
            token: token.clone(),
            strategy: strategy.clone(),
            shares: Uint128::zero(),
        };
    
        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            exec_msg,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        // Test exceeding the max strategy list length
        let mut strategy_list = Vec::new();
        for i in 0..MAX_STAKER_STRATEGY_LIST_LENGTH {
            strategy_list.push(Addr::unchecked(format!("strategy{}", i)));
        }
        STAKER_STRATEGY_LIST
            .save(&mut deps.storage, &staker, &strategy_list)
            .unwrap();
    
        let new_strategy = Addr::unchecked("new_strategy");
        let exec_msg = ExecuteMsg::AddShares {
            staker: staker.clone(),
            token: token.clone(),
            strategy: new_strategy.clone(),
            shares,
        };
    
        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager,
            exec_msg,
        );
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
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");
    
        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker,
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100))
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200))
            .unwrap();
    
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy1.clone(),
            shares: Uint128::new(50),
        };
    
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        )
        .unwrap();
    
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "remove_shares");
        assert_eq!(res.attributes[1].key, "staker");
        assert_eq!(res.attributes[1].value, staker.to_string());
        assert_eq!(res.attributes[2].key, "strategy");
        assert_eq!(res.attributes[2].value, strategy1.to_string());
        assert_eq!(res.attributes[3].key, "shares");
        assert_eq!(res.attributes[3].value, "50");
        assert_eq!(res.attributes[4].key, "strategy_removed");
        assert_eq!(res.attributes[4].value, "false");
    
        let stored_shares =
            STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy1)).unwrap();
        println!("Stored shares after removal: {}", stored_shares);
        assert_eq!(stored_shares, Uint128::new(50));
    
        // Test removing shares with an unauthorized user
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.clone(),
            strategy: strategy2.clone(),
            shares: Uint128::new(50),
        };
    
        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_unauthorized.clone(),
            msg,
        );
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
    
        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        );
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
    
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        )
        .unwrap();
    
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[4].key, "strategy_removed");
        assert_eq!(res.attributes[4].value, "true");
    
        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        println!("Strategy list after removal: {:?}", strategy_list);
        assert_eq!(strategy_list.len(), 1);
        assert!(!strategy_list.contains(&strategy1));
        assert!(strategy_list.contains(&strategy2));
    }

    #[test]
    fn test_remove_shares_internal() {
        let mut deps = mock_dependencies();
        
        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");
    
        STAKER_STRATEGY_LIST.save(&mut deps.storage,&staker,&vec![strategy1.clone(), strategy2.clone()],).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100)).unwrap();
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200)).unwrap();
    
        let result = _remove_shares(deps.as_mut(), staker.clone(), strategy1.clone(), Uint128::new(50),).unwrap();
        assert!(!result);
    
        let stored_shares = STAKER_STRATEGY_SHARES.load(&deps.storage, (&staker, &strategy1)).unwrap();
        println!("Stored shares after partial removal: {}", stored_shares);

        assert_eq!(stored_shares, Uint128::new(50));
    
        let result = _remove_shares(
            deps.as_mut(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();

        assert!(result);
    
        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        println!("Strategy list after full removal: {:?}", strategy_list);
        assert_eq!(strategy_list.len(), 1);
        assert!(!strategy_list.contains(&strategy1));
        assert!(strategy_list.contains(&strategy2));
    
        let result = _remove_shares(
            deps.as_mut(),
            staker.clone(),
            strategy2.clone(),
            Uint128::new(300),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        let result = _remove_shares(
            deps.as_mut(),
            staker.clone(),
            strategy2.clone(),
            Uint128::zero(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
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
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let strategy = Addr::unchecked("strategy1");
        let token = Addr::unchecked("token1");
        let amount = Uint128::new(100);
    
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
            amount: amount.u128(),
            nonce,
            expiry,
            chain_id: chain_id.to_string(),
            contract_addr: contract_addr.clone(),
        };
    
        let signature = mock_signature_with_message(params, &secret_key);
    
        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == strategy_for_closure.to_string() => {
                let strategy_query_msg: StrategyQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    StrategyQueryMsg::GetStrategyState {} => {
                        let strategy_state = StrategyState {
                            strategy_manager: Addr::unchecked("delegation_manager"),
                            underlying_token: token_for_closure.clone(),
                            total_shares: Uint128::new(1000),
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&strategy_state).unwrap()))
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            },
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == token_for_closure.to_string() => {
                let cw20_query_msg: Cw20QueryMsg = from_json(msg).unwrap();
                match cw20_query_msg {
                    Cw20QueryMsg::Balance { address: _ } => {
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1000) }).unwrap()))
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            },
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });
    
        let msg = ExecuteMsg::DepositIntoStrategyWithSignature {
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
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
        assert_eq!(res.attributes[3].value, "105"); 
        
        let stored_nonce = NONCES.load(&deps.storage, &staker).unwrap();
        assert_eq!(stored_nonce, 1);
    }        

    #[test]
    fn test_is_third_party_transfers_forbidden() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let strategy = Addr::unchecked("strategy1");
        THIRD_PARTY_TRANSFERS_FORBIDDEN.save(&mut deps.storage, &strategy, &true).unwrap();

        let result = query_is_third_party_transfers_forbidden(deps.as_ref(), strategy.clone()).unwrap();
        assert!(result);

        let non_forbidden_strategy = Addr::unchecked("non_forbidden_strategy");
        let result = query_is_third_party_transfers_forbidden(deps.as_ref(), non_forbidden_strategy).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_get_nonce() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let staker = Addr::unchecked("staker1");
        NONCES.save(&mut deps.storage, &staker, &5).unwrap();

        let nonce = query_nonce(deps.as_ref(), staker.clone()).unwrap();
        assert_eq!(nonce, 5);

        let new_staker = Addr::unchecked("new_staker");
        let nonce = query_nonce(deps.as_ref(), new_staker).unwrap();
        assert_eq!(nonce, 0);
    }

    #[test]
    fn test_get_staker_strategy_list() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let staker = Addr::unchecked("staker1");
        let strategies = vec![Addr::unchecked("strategy1"), Addr::unchecked("strategy2")];
        STAKER_STRATEGY_LIST.save(&mut deps.storage, &staker, &strategies.clone()).unwrap();

        let strategy_list = query_staker_strategy_list(deps.as_ref(), staker.clone()).unwrap();
        assert_eq!(strategy_list, strategies);

        let new_staker = Addr::unchecked("new_staker");
        let strategy_list = query_staker_strategy_list(deps.as_ref(), new_staker).unwrap();
        assert!(strategy_list.is_empty());
    }

    #[test]
    fn test_get_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let owner = query_owner(deps.as_ref()).unwrap();
        assert_eq!(owner, Addr::unchecked("owner"));
    }

    #[test]
    fn test_is_strategy_whitelisted() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let strategy = Addr::unchecked("strategy1");
        STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(&mut deps.storage, &strategy, &true).unwrap();

        let result = query_is_strategy_whitelisted(deps.as_ref(), strategy.clone()).unwrap();
        assert!(result);

        let non_whitelisted_strategy = Addr::unchecked("non_whitelisted_strategy");
        let result = query_is_strategy_whitelisted(deps.as_ref(), non_whitelisted_strategy).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_get_strategy_whitelister() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let whitelister = query_strategy_whitelister(deps.as_ref()).unwrap();
        assert_eq!(whitelister, Addr::unchecked("whitelister"));
    }

    #[test]
    fn test_get_strategy_manager_state() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();

        let state = query_strategy_manager_state(deps.as_ref()).unwrap();
        assert_eq!(state.delegation_manager, Addr::unchecked("delegation_manager"));
        assert_eq!(state.slasher, Addr::unchecked("slasher"));
    }

    #[test]
    fn test_get_deposit_typehash() {
        let typehash = query_deposit_typehash().unwrap();
        let expected_str = String::from_utf8_lossy(DEPOSIT_TYPEHASH).to_string();
        assert_eq!(typehash, expected_str);
    }

    #[test]
    fn test_get_domain_typehash() {
        let typehash = query_domain_typehash().unwrap();
        let expected_str = String::from_utf8_lossy(DOMAIN_TYPEHASH).to_string();
        assert_eq!(typehash, expected_str);
    }

    #[test]
    fn test_get_domain_name() {
        let name = query_domain_name().unwrap();
        let expected_str = String::from_utf8_lossy(DOMAIN_NAME).to_string();
        assert_eq!(name, expected_str);
    }

    #[test]
    fn test_get_staker_strategy_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let staker = Addr::unchecked("staker1");
        let strategy = Addr::unchecked("strategy1");
        let shares = Uint128::new(100);
    
        // Save initial shares
        STAKER_STRATEGY_SHARES.save(&mut deps.storage, (&staker, &strategy), &shares).unwrap();
    
        // Retrieve shares for existing staker and strategy
        let retrieved_shares = query_staker_strategy_shares(deps.as_ref(), staker.clone(), strategy.clone()).unwrap();
        assert_eq!(retrieved_shares, shares);
    
        // Check retrieval for a non-existent staker
        let new_staker = Addr::unchecked("new_staker");
        let retrieved_shares = query_staker_strategy_shares(deps.as_ref(), new_staker.clone(), strategy.clone()).unwrap();
        assert_eq!(retrieved_shares, Uint128::zero());
    
        // Check retrieval for a non-existent strategy
        let new_strategy = Addr::unchecked("new_strategy");
        let retrieved_shares = query_staker_strategy_shares(deps.as_ref(), staker.clone(), new_strategy.clone()).unwrap();
        assert_eq!(retrieved_shares, Uint128::zero());
    }

    #[test]
    fn test_get_delegation_manager() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let query_msg = QueryMsg::GetDelegationManager {};
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let delegation_manager: Addr = from_json(bin).unwrap();
    
        assert_eq!(delegation_manager, Addr::unchecked("delegation_manager"));
    }

    #[test]
    fn test_set_delegation_manager() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info_creator = message_info(&Addr::unchecked("creator"), &[]);
        let info_owner = message_info(&Addr::unchecked("owner"), &[]);
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
    
        let msg = InstantiateMsg {
            initial_owner: Addr::unchecked("owner"),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
    
        let _res = instantiate(deps.as_mut(), env.clone(), info_creator, msg).unwrap();
    
        let new_delegation_manager = Addr::unchecked("new_delegation_manager");
    
        let msg = ExecuteMsg::SetDelegationManager {
            new_delegation_manager: new_delegation_manager.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info_owner.clone(), msg).unwrap();
    
        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "set_delegation_manager");
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(event.attributes[0].key, "new_delegation_manager");
        assert_eq!(event.attributes[0].value, new_delegation_manager.to_string());
    
        let state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.delegation_manager, new_delegation_manager);
    
        // Test with an unauthorized user
        let msg = ExecuteMsg::SetDelegationManager {
            new_delegation_manager: Addr::unchecked("another_delegation_manager"),
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
    fn test_transfer_ownership() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = Addr::unchecked("owner");
        let new_owner = Addr::unchecked("new_owner");
        let unauthorized_user = Addr::unchecked("unauthorized");
        
        let info_owner = message_info(&owner, &[]);
        let info_unauthorized = message_info(&unauthorized_user, &[]);
    
        let instantiate_msg = InstantiateMsg {
            initial_owner: owner.clone(),
            delegation_manager: Addr::unchecked("delegation_manager"),
            slasher: Addr::unchecked("slasher"),
            initial_strategy_whitelister: Addr::unchecked("whitelister"),
        };
        instantiate(deps.as_mut(), env.clone(), info_owner.clone(), instantiate_msg).unwrap();
    
        let transfer_msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.clone(), 
        };
        let res = execute(deps.as_mut(), env.clone(), info_owner.clone(), transfer_msg);
        assert!(res.is_ok());
    
        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "transfer_ownership");
        assert_eq!(res.attributes[1].key, "new_owner");
        assert_eq!(res.attributes[1].value, new_owner.to_string());
    
        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);
    
        let transfer_msg = ExecuteMsg::TransferOwnership {
            new_owner: owner.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), transfer_msg);
    
        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    
        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);
    
        let transfer_msg = ExecuteMsg::TransferOwnership {
            new_owner: owner.clone(),
        };
        let res = execute(deps.as_mut(), env.clone(), message_info(&new_owner, &[]), transfer_msg);
        assert!(res.is_ok());
    
        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "transfer_ownership");
        assert_eq!(res.attributes[1].key, "new_owner");
        assert_eq!(res.attributes[1].value, owner.to_string());
    
        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, owner);
    }       
}