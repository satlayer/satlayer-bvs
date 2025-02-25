#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::{
        CalculateDigestHashResponse, DelegationManagerResponse, DepositTypeHashResponse,
        DepositsResponse, DomainNameResponse, DomainTypeHashResponse, NonceResponse, OwnerResponse,
        StakerStrategyListLengthResponse, StakerStrategyListResponse, StakerStrategySharesResponse,
        StrategyManagerStateResponse, StrategyWhitelistedResponse, StrategyWhitelisterResponse,
        ThirdPartyTransfersForbiddenResponse,
    },
    state::{
        StrategyManagerState, MAX_STAKER_STRATEGY_LIST_LENGTH, NONCES, OWNER, STAKER_STRATEGY_LIST,
        STAKER_STRATEGY_SHARES, STRATEGY_IS_WHITELISTED_FOR_DEPOSIT, STRATEGY_MANAGER_STATE,
        STRATEGY_WHITELISTER, THIRD_PARTY_TRANSFERS_FORBIDDEN,
    },
    utils::{
        calculate_digest_hash, recover, validate_addresses, DepositWithSignatureParams,
        DigestHashParams, QueryDigestHashParams, DEPOSIT_TYPEHASH, DOMAIN_NAME, DOMAIN_TYPEHASH,
    },
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use bvs_base::base::{
    ExecuteMsg as StrategyExecuteMsg, QueryMsg as StrategyQueryMsg, StrategyState,
};
use bvs_base::delegation::ExecuteMsg as DelegationManagerExecuteMsg;
use bvs_base::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use bvs_base::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};

const CONTRACT_NAME: &str = "BVS Strategy Manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAUSED_DEPOSITS: u8 = 0;

const SHARES_OFFSET: Uint128 = Uint128::new(1000000000000000000);
const BALANCE_OFFSET: Uint128 = Uint128::new(1000000000000000000);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let delegation_manager = deps.api.addr_validate(&msg.delegation_manager)?;
    let slash_manager = deps.api.addr_validate(&msg.slash_manager)?;
    let initial_strategy_whitelister = deps.api.addr_validate(&msg.initial_strategy_whitelister)?;
    let initial_owner = deps.api.addr_validate(&msg.initial_owner)?;
    let strategy_factory = deps.api.addr_validate(&msg.strategy_factory)?;

    let state = StrategyManagerState {
        delegation_manager: delegation_manager.clone(),
        slash_manager: slash_manager.clone(),
        strategy_factory: strategy_factory.clone(),
    };

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    PAUSED_STATE.save(deps.storage, &msg.initial_paused_status)?;
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;
    STRATEGY_WHITELISTER.save(deps.storage, &initial_strategy_whitelister)?;
    OWNER.save(deps.storage, &initial_owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("delegation_manager", state.delegation_manager.to_string())
        .add_attribute("slasher", state.slash_manager.to_string())
        .add_attribute(
            "strategy_whitelister",
            msg.initial_strategy_whitelister.to_string(),
        )
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
        ExecuteMsg::AddStrategiesToWhitelist {
            strategies,
            third_party_transfers_forbidden_values,
        } => {
            let strategies_addr = validate_addresses(deps.api, &strategies)?;

            add_strategies_to_deposit_whitelist(
                deps,
                info,
                strategies_addr,
                third_party_transfers_forbidden_values,
            )
        }
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => {
            let strategies_addr = validate_addresses(deps.api, &strategies)?;

            remove_strategies_from_deposit_whitelist(deps, info, strategies_addr)
        }
        ExecuteMsg::SetStrategyWhitelister {
            new_strategy_whitelister,
        } => {
            let new_strategy_whitelister_addr =
                deps.api.addr_validate(&new_strategy_whitelister)?;

            set_strategy_whitelister(deps, info, new_strategy_whitelister_addr)
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
        ExecuteMsg::SetThirdPartyTransfersForbidden { strategy, value } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            set_third_party_transfers_forbidden(deps, info, strategy_addr, value)
        }
        ExecuteMsg::DepositIntoStrategyWithSignature {
            strategy,
            token,
            amount,
            staker,
            public_key,
            expiry,
            signature,
        } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            let token_addr = deps.api.addr_validate(&token)?;
            let staker_addr = Addr::unchecked(staker);

            let public_key_binary = Binary::from_base64(&public_key)?;
            let signature_binary = Binary::from_base64(&signature)?;

            let params = DepositWithSignatureParams {
                strategy: strategy_addr,
                token: token_addr,
                amount,
                staker: staker_addr,
                public_key: public_key_binary,
                expiry,
                signature: signature_binary,
            };

            let response = deposit_into_strategy_with_signature(deps, env, info, params)?;

            Ok(response)
        }
        ExecuteMsg::RemoveShares {
            staker,
            strategy,
            shares,
        } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            remove_shares(deps, info, staker_addr, strategy_addr, shares)
        }
        ExecuteMsg::WithdrawSharesAsTokens {
            recipient,
            strategy,
            shares,
            token,
        } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            let recipient_addr = deps.api.addr_validate(&recipient)?;
            let token_addr = deps.api.addr_validate(&token)?;

            withdraw_shares_as_tokens(
                deps,
                info,
                recipient_addr,
                strategy_addr,
                shares,
                token_addr,
            )
        }
        ExecuteMsg::AddShares {
            staker,
            token,
            strategy,
            shares,
        } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let token_addr = deps.api.addr_validate(&token)?;
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            add_shares(deps, info, staker_addr, token_addr, strategy_addr, shares)
        }
        ExecuteMsg::SetDelegationManager {
            new_delegation_manager,
        } => {
            let new_delegation_manager_addr = deps.api.addr_validate(&new_delegation_manager)?;

            set_delegation_manager(deps, info, new_delegation_manager_addr)
        }
        ExecuteMsg::SetSlashManager { new_slash_manager } => {
            let new_slash_manager_addr = deps.api.addr_validate(&new_slash_manager)?;

            set_slash_manager(deps, info, new_slash_manager_addr)
        }
        ExecuteMsg::SetStrategyFactory {
            new_strategy_factory,
        } => {
            let new_strategy_factory_addr = deps.api.addr_validate(&new_strategy_factory)?;

            set_strategy_factory(deps, info, new_strategy_factory_addr)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr: Addr = Addr::unchecked(new_owner);
            transfer_ownership(deps, info, new_owner_addr)
        }
        ExecuteMsg::Pause {} => {
            check_pauser(deps.as_ref(), info.clone())?;
            pause(deps, &info).map_err(ContractError::Std)
        }
        ExecuteMsg::Unpause {} => {
            check_unpauser(deps.as_ref(), info.clone())?;
            unpause(deps, &info).map_err(ContractError::Std)
        }
        ExecuteMsg::SetPauser { new_pauser } => {
            only_owner(deps.as_ref(), &info.clone())?;
            let new_pauser_addr = deps.api.addr_validate(&new_pauser)?;
            set_pauser(deps, new_pauser_addr).map_err(ContractError::Std)
        }
        ExecuteMsg::SetUnpauser { new_unpauser } => {
            only_owner(deps.as_ref(), &info.clone())?;
            let new_unpauser_addr = deps.api.addr_validate(&new_unpauser)?;
            set_unpauser(deps, new_unpauser_addr).map_err(ContractError::Std)
        }
    }
}

