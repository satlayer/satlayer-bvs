#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    auth,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{
        DepositsResponse, StakerStrategyListLengthResponse, StakerStrategyListResponse,
        StakerStrategySharesResponse, StrategyWhitelistedResponse,
    },
    state::{
        DEPLOYED_STRATEGIES, IS_BLACKLISTED, MAX_STAKER_STRATEGY_LIST_LENGTH, STAKER_STRATEGY_LIST,
        STAKER_STRATEGY_SHARES, STRATEGY_IS_WHITELISTED_FOR_DEPOSIT,
    },
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use crate::msg::delegation_manager::{self, IncreaseDelegatedShares};
use crate::query::{IsTokenBlacklistedResponse, TokenStrategyResponse};
use bvs_library::ownership;
use bvs_strategy_base::{
    msg::ExecuteMsg as BaseExecuteMsg,
    msg::QueryMsg as BaseQueryMsg,
    msg::{StrategyManagerResponse, TotalSharesResponse, UnderlyingTokenResponse},
};

const CONTRACT_NAME: &str = "BVS Strategy Manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SHARES_OFFSET: Uint128 = Uint128::new(1000000000000000000);
const BALANCE_OFFSET: Uint128 = Uint128::new(1000000000000000000);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let registry = deps.api.addr_validate(&msg.registry)?;
    bvs_registry::api::set_registry_addr(deps.storage, &registry)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_registry::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::AddNewStrategy {
            new_strategy,
            token,
        } => {
            let strategy_addr = deps.api.addr_validate(&new_strategy)?;
            let token_addr = deps.api.addr_validate(&token)?;
            add_new_strategy(deps, env, info, strategy_addr, token_addr)
        }
        ExecuteMsg::BlacklistTokens { tokens } => {
            let tokens = bvs_library::addr::validate_addrs(deps.api, &tokens)?;
            blacklist_tokens(deps, env, info, tokens)
        }
        ExecuteMsg::AddStrategiesToWhitelist { strategies } => {
            let strategies = bvs_library::addr::validate_addrs(deps.api, &strategies)?;
            add_strategies_to_deposit_whitelist(deps, info, strategies)
        }
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => {
            let strategies = bvs_library::addr::validate_addrs(deps.api, &strategies)?;
            remove_strategies_from_deposit_whitelist(deps, info, strategies)
        }
        ExecuteMsg::DepositIntoStrategy {
            strategy,
            token,
            amount,
        } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            let token_addr = deps.api.addr_validate(&token)?;

            let staker = info.sender.clone();
            deposit_into_strategy(deps, info, staker, strategy_addr, token_addr, amount)
        }
        ExecuteMsg::WithdrawSharesAsTokens {
            recipient,
            strategy,
            shares,
        } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            let recipient_addr = deps.api.addr_validate(&recipient)?;

            withdraw_shares_as_tokens(deps, info, recipient_addr, strategy_addr, shares)
        }
        ExecuteMsg::RemoveShares {
            staker,
            strategy,
            shares,
        } => {
            let staker = deps.api.addr_validate(&staker)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            remove_shares(deps, info, staker, strategy, shares)
        }
        ExecuteMsg::AddShares {
            staker,
            strategy,
            shares,
        } => {
            let staker = deps.api.addr_validate(&staker)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            add_shares(deps, info, staker, strategy, shares)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
        ExecuteMsg::SetRouting {
            delegation_manager,
            slash_manager,
        } => {
            let delegation_manager = deps.api.addr_validate(&delegation_manager)?;
            let slash_manager = deps.api.addr_validate(&slash_manager)?;

            auth::set_routing(deps, info, delegation_manager, slash_manager)
        }
    }
}

pub fn add_strategies_to_deposit_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    strategies_to_whitelist: Vec<Addr>,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    let mut events = vec![];

    for strategy in &strategies_to_whitelist {
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .may_load(deps.storage, strategy)?
            .unwrap_or(false);

        if !is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, strategy, &true)?;

            let event = Event::new("StrategyAddedToDepositWhitelist")
                .add_attribute("strategy", strategy.to_string());
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
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    let mut events = vec![];

    for strategy in strategies {
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .may_load(deps.storage, &strategy)?
            .unwrap_or(false);

        if is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, &strategy, &false)?;

            let event = Event::new("StrategyRemovedFromDepositWhitelist")
                .add_attribute("strategy", strategy.to_string());

            events.push(event);
        }
    }

    Ok(Response::new().add_events(events))
}