pub fn set_delegation_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_delegation_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    state.delegation_manager = new_delegation_manager.clone();
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

pub fn set_slash_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_slash_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    state.slash_manager = new_slash_manager.clone();
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;

    let event = Event::new("SlashManagerSet")
        .add_attribute("method", "set_slash_manager")
        .add_attribute("new_slash_manager", new_slash_manager.to_string());

    Ok(Response::new().add_event(event))
}

pub fn set_strategy_factory(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_factory: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    state.strategy_factory = new_strategy_factory.clone();
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

pub fn add_strategies_to_deposit_whitelist(
    mut deps: DepsMut,
    info: MessageInfo,
    strategies_to_whitelist: Vec<Addr>,
    third_party_transfers_forbidden_values: Vec<bool>,
) -> Result<Response, ContractError> {
    only_strategy_whitelister(deps.as_ref(), &info)?;

    if strategies_to_whitelist.len() != third_party_transfers_forbidden_values.len() {
        return Err(ContractError::InvalidInput {});
    }

    let mut events = vec![];

    for (i, strategy) in strategies_to_whitelist.iter().enumerate() {
        let forbidden_value = third_party_transfers_forbidden_values[i];

        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .may_load(deps.storage, strategy)?
            .unwrap_or(false);

        if !is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, strategy, &true)?;
            set_third_party_transfers_forbidden(
                deps.branch(),
                info.clone(),
                strategy.clone(),
                forbidden_value,
            )?;

            let event = Event::new("StrategyAddedToDepositWhitelist")
                .add_attribute("strategy", strategy.to_string())
                .add_attribute(
                    "third_party_transfers_forbidden",
                    forbidden_value.to_string(),
                );
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
    only_strategy_whitelister(deps.as_ref(), &info)?;

    let mut events = vec![];

    for strategy in strategies {
        let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
            .may_load(deps.storage, &strategy)?
            .unwrap_or(false);

        if is_whitelisted {
            STRATEGY_IS_WHITELISTED_FOR_DEPOSIT.save(deps.storage, &strategy, &false)?;
            set_third_party_transfers_forbidden(
                deps.branch(),
                info.clone(),
                strategy.clone(),
                false,
            )?;

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
    only_owner(deps.as_ref(), &info)?;

    let strategy_whitelister = STRATEGY_WHITELISTER.load(deps.storage)?;

    STRATEGY_WHITELISTER.save(deps.storage, &new_strategy_whitelister)?;

    let event = Event::new("set_strategy_whitelister")
        .add_attribute("old_strategy_whitelister", strategy_whitelister.to_string())
        .add_attribute(
            "new_strategy_whitelister",
            new_strategy_whitelister.to_string(),
        );

    Ok(Response::new().add_event(event))
}

pub fn set_third_party_transfers_forbidden(
    deps: DepsMut,
    info: MessageInfo,
    strategy: Addr,
    value: bool,
) -> Result<Response, ContractError> {
    only_strategy_whitelister(deps.as_ref(), &info)?;

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
    only_when_not_paused(deps.as_ref(), PAUSED_DEPOSITS)?;
    deposit_into_strategy_internal(deps, info, staker, strategy, token, amount)
}

pub fn deposit_into_strategy_with_signature(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: DepositWithSignatureParams,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_DEPOSITS)?;

    let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN
        .may_load(deps.storage, &params.strategy)?
        .unwrap_or(false);
    if forbidden {
        return Err(ContractError::Unauthorized {});
    }

    if params.expiry < env.block.time.seconds() {
        return Err(ContractError::SignatureExpired {});
    }

    let nonce = NONCES.may_load(deps.storage, &params.staker)?.unwrap_or(0);

    let digest_params = DigestHashParams {
        staker: params.staker.clone(),
        public_key: params.public_key.clone(),
        strategy: params.strategy.clone(),
        token: params.token.clone(),
        amount: params.amount,
        nonce,
        expiry: params.expiry,
        chain_id: env.block.chain_id.clone(),
        contract_addr: env.contract.address.clone(),
    };

    let struct_hash = calculate_digest_hash(&digest_params);

    if !recover(&struct_hash, &params.signature, &params.public_key)? {
        return Err(ContractError::InvalidSignature {});
    }

    NONCES.save(deps.storage, &params.staker, &(nonce + 1))?;

    deposit_into_strategy_internal(
        deps,
        info,
        params.staker.clone(),
        params.strategy.clone(),
        params.token.clone(),
        params.amount,
    )
}

pub fn add_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    token: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_delegation_manager(deps.as_ref(), &info)?;

    add_shares_internal(deps, staker, token, strategy, shares)
}

pub fn remove_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    only_delegation_manager(deps.as_ref(), &info)?;
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
    token: Addr,
) -> Result<Response, ContractError> {
    only_delegation_manager(deps.as_ref(), &info)?;

    let withdraw_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::Withdraw {
            recipient: recipient.to_string(),
            token: token.to_string(),
            amount_shares: shares,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_message(withdraw_msg);

    Ok(response)
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
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
        QueryMsg::IsThirdPartyTransfersForbidden { strategy } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            to_json_binary(&query_is_third_party_transfers_forbidden(
                deps,
                strategy_addr,
            )?)
        }
        QueryMsg::GetNonce { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            to_json_binary(&query_nonce(deps, staker_addr)?)
        }
        QueryMsg::GetStakerStrategyList { staker } => {
            let staker_addr = deps.api.addr_validate(&staker)?;

            to_json_binary(&query_staker_strategy_list(deps, staker_addr)?)
        }
        QueryMsg::Owner {} => to_json_binary(&query_owner(deps)?),
        QueryMsg::IsStrategyWhitelisted { strategy } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;

            to_json_binary(&query_is_strategy_whitelisted(deps, strategy_addr)?)
        }
        QueryMsg::CalculateDigestHash { digest_hash_params } => {
            let response = query_calculate_digest_hash(deps, digest_hash_params)?;
            to_json_binary(&response)
        }
        QueryMsg::GetStrategyWhitelister {} => to_json_binary(&query_strategy_whitelister(deps)?),
        QueryMsg::GetStrategyManagerState {} => {
            to_json_binary(&query_strategy_manager_state(deps)?)
        }
        QueryMsg::GetDepositTypeHash {} => to_json_binary(&query_deposit_type_hash()?),
        QueryMsg::DomainTypeHash {} => to_json_binary(&query_domain_type_hash()?),
        QueryMsg::DomainName {} => to_json_binary(&query_domain_name()?),
        QueryMsg::DelegationManager {} => to_json_binary(&query_delegation_manager(deps)?),
    }
}

fn query_calculate_digest_hash(
    deps: Deps,
    digest_hash_params: QueryDigestHashParams,
) -> StdResult<CalculateDigestHashResponse> {
    let staker_addr = deps.api.addr_validate(&digest_hash_params.staker)?;
    let strategy_addr = deps.api.addr_validate(&digest_hash_params.strategy)?;
    let token_addr = deps.api.addr_validate(&digest_hash_params.token)?;
    let contract_addr = Addr::unchecked(&digest_hash_params.contract_addr);

    let public_key_binary = Binary::from_base64(&digest_hash_params.public_key)?;

    let params = DigestHashParams {
        staker: staker_addr,
        public_key: public_key_binary,
        strategy: strategy_addr,
        token: token_addr,
        amount: digest_hash_params.amount,
        nonce: digest_hash_params.nonce,
        expiry: digest_hash_params.expiry,
        chain_id: digest_hash_params.chain_id,
        contract_addr,
    };

    let digest_hash = calculate_digest_hash(&params);
    let digest_hash = Binary::new(digest_hash);
    Ok(CalculateDigestHashResponse { digest_hash })
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

fn query_is_third_party_transfers_forbidden(
    deps: Deps,
    strategy: Addr,
) -> StdResult<ThirdPartyTransfersForbiddenResponse> {
    let is_forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN
        .may_load(deps.storage, &strategy)?
        .unwrap_or(false);
    Ok(ThirdPartyTransfersForbiddenResponse { is_forbidden })
}

fn query_nonce(deps: Deps, staker: Addr) -> StdResult<NonceResponse> {
    let nonce = NONCES.may_load(deps.storage, &staker)?.unwrap_or(0);
    Ok(NonceResponse { nonce })
}

fn query_staker_strategy_list(deps: Deps, staker: Addr) -> StdResult<StakerStrategyListResponse> {
    let strategies = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);
    Ok(StakerStrategyListResponse { strategies })
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let owner_addr = OWNER.load(deps.storage)?;
    Ok(OwnerResponse { owner_addr })
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

fn query_strategy_whitelister(deps: Deps) -> StdResult<StrategyWhitelisterResponse> {
    let whitelister = STRATEGY_WHITELISTER.load(deps.storage)?;
    Ok(StrategyWhitelisterResponse { whitelister })
}

fn query_strategy_manager_state(deps: Deps) -> StdResult<StrategyManagerStateResponse> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    Ok(StrategyManagerStateResponse { state })
}