pub fn deposit_into_strategy(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    deposit_into_strategy_internal(deps, info, staker, strategy, token, amount)
}

pub fn add_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;

    add_shares_internal(deps, staker, strategy, shares)
}

pub fn remove_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;
    let strategy_removed = remove_shares_internal(deps, staker.clone(), strategy.clone(), shares)?;

    let response = Response::new()
        .add_attribute("method", "remove_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("strategy_removed", strategy_removed.to_string());

    Ok(response)
}

pub fn withdraw_shares_as_tokens(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;

    let withdraw_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&BaseExecuteMsg::Withdraw {
            recipient: recipient.to_string(),
            shares: shares.clone(),
        })?,
        funds: vec![],
    });

    let new_amount = STAKER_STRATEGY_SHARES.load(deps.storage, (&recipient, &strategy))? - shares;

    STAKER_STRATEGY_SHARES.save(deps.storage, (&recipient, &strategy), &new_amount)?;

    let response = Response::new().add_message(withdraw_msg);

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsTokenBlacklisted { token } => {
            let token_addr = deps.api.addr_validate(&token)?;

            to_json_binary(&query_blacklist_status_for_token(deps, token_addr)?)
        }
        QueryMsg::TokenStrategy { token } => {
            let token_addr = deps.api.addr_validate(&token)?;

            to_json_binary(&query_strategy_for_token(deps, token_addr)?)
        }
        QueryMsg::GetDeposits { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            to_json_binary(&query_get_deposits(deps, staker_addr)?)
        }
        QueryMsg::StakerStrategyListLength { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            to_json_binary(&query_staker_strategy_list_length(deps, staker_addr)?)
        }
        QueryMsg::GetStakerStrategyShares { staker, strategy } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            to_json_binary(&query_staker_strategy_shares(
                deps,
                staker_addr,
                strategy_addr,
            )?)
        }
        QueryMsg::GetStakerStrategyList { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            to_json_binary(&query_staker_strategy_list(deps, staker_addr)?)
        }
        QueryMsg::IsStrategyWhitelisted { strategy } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            to_json_binary(&query_is_strategy_whitelisted(deps, strategy_addr)?)
        }
    }
}

fn query_strategy_for_token(deps: Deps, token: Addr) -> StdResult<TokenStrategyResponse> {
    let strategy = DEPLOYED_STRATEGIES.load(deps.storage, &token)?;
    Ok(TokenStrategyResponse { strategy })
}

fn query_blacklist_status_for_token(
    deps: Deps,
    token: Addr,
) -> StdResult<IsTokenBlacklistedResponse> {
    let is_blacklisted = IS_BLACKLISTED
        .may_load(deps.storage, &token)?
        .unwrap_or(false);
    Ok(IsTokenBlacklistedResponse {
        token,
        is_blacklisted,
    })
}

fn query_staker_strategy_shares(
    deps: Deps,
    staker: Addr,
    strategy: Addr,
) -> StdResult<StakerStrategySharesResponse> {
    let shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or(Uint128::zero());
    Ok(StakerStrategySharesResponse { shares })
}

fn query_staker_strategy_list(deps: Deps, staker: Addr) -> StdResult<StakerStrategyListResponse> {
    let strategies = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);
    Ok(StakerStrategyListResponse { strategies })
}

fn query_is_strategy_whitelisted(
    deps: Deps,
    strategy: Addr,
) -> StdResult<StrategyWhitelistedResponse> {
    let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
        .may_load(deps.storage, &strategy)?
        .unwrap_or(false);
    Ok(StrategyWhitelistedResponse { is_whitelisted })
}

fn query_get_deposits(deps: Deps, staker: Addr) -> StdResult<DepositsResponse> {
    let (strategies, shares) = get_deposits(deps, staker)?;
    Ok(DepositsResponse { strategies, shares })
}

fn query_staker_strategy_list_length(
    deps: Deps,
    staker: Addr,
) -> StdResult<StakerStrategyListLengthResponse> {
    let strategies_len = staker_strategy_list_length(deps, staker)?;
    Ok(StakerStrategyListLengthResponse { strategies_len })
}

fn only_strategies_whitelisted_for_deposit(
    deps: Deps,
    strategy: &Addr,
) -> Result<(), ContractError> {
    let whitelist = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
        .may_load(deps.storage, strategy)?
        .unwrap_or(false);
    if !whitelist {
        return Err(ContractError::StrategyNotWhitelisted {});
    }
    Ok(())
}

fn deposit_into_strategy_internal(
    mut deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy)?;

    if amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let transfer_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: strategy.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    let mut response = Response::new().add_message(transfer_msg);

    let UnderlyingTokenResponse(token) = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: strategy.to_string(),
            msg: to_json_binary(&BaseQueryMsg::UnderlyingToken {})?,
        }
        .into(),
    )?;

    let TotalSharesResponse(total_shares) = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: strategy.to_string(),
            msg: to_json_binary(&BaseQueryMsg::TotalShares {})?,
        }
        .into(),
    )?;

    let balance = token_balance(&deps.querier, &token, &strategy)?;
    let new_shares = calculate_new_shares(total_shares, balance, amount)?;

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    let deposit_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&BaseExecuteMsg::Deposit { amount })?,
        funds: vec![],
    }
    .into();

    response = response.add_message(deposit_msg);

    add_shares_internal(deps.branch(), staker.clone(), strategy.clone(), new_shares)?;

    let delegation_manager = auth::get_delegation_manager(deps.storage)?;
    let increase_delegated_shares_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: delegation_manager.to_string(),
        msg: to_json_binary(&delegation_manager::ExecuteMsg::IncreaseDelegatedShares(
            IncreaseDelegatedShares {
                staker: staker.to_string(),
                strategy: strategy.to_string(),
                shares: new_shares,
            },
        ))?,
        funds: vec![],
    });

    Ok(response
        .add_message(increase_delegated_shares_msg)
        .add_attribute("new_shares", new_shares.to_string()))
}

fn add_shares_internal(
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

    let event = Event::new("add_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string());

    Ok(Response::new().add_event(event))
}

fn remove_shares_internal(
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
    current_shares = current_shares
        .checked_sub(shares)
        .map_err(|_| ContractError::InvalidShares {})?;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &current_shares)?;

    // If no existing shares, remove the strategy from the staker's list
    if current_shares.is_zero() {
        remove_strategy_from_staker_strategy_list(deps, staker.clone(), strategy.clone())?;
        return Ok(true);
    }

    Ok(false)
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

fn calculate_new_shares(
    total_shares: Uint128,
    token_balance: Uint128,
    deposit_amount: Uint128,
) -> Result<Uint128, ContractError> {
    let virtual_share_amount = total_shares
        .checked_add(SHARES_OFFSET)
        .map_err(|_| ContractError::Overflow)?;

    let virtual_token_balance = token_balance
        .checked_add(BALANCE_OFFSET)
        .map_err(|_| ContractError::Overflow)?;

    let virtual_prior_token_balance = virtual_token_balance
        .checked_sub(deposit_amount)
        .map_err(|_| ContractError::Underflow)?;

    let numerator = deposit_amount
        .checked_mul(virtual_share_amount)
        .map_err(|_| ContractError::Overflow)?;

    if virtual_prior_token_balance.is_zero() {
        return Err(ContractError::DivideByZero);
    }

    let new_shares = numerator
        .checked_div(virtual_prior_token_balance)
        .map_err(|_| ContractError::DivideByZero)?;

    Ok(new_shares)
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

pub fn get_deposits(deps: Deps, staker: Addr) -> StdResult<(Vec<Addr>, Vec<Uint128>)> {
    let strategies = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    let mut shares = Vec::with_capacity(strategies.len());

    for strategy in &strategies {
        let share = STAKER_STRATEGY_SHARES
            .may_load(deps.storage, (&staker, strategy))?
            .unwrap_or_else(Uint128::zero);
        shares.push(share);
    }

    Ok((strategies, shares))
}

pub fn staker_strategy_list_length(deps: Deps, staker: Addr) -> StdResult<Uint128> {
    let strategies = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);
    Ok(Uint128::new(strategies.len() as u128))
}

pub fn blacklist_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tokens: Vec<Addr>,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    let mut strategies_to_remove: Vec<Addr> = Vec::new();

    let mut events = vec![];

    for token in &tokens {
        let is_already_blacklisted = IS_BLACKLISTED
            .may_load(deps.storage, token)?
            .unwrap_or(false);
        if is_already_blacklisted {
            return Err(ContractError::TokenAlreadyBlacklisted {});
        }

        IS_BLACKLISTED.save(deps.storage, token, &true)?;

        if let Some(deployed_strategy) = DEPLOYED_STRATEGIES.may_load(deps.storage, token)? {
            if deployed_strategy != Addr::unchecked("") {
                strategies_to_remove.push(deployed_strategy);
            }
        }

        let event = Event::new("TokenBlacklisted")
            .add_attribute("method", "blacklist_tokens")
            .add_attribute("token", token.to_string());

        events.push(event);
    }

    if !strategies_to_remove.is_empty() {
        remove_strategies_from_deposit_whitelist(deps, info, strategies_to_remove)?;
    }

    Ok(Response::new()
        .add_events(events)
        .add_attribute("method", "blacklist_tokens"))
}