fn query_deposit_type_hash() -> StdResult<DepositTypeHashResponse> {
    let deposit_type_hash = String::from_utf8_lossy(DEPOSIT_TYPEHASH).to_string();
    Ok(DepositTypeHashResponse { deposit_type_hash })
}

fn query_domain_type_hash() -> StdResult<DomainTypeHashResponse> {
    let domain_type_hash = String::from_utf8_lossy(DOMAIN_TYPEHASH).to_string();
    Ok(DomainTypeHashResponse { domain_type_hash })
}

fn query_domain_name() -> StdResult<DomainNameResponse> {
    let domain_name = String::from_utf8_lossy(DOMAIN_NAME).to_string();
    Ok(DomainNameResponse { domain_name })
}

fn query_delegation_manager(deps: Deps) -> StdResult<DelegationManagerResponse> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;
    Ok(DelegationManagerResponse {
        delegation_manager: state.delegation_manager,
    })
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

fn only_strategy_whitelister(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let whitelister: Addr = STRATEGY_WHITELISTER.load(deps.storage)?;
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    if info.sender != whitelister && info.sender != state.strategy_factory {
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
    if info.sender != state.delegation_manager && info.sender != state.slash_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
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

    let state: StrategyState = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyQueryMsg::GetStrategyState {})?,
    }))?;

    let balance = token_balance(&deps.querier, &state.underlying_token, &strategy)?;
    let new_shares = calculate_new_shares(state.total_shares, balance, amount)?;

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    let deposit_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::Deposit { amount })?,
        funds: vec![],
    });

    response = response.add_message(deposit_msg);

    add_shares_internal(
        deps.branch(),
        staker.clone(),
        token.clone(),
        strategy.clone(),
        new_shares,
    )?;

    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    let increase_delegated_shares_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.delegation_manager.to_string(),
        msg: to_json_binary(&DelegationManagerExecuteMsg::IncreaseDelegatedShares {
            staker: staker.to_string(),
            strategy: strategy.to_string(),
            shares: new_shares,
        })?,
        funds: vec![],
    });

    Ok(response
        .add_message(increase_delegated_shares_msg)
        .add_attribute("new_shares", new_shares.to_string()))
}