pub fn add_new_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategy: Addr,
    token: Addr,
) -> Result<Response, ContractError> {
    ownership::assert_owner(deps.storage, &info)?;

    let is_blacklisted = IS_BLACKLISTED
        .may_load(deps.storage, &token)?
        .unwrap_or(false);
    if is_blacklisted {
        return Err(ContractError::TokenAlreadyBlacklisted {});
    }

    let existing_strategy = DEPLOYED_STRATEGIES
        .may_load(deps.storage, &token)?
        .unwrap_or(Addr::unchecked(""));
    if existing_strategy != Addr::unchecked("") {
        return Err(ContractError::StrategyAlreadyExists {});
    }

    // let's check if contract is properly uploaded and initiated on the chain
    let StrategyManagerResponse(strategy_manager_addr) = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: strategy.to_string().clone(),
            msg: to_json_binary(&BaseQueryMsg::StrategyManager {})?,
        }
        .into(),
    )?;

    if strategy_manager_addr != env.contract.address {
        return Err(ContractError::StrategyNotCompatible {});
    }

    DEPLOYED_STRATEGIES.save(deps.storage, &token, &strategy)?;

    // when new strategy is added it is automatically whitelisted for deposit
    STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, &strategy, &true)?;

    let event = Event::new("NewStrategyAdded")
        .add_attribute("method", "add_new_strategy")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("token", token.to_string());

    Ok(Response::new().add_event(event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::IsTokenBlacklistedResponse;
    use bvs_library::ownership::OwnershipError;
    use bvs_strategy_base::msg::QueryMsg::UnderlyingToken;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_json, Addr, ContractResult, OwnedDeps, SystemError, SystemResult};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.as_str());

        let owner = ownership::get_owner(&deps.storage).unwrap();
        assert_eq!(owner, owner.clone());
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
        };

        let delegation_manager = deps.api.addr_make("delegation_manager");
        let slasher = deps.api.addr_make("slasher");
        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        auth::set_routing(
            deps.as_mut(),
            owner_info.clone(),
            delegation_manager.clone(),
            slasher,
        )
        .unwrap();

        (
            deps,
            env,
            owner_info,
            message_info(&delegation_manager, &[]),
        )
    }

    #[test]
    fn test_add_new_strategy() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy");
        let token = deps.api.addr_make("token");

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr: _,
                msg,
            } => {
                let query_msg: BaseQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    BaseQueryMsg::StrategyManager {} => {
                        let response = StrategyManagerResponse(_env.contract.address.clone());
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

        let res = add_new_strategy(
            deps.as_mut(),
            mock_env(),
            _owner_info.clone(),
            strategy.clone(),
            token.clone(),
        );

        assert_eq!(res.is_ok(), true);

        let query_msg = QueryMsg::TokenStrategy {
            token: token.to_string(),
        };

        let response: TokenStrategyResponse =
            from_json(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert_eq!(response.strategy.to_string(), strategy.to_string());

        let query_msg = QueryMsg::IsTokenBlacklisted {
            token: token.to_string(),
        };

        let response: IsTokenBlacklistedResponse =
            from_json(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert!(!response.is_blacklisted);
    }

    #[test]
    fn test_blacklist_token() {
        let (mut deps, _env, owner_info, _info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy");
        let token = deps.api.addr_make("token");

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr: _,
                msg,
            } => {
                let query_msg: bvs_strategy_base::msg::QueryMsg = from_json(msg).unwrap();
                match query_msg {
                    BaseQueryMsg::StrategyManager {} => {
                        let response = StrategyManagerResponse(_env.contract.address.clone());
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

        let res = add_new_strategy(
            deps.as_mut(),
            mock_env(),
            owner_info.clone(),
            strategy.clone(),
            token.clone(),
        );

        assert_eq!(res.is_ok(), true);

        let query_msg = QueryMsg::TokenStrategy {
            token: token.to_string(),
        };

        let response: TokenStrategyResponse =
            from_json(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert_eq!(response.strategy.to_string(), strategy.to_string());

        let query_msg = QueryMsg::IsTokenBlacklisted {
            token: token.to_string(),
        };

        let response: IsTokenBlacklistedResponse =
            from_json(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert!(!response.is_blacklisted);

        let _ =
            blacklist_tokens(deps.as_mut(), mock_env(), owner_info, vec![token.clone()]).unwrap();

        let query_msg = QueryMsg::IsTokenBlacklisted {
            token: token.to_string(),
        };

        let response: IsTokenBlacklistedResponse =
            from_json(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert!(response.is_blacklisted);
    }

    #[test]
    fn test_only_strategies_whitelisted_for_deposit() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let strategy = Addr::unchecked("strategy");
        STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .save(&mut deps.storage, &strategy, &true)
            .unwrap();

        let result = only_strategies_whitelisted_for_deposit(deps.as_ref(), &strategy);
        assert!(result.is_ok());

        let non_whitelisted_strategy = Addr::unchecked("non_whitelisted_strategy");
        let result =
            only_strategies_whitelisted_for_deposit(deps.as_ref(), &non_whitelisted_strategy);
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
        let (mut deps, _env, owner_info, _info_delegation_manager) = instantiate_contract();

        let strat1 = deps.api.addr_make("strategy1");
        let strat2 = deps.api.addr_make("strategy2");

        let strategies = vec![strat1.clone(), strat2.clone()];

        let res = add_strategies_to_deposit_whitelist(
            deps.as_mut(),
            owner_info.clone(),
            strategies.clone(),
        )
        .unwrap();

        let events = res.events;

        assert_eq!(events.len(), strategies.len());

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.ty, "StrategyAddedToDepositWhitelist");
            assert_eq!(event.attributes.len(), 1);
            assert_eq!(event.attributes[0].key, "strategy");
            assert_eq!(event.attributes[0].value, strategies[i].to_string());
        }

        for strategy in &strategies {
            let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
                .load(&deps.storage, &Addr::unchecked(strategy.clone()))
                .unwrap();
            assert!(is_whitelisted);
        }

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
        let result = add_strategies_to_deposit_whitelist(
            deps.as_mut(),
            info_unauthorized.clone(),
            strategies.clone(),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Ownership(OwnershipError::Unauthorized {})
        );
    }

    #[test]
    fn test_remove_strategies_from_deposit_whitelist() {
        let (mut deps, _env, owner_info, _info_delegation_manager) = instantiate_contract();

        let strategies = vec![
            deps.api.addr_make("strategy1"),
            deps.api.addr_make("strategy2"),
        ];

        let _res = add_strategies_to_deposit_whitelist(
            deps.as_mut(),
            owner_info.clone(),
            strategies.clone(),
        )
        .unwrap();

        let res = remove_strategies_from_deposit_whitelist(
            deps.as_mut(),
            owner_info.clone(),
            strategies.clone(),
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 2);

        for (i, strategy) in strategies.iter().enumerate() {
            let event = &events[i];
            assert_eq!(event.ty, "StrategyRemovedFromDepositWhitelist");
            assert_eq!(event.attributes.len(), 1);
            assert_eq!(event.attributes[0].key, "strategy");
            assert_eq!(event.attributes[0].value, strategy.to_string());
        }

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
        let result = remove_strategies_from_deposit_whitelist(
            deps.as_mut(),
            info_unauthorized.clone(),
            strategies.clone(),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Ownership(OwnershipError::Unauthorized {})
        );
    }

    #[test]
    fn test_deposit_into_strategy() {
        let (mut deps, _env, owner_info, info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1");
        let token = deps.api.addr_make("token");
        let amount = Uint128::new(100);

        let _res = add_strategies_to_deposit_whitelist(
            deps.as_mut(),
            owner_info.clone(),
            vec![strategy.clone()],
        )
        .unwrap();

        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == strategy_for_closure.to_string() =>
            {
                let strategy_query_msg: BaseQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    UnderlyingToken {} => {
                        let response = UnderlyingTokenResponse(token_for_closure.clone());
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    }
                    BaseQueryMsg::TotalShares {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&TotalSharesResponse(Uint128::new(1000))).unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == token_for_closure.to_string() =>
            {
                let cw20_query_msg: Cw20QueryMsg = from_json(msg).unwrap();
                match cw20_query_msg {
                    Cw20QueryMsg::Balance { address: _ } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&Cw20BalanceResponse {
                            balance: Uint128::new(1000),
                        })
                        .unwrap(),
                    )),
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

        let res = deposit_into_strategy(
            deps.as_mut(),
            info_delegation_manager.clone(),
            info_delegation_manager.sender.clone(),
            strategy.clone(),
            token.clone(),
            amount,
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0].key, "new_shares");
        assert_eq!(res.attributes[0].value, "100");

        let non_whitelisted_strategy = deps.api.addr_make("non_whitelisted_strategy");

        let result = deposit_into_strategy(
            deps.as_mut(),
            info_delegation_manager.clone(),
            info_delegation_manager.sender.clone(),
            non_whitelisted_strategy.clone(),
            token.clone(),
            amount,
        );
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
        let (mut deps, env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker.clone(),
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100))
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200))
            .unwrap();

        // Query deposits for the staker
        let query_msg = QueryMsg::GetDeposits {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: DepositsResponse = from_json(bin).unwrap();

        assert_eq!(response.strategies.len(), 2);
        assert_eq!(response.shares.len(), 2);
        assert_eq!(response.strategies[0], strategy1);
        assert_eq!(response.shares[0], Uint128::new(100));
        assert_eq!(response.strategies[1], strategy2);
        assert_eq!(response.shares[1], Uint128::new(200));

        // Test with a staker that has no deposits
        let new_staker = deps.api.addr_make("new_staker").to_string();

        let query_msg = QueryMsg::GetDeposits { staker: new_staker };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: DepositsResponse = from_json(bin).unwrap();

        assert_eq!(response.strategies.len(), 0);
        assert_eq!(response.shares.len(), 0);
    }

    #[test]
    fn test_staker_strategy_list_length() {
        let (mut deps, env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker,
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();

        // Query the strategy list length for the staker
        let query_msg = QueryMsg::StakerStrategyListLength {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: StakerStrategyListLengthResponse = from_json(bin).unwrap();
        let length = response.strategies_len;

        assert_eq!(length, Uint128::new(2));

        // Test with a staker that has no strategies
        let new_staker = deps.api.addr_make("new_staker");

        let query_msg = QueryMsg::StakerStrategyListLength {
            staker: new_staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: StakerStrategyListLengthResponse = from_json(bin).unwrap();
        let length = response.strategies_len;

        assert_eq!(length, Uint128::new(0));
    }

    #[test]
    fn test_add_shares_internal() {
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);

        let res =
            add_shares_internal(deps.as_mut(), staker.clone(), strategy.clone(), shares).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);

        let additional_shares = Uint128::new(50);
        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            additional_shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, additional_shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);

        // Test with zero shares
        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            Uint128::zero(),
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
        let result =
            add_shares_internal(deps.as_mut(), staker.clone(), new_strategy.clone(), shares);
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
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let strategy = deps.api.addr_make("strategy");
        let shares = Uint128::new(100);

        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, shares.to_string());

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

        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            additional_shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, additional_shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = add_shares(
            deps.as_mut(),
            info_unauthorized.clone(),
            staker.clone(),
            strategy.clone(),
            shares,
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            Uint128::zero(),
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

        let new_strategy = deps.api.addr_make("new_strategy");

        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            new_strategy.clone(),
            shares,
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
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

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

        let res = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
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

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy1))
            .unwrap();
        println!("Stored shares after removal: {}", stored_shares);
        assert_eq!(stored_shares, Uint128::new(50));

        // Test removing shares with an unauthorized user
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = remove_shares(
            deps.as_mut(),
            info_unauthorized.clone(),
            staker.clone(),
            strategy2.clone(),
            Uint128::new(50),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test removing more shares than available
        let result = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(60),
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test removing all shares, which should remove the strategy from the staker's list
        let res = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
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
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

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

        let result = remove_shares_internal(
            deps.as_mut(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();
        assert!(!result);

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy1))
            .unwrap();
        println!("Stored shares after partial removal: {}", stored_shares);

        assert_eq!(stored_shares, Uint128::new(50));

        let result = remove_shares_internal(
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

        let result = remove_shares_internal(
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

        let result = remove_shares_internal(
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

    #[test]
    fn test_get_staker_strategy_list() {
        let (mut deps, env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");

        let strategies = vec![
            deps.api.addr_make("strategy1"),
            deps.api.addr_make("strategy2"),
        ];
        STAKER_STRATEGY_LIST
            .save(&mut deps.storage, &staker, &strategies.clone())
            .unwrap();

        let query_msg = QueryMsg::GetStakerStrategyList {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let strategy_list_response: StakerStrategyListResponse = from_json(bin).unwrap();
        assert_eq!(strategy_list_response.strategies, strategies);

        let new_staker = deps.api.addr_make("new_staker");

        let query_msg = QueryMsg::GetStakerStrategyList {
            staker: new_staker.to_string(),
        };
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let strategy_list_response: StakerStrategyListResponse = from_json(bin).unwrap();
        assert!(strategy_list_response.strategies.is_empty());
    }

    #[test]
    fn test_is_strategy_whitelisted() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1");

        STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .save(&mut deps.storage, &strategy, &true)
            .unwrap();

        let result = query_is_strategy_whitelisted(deps.as_ref(), strategy.clone()).unwrap();
        assert!(result.is_whitelisted);

        let non_whitelisted_strategy = deps.api.addr_make("non_whitelisted_strategy");

        let result =
            query_is_strategy_whitelisted(deps.as_ref(), non_whitelisted_strategy).unwrap();
        assert!(!result.is_whitelisted);
    }

    #[test]
    fn test_get_staker_strategy_shares() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = Addr::unchecked("staker1");
        let strategy = deps.api.addr_make("strategy");
        let shares = Uint128::new(100);

        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy), &shares)
            .unwrap();

        let retrieved_shares =
            query_staker_strategy_shares(deps.as_ref(), staker.clone(), strategy.clone()).unwrap();
        assert_eq!(retrieved_shares.shares, shares);

        let new_staker = Addr::unchecked("new_staker");
        let retrieved_shares =
            query_staker_strategy_shares(deps.as_ref(), new_staker.clone(), strategy.clone())
                .unwrap();
        assert_eq!(retrieved_shares.shares, Uint128::zero());

        let new_strategy = Addr::unchecked("new_strategy");
        let retrieved_shares =
            query_staker_strategy_shares(deps.as_ref(), staker.clone(), new_strategy.clone())
                .unwrap();
        assert_eq!(retrieved_shares.shares, Uint128::zero());
    }
}