fn add_shares_internal(
    deps: DepsMut,
    staker: Addr,
    token: Addr,
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
        .add_attribute("token", token.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use bech32::{self, ToBase32, Variant};
    use bvs_base::roles::{PAUSER, UNPAUSER};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, from_json, Addr, ContractResult, OwnedDeps, SystemError, SystemResult,
    };
    use cw2::get_contract_version;
    use ripemd::Ripemd160;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner = deps.api.addr_make("owner").to_string();
        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
        let slasher = deps.api.addr_make("slasher").to_string();
        let strategy_factory = deps.api.addr_make("strategy_factory").to_string();
        let strategy_whitelister = deps.api.addr_make("strategy_whitelister").to_string();
        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner.clone(),
            delegation_manager: delegation_manager.clone(),
            slash_manager: slasher.clone(),
            strategy_factory: strategy_factory.clone(),
            initial_strategy_whitelister: strategy_whitelister.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "delegation_manager");
        assert_eq!(res.attributes[1].value, delegation_manager.clone());
        assert_eq!(res.attributes[2].key, "slasher");
        assert_eq!(res.attributes[2].value, slasher.clone());
        assert_eq!(res.attributes[3].key, "strategy_whitelister");
        assert_eq!(res.attributes[3].value, strategy_whitelister.clone());
        assert_eq!(res.attributes[4].key, "owner");
        assert_eq!(res.attributes[4].value, owner.clone());

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, owner.clone());

        let strategy_manager_state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(
            strategy_manager_state.delegation_manager,
            Addr::unchecked(delegation_manager.clone())
        );
        assert_eq!(
            strategy_manager_state.slash_manager,
            Addr::unchecked(slasher.clone())
        );

        let strategy_whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        assert_eq!(
            strategy_whitelister,
            Addr::unchecked(strategy_whitelister.clone())
        );
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner").to_string();
        let owner_info = message_info(&Addr::unchecked(owner.clone()), &[]);

        let delegation_manager = deps.api.addr_make("delegation_manager").to_string();
        let slasher = deps.api.addr_make("slasher").to_string();
        let strategy_factory = deps.api.addr_make("strategy_factory").to_string();
        let strategy_whitelister = deps.api.addr_make("strategy_whitelister").to_string();

        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();

        let pauser_info = message_info(&Addr::unchecked(pauser.clone()), &[]);
        let unpauser_info = message_info(&Addr::unchecked(unpauser.clone()), &[]);
        let strategy_whitelister_info =
            message_info(&Addr::unchecked(strategy_whitelister.clone()), &[]);
        let delegation_manager_info =
            message_info(&Addr::unchecked(delegation_manager.clone()), &[]);

        let msg = InstantiateMsg {
            initial_owner: owner.clone(),
            delegation_manager: delegation_manager.clone(),
            slash_manager: slasher.clone(),
            strategy_factory: strategy_factory.clone(),
            initial_strategy_whitelister: strategy_whitelister.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
        };

        let _res = instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        (
            deps,
            env,
            owner_info,
            delegation_manager_info,
            strategy_whitelister_info,
            pauser_info,
            unpauser_info,
        )
    }

    #[test]
    fn test_only_strategy_whitelister() {
        let (
            deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = only_strategy_whitelister(deps.as_ref(), &info_whitelister);
        assert!(result.is_ok());

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
        let (
            deps,
            _env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = only_owner(deps.as_ref(), &owner_info);
        assert!(result.is_ok());

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
        let (
            deps,
            _env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = only_delegation_manager(deps.as_ref(), &info_delegation_manager);
        assert!(result.is_ok());

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
        let (
            mut deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategies = vec![
            deps.api.addr_make("strategy1").to_string(),
            deps.api.addr_make("strategy2").to_string(),
        ];

        let forbidden_values = vec![true, false];
        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();

        let events = res.events;

        assert_eq!(events.len(), strategies.len());

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.ty, "StrategyAddedToDepositWhitelist");
            assert_eq!(event.attributes.len(), 2);
            assert_eq!(event.attributes[0].key, "strategy");
            assert_eq!(event.attributes[0].value, strategies[i]);
            assert_eq!(event.attributes[1].key, "third_party_transfers_forbidden");
            assert_eq!(event.attributes[1].value, forbidden_values[i].to_string());
        }

        for (i, strategy) in strategies.iter().enumerate() {
            let is_whitelisted = STRATEGY_IS_WHITELISTED_FOR_DEPOSIT
                .load(&deps.storage, &Addr::unchecked(strategy.clone()))
                .unwrap();
            assert!(is_whitelisted);

            let forbidden = THIRD_PARTY_TRANSFERS_FORBIDDEN
                .load(&deps.storage, &Addr::unchecked(strategy.clone()))
                .unwrap();
            assert_eq!(forbidden, forbidden_values[i]);
        }

        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: strategies.clone(),
            third_party_transfers_forbidden_values: forbidden_values.clone(),
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
        let result = execute(deps.as_mut(), env.clone(), info_unauthorized.clone(), msg);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test with mismatched strategies and forbidden_values length
        let strategies = vec![deps.api.addr_make("strategy3").to_string()];
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
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategies = vec![
            deps.api.addr_make("strategy1").to_string(),
            deps.api.addr_make("strategy2").to_string(),
        ];
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

        // Test with an unauthorized user
        let msg = ExecuteMsg::RemoveStrategiesFromWhitelist {
            strategies: strategies.clone(),
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);
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
        let (
            mut deps,
            _env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let old_whitelister = STRATEGY_WHITELISTER.load(&deps.storage).unwrap();
        let new_whitelister = Addr::unchecked("new_whitelister");

        let res =
            set_strategy_whitelister(deps.as_mut(), owner_info.clone(), new_whitelister.clone())
                .unwrap();

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

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

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
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1").to_string();
        let value = true;

        let msg = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value,
        };

        let res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "set_third_party_transfers_forbidden");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "strategy");
        assert_eq!(event.attributes[0].value, strategy.to_string());
        assert_eq!(event.attributes[1].key, "value");
        assert_eq!(event.attributes[1].value, value.to_string());

        let stored_value = THIRD_PARTY_TRANSFERS_FORBIDDEN
            .load(&deps.storage, &Addr::unchecked(strategy.clone()))
            .unwrap();
        assert_eq!(stored_value, value);

        // Test with an unauthorized user
        let exec_msg_unauthorized = ExecuteMsg::SetThirdPartyTransfersForbidden {
            strategy: strategy.clone(),
            value,
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = execute(
            deps.as_mut(),
            env.clone(),
            info_unauthorized.clone(),
            exec_msg_unauthorized,
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
    fn test_deposit_into_strategy() {
        let (
            mut deps,
            env,
            _owner_info,
            info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1").to_string();
        let token = deps.api.addr_make("token").to_string();
        let amount = Uint128::new(100);

        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.clone()],
            third_party_transfers_forbidden_values: vec![false],
        };

        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();

        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();
        let delegation_manager_sender = info_delegation_manager.sender.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == strategy_for_closure => {
                let strategy_query_msg: StrategyQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    StrategyQueryMsg::GetStrategyState {} => {
                        let strategy_state = StrategyState {
                            strategy_manager: delegation_manager_sender.clone(),
                            underlying_token: Addr::unchecked(token_for_closure.clone()),
                            total_shares: Uint128::new(1000),
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&strategy_state).unwrap(),
                        ))
                    }
                }
            }
            WasmQuery::Smart { contract_addr, msg } if *contract_addr == token_for_closure => {
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

        let msg = ExecuteMsg::DepositIntoStrategy {
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
        };

        let res: Response = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0].key, "new_shares");
        assert_eq!(res.attributes[0].value, "100");

        // Test deposit into strategy with non-whitelisted strategy
        let non_whitelisted_strategy = deps.api.addr_make("non_whitelisted_strategy").to_string();

        let msg = ExecuteMsg::DepositIntoStrategy {
            strategy: non_whitelisted_strategy.clone(),
            token: token.clone(),
            amount,
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
                ContractError::StrategyNotWhitelisted {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_get_deposits() {
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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
        let (
            mut deps,
            _env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let token = Addr::unchecked("token");
        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);

        let res = add_shares_internal(
            deps.as_mut(),
            staker.clone(),
            token.clone(),
            strategy.clone(),
            shares,
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

        let additional_shares = Uint128::new(50);
        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            token.clone(),
            strategy.clone(),
            additional_shares,
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

        // Test with zero shares
        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            token.clone(),
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
        let result = add_shares_internal(
            deps.as_mut(),
            staker.clone(),
            token.clone(),
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
    fn test_add_shares() {
        let (
            mut deps,
            env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let token = deps.api.addr_make("token");
        let staker = deps.api.addr_make("staker");
        let strategy = deps.api.addr_make("strategy");
        let shares = Uint128::new(100);

        let msg = ExecuteMsg::AddShares {
            staker: staker.to_string(),
            token: token.to_string(),
            strategy: strategy.to_string(),
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
            staker: staker.to_string(),
            token: token.to_string(),
            strategy: strategy.to_string(),
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
            staker: staker.to_string(),
            token: token.to_string(),
            strategy: strategy.to_string(),
            shares,
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

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
            staker: staker.to_string(),
            token: token.to_string(),
            strategy: strategy.to_string(),
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

        let new_strategy = deps.api.addr_make("new_strategy");

        let exec_msg = ExecuteMsg::AddShares {
            staker: staker.to_string(),
            token: token.to_string(),
            strategy: new_strategy.to_string(),
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
        let (
            mut deps,
            env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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

        let msg = ExecuteMsg::RemoveShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
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

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy1))
            .unwrap();
        println!("Stored shares after removal: {}", stored_shares);
        assert_eq!(stored_shares, Uint128::new(50));

        // Test removing shares with an unauthorized user
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.to_string(),
            strategy: strategy2.to_string(),
            shares: Uint128::new(50),
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = execute(deps.as_mut(), env.clone(), info_unauthorized, msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test removing more shares than available
        let msg = ExecuteMsg::RemoveShares {
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
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
            staker: staker.to_string(),
            strategy: strategy1.to_string(),
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
        let (
            mut deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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

    fn generate_osmosis_public_key_from_private_key(
        private_key_hex: &str,
    ) -> (Addr, SecretKey, Vec<u8>) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(private_key_hex).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize();
        let sha256_result = Sha256::digest(public_key_bytes);
        let ripemd160_result = Ripemd160::digest(sha256_result);
        let address =
            bech32::encode("osmo", ripemd160_result.to_base32(), Variant::Bech32).unwrap();
        (
            Addr::unchecked(address),
            secret_key,
            public_key_bytes.to_vec(),
        )
    }

    fn mock_signature_with_message(params: DigestHashParams, secret_key: &SecretKey) -> Binary {
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
        let (
            mut deps,
            env,
            _owner_info,
            info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy");
        let token = deps.api.addr_make("token");
        let amount = Uint128::new(100);

        let msg = ExecuteMsg::AddStrategiesToWhitelist {
            strategies: vec![strategy.to_string()],
            third_party_transfers_forbidden_values: vec![false],
        };

        let _res = execute(deps.as_mut(), env.clone(), info_whitelister.clone(), msg).unwrap();

        let private_key_hex = "3556b8af0d03b26190927a3aec5b72d9c1810e97cd6430cefb65734eb9c804aa";
        let (staker, secret_key, public_key_bytes) =
            generate_osmosis_public_key_from_private_key(private_key_hex);
        let current_time = env.block.time.seconds();
        let expiry = current_time + 1000;
        let nonce = 0;
        let chain_id = env.block.chain_id.clone();
        let contract_addr = env.contract.address.clone();

        let public_key = Binary::from(public_key_bytes);
        let public_key_base64 = general_purpose::STANDARD.encode(public_key.clone());

        let params = DigestHashParams {
            staker: staker.clone(),
            public_key,
            strategy: strategy.clone(),
            token: token.clone(),
            amount,
            nonce,
            expiry,
            chain_id: chain_id.to_string(),
            contract_addr: contract_addr.clone(),
        };

        let signature = mock_signature_with_message(params, &secret_key);
        let signature_base64 = general_purpose::STANDARD.encode(signature);

        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == strategy_for_closure.to_string() =>
            {
                let strategy_query_msg: StrategyQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    StrategyQueryMsg::GetStrategyState {} => {
                        let strategy_state = StrategyState {
                            strategy_manager: Addr::unchecked("delegation_manager"),
                            underlying_token: token_for_closure.clone(),
                            total_shares: Uint128::new(1000),
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&strategy_state).unwrap(),
                        ))
                    }
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

        let msg = ExecuteMsg::DepositIntoStrategyWithSignature {
            strategy: strategy.to_string(),
            token: token.to_string(),
            amount,
            staker: staker.to_string(),
            public_key: public_key_base64,
            expiry,
            signature: signature_base64,
        };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_delegation_manager.clone(),
            msg,
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0].key, "new_shares");
        assert_eq!(res.attributes[0].value, "100");

        let stored_nonce = NONCES.load(&deps.storage, &staker).unwrap();
        assert_eq!(stored_nonce, 1);
    }

    #[test]
    fn test_is_third_party_transfers_forbidden() {
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1");
        THIRD_PARTY_TRANSFERS_FORBIDDEN
            .save(&mut deps.storage, &strategy, &true)
            .unwrap();

        let query_msg = QueryMsg::IsThirdPartyTransfersForbidden {
            strategy: strategy.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let result: ThirdPartyTransfersForbiddenResponse = from_json(bin).unwrap();
        assert!(result.is_forbidden);

        let non_forbidden_strategy = deps.api.addr_make("non_forbidden_strategy");

        let query_msg = QueryMsg::IsThirdPartyTransfersForbidden {
            strategy: non_forbidden_strategy.to_string(),
        };
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let result: ThirdPartyTransfersForbiddenResponse = from_json(bin).unwrap();
        assert!(!result.is_forbidden);
    }

    #[test]
    fn test_get_nonce() {
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");

        NONCES.save(&mut deps.storage, &staker, &5).unwrap();

        let query_msg = QueryMsg::GetNonce {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let nonce_response: NonceResponse = from_json(bin).unwrap();
        assert_eq!(nonce_response.nonce, 5);

        let new_staker = deps.api.addr_make("new_staker");

        let query_msg = QueryMsg::GetNonce {
            staker: new_staker.to_string(),
        };
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let nonce_response: NonceResponse = from_json(bin).unwrap();
        assert_eq!(nonce_response.nonce, 0);
    }

    #[test]
    fn test_get_staker_strategy_list() {
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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
    fn test_get_owner() {
        let (
            deps,
            env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let query_msg = QueryMsg::Owner {};
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let owner_response: OwnerResponse = from_json(bin).unwrap();

        assert_eq!(owner_response.owner_addr, owner_info.sender);
    }

    #[test]
    fn test_is_strategy_whitelisted() {
        let (
            mut deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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
    fn test_get_strategy_whitelister() {
        let (
            deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let response = query_strategy_whitelister(deps.as_ref()).unwrap();
        assert_eq!(response.whitelister, info_whitelister.sender);
    }

    #[test]
    fn test_get_strategy_manager_state() {
        let (
            deps,
            _env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let state = query_strategy_manager_state(deps.as_ref()).unwrap();
        assert_eq!(
            state.state.delegation_manager,
            info_delegation_manager.sender
        );
    }

    #[test]
    fn test_get_deposit_type_hash() {
        let type_hash = query_deposit_type_hash().unwrap();
        let expected_str = String::from_utf8_lossy(DEPOSIT_TYPEHASH).to_string();
        assert_eq!(type_hash.deposit_type_hash, expected_str);
    }

    #[test]
    fn test_get_domain_type_hash() {
        let type_hash = query_domain_type_hash().unwrap();
        let expected_str = String::from_utf8_lossy(DOMAIN_TYPEHASH).to_string();
        assert_eq!(type_hash.domain_type_hash, expected_str);
    }

    #[test]
    fn test_get_domain_name() {
        let name = query_domain_name().unwrap();
        let expected_str = String::from_utf8_lossy(DOMAIN_NAME).to_string();
        assert_eq!(name.domain_name, expected_str);
    }

    #[test]
    fn test_get_staker_strategy_shares() {
        let (
            mut deps,
            _env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

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

    #[test]
    fn test_get_delegation_manager() {
        let (
            deps,
            env,
            _owner_info,
            info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let query_msg = QueryMsg::DelegationManager {};
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let delegation_manager: DelegationManagerResponse = from_json(bin).unwrap();

        assert_eq!(
            delegation_manager.delegation_manager,
            info_delegation_manager.sender
        );
    }

    #[test]
    fn test_set_delegation_manager() {
        let (
            mut deps,
            env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let new_delegation_manager = deps.api.addr_make("new_delegation_manager");

        let msg = ExecuteMsg::SetDelegationManager {
            new_delegation_manager: new_delegation_manager.to_string(),
        };

        execute(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        let state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.delegation_manager, new_delegation_manager);

        // Test with an unauthorized user
        let msg = ExecuteMsg::SetDelegationManager {
            new_delegation_manager: new_delegation_manager.to_string(),
        };

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = execute(deps.as_mut(), env.clone(), info_unauthorized, msg);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_set_slash_manager() {
        let (
            mut deps,
            env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let new_slash_manager = deps.api.addr_make("new_slash_manager");

        let msg = ExecuteMsg::SetSlashManager {
            new_slash_manager: new_slash_manager.to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "SlashManagerSet");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "method");
        assert_eq!(event.attributes[0].value, "set_slash_manager");
        assert_eq!(event.attributes[1].key, "new_slash_manager");
        assert_eq!(event.attributes[1].value, new_slash_manager.to_string());

        let state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.slash_manager, new_slash_manager);

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let msg = ExecuteMsg::SetSlashManager {
            new_slash_manager: new_slash_manager.to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), info_unauthorized, msg);

        assert!(res.is_err());
        if let Err(err) = res {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let state = STRATEGY_MANAGER_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.slash_manager, new_slash_manager);
    }

    #[test]
    fn test_transfer_ownership() {
        let (
            mut deps,
            env,
            owner_info,
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
        ) = instantiate_contract();

        let new_owner = deps.api.addr_make("new_owner");

        let transfer_msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), transfer_msg);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "transfer_ownership");
        assert_eq!(res.attributes[1].key, "new_owner");
        assert_eq!(res.attributes[1].value, new_owner.to_string());

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let transfer_msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.to_string(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info_unauthorized.clone(),
            transfer_msg,
        );

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
            new_owner: new_owner.to_string(),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            message_info(&new_owner, &[]),
            transfer_msg,
        );
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "transfer_ownership");
        assert_eq!(res.attributes[1].key, "new_owner");
        assert_eq!(res.attributes[1].value, new_owner.to_string());

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner);
    }

    #[test]
    fn test_pause() {
        let (
            mut deps,
            env,
            _owner_info,
            _info_delegation_manager,
            _info_whitelister,
            pauser_info,
            _unpauser_info,
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
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            unpauser_info,
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
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
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
            _info_delegation_manager,
            _info_whitelister,
            _pauser_info,
            _unpauser_info,
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
