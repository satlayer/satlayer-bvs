use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{
        ExplanationResponse, SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse,
        TotalSharesResponse, UnderlyingToShareResponse, UnderlyingToSharesResponse,
        UnderlyingTokenResponse, UserUnderlyingResponse,
    },
    state::{StrategyState, OWNER, STRATEGY_STATE},
};
use common::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use common::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};
use common::strategy::{QueryMsg as StrategyManagerQueryMsg, StakerStrategySharesResponse};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAUSED_DEPOSITS: u8 = 0;
const PAUSED_WITHDRAWALS: u8 = 1;

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

    let owner = deps.api.addr_validate(&msg.initial_owner)?;

    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;
    let underlying_token = deps.api.addr_validate(&msg.underlying_token)?;

    OWNER.save(deps.storage, &owner)?;

    let state = StrategyState {
        strategy_manager: strategy_manager.clone(),
        underlying_token: underlying_token.clone(),
        total_shares: Uint128::zero(),
    };

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    STRATEGY_STATE.save(deps.storage, &state)?;
    PAUSED_STATE.save(deps.storage, &msg.initial_paused_status)?;

    let underlying_token = msg.underlying_token.clone();

    let token_info: cw20::TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: underlying_token.to_string(),
            msg: to_json_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    let decimals = token_info.decimals;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("underlying_token", state.underlying_token.to_string())
        .add_attribute("underlying_token_decimals", decimals.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { amount } => deposit(deps, env, info, amount),
        ExecuteMsg::Withdraw {
            recipient,
            token,
            amount_shares,
        } => {
            let recipient_addr = deps.api.addr_validate(&recipient)?;
            let token_addr = deps.api.addr_validate(&token)?;
            withdraw(deps, env, info, recipient_addr, token_addr, amount_shares)
        }
        ExecuteMsg::SetStrategyManager { new_strategy_manager } => {
            let new_strategy_manager_addr = deps.api.addr_validate(&new_strategy_manager)?;
            set_strategy_manager(deps, info, new_strategy_manager_addr)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
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

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_DEPOSITS)?;

    let mut state = STRATEGY_STATE.load(deps.storage)?;

    only_strategy_manager(deps.as_ref(), &info)?;
    before_deposit(&state, &state.underlying_token)?;

    let balance = token_balance(
        &deps.querier,
        &state.underlying_token,
        &env.contract.address,
    )?;

    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let new_shares = (amount * virtual_share_amount) / virtual_prior_token_balance;

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    state.total_shares += new_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    let exchange_rate_event =
        emit_exchange_rate(virtual_token_balance, state.total_shares + SHARES_OFFSET)?;

    let response = Response::new()
        .add_attribute("method", "deposit")
        .add_attribute("new_shares", new_shares.to_string())
        .add_attribute("total_shares", state.total_shares.to_string())
        .add_event(exchange_rate_event.events[0].clone());

    Ok(response)
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    token: Addr,
    amount_shares: Uint128,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_WITHDRAWALS)?;

    let mut state = STRATEGY_STATE.load(deps.storage)?;

    only_strategy_manager(deps.as_ref(), &info)?;
    before_withdrawal(&state, &token)?;

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let balance = token_balance(
        &deps.querier,
        &state.underlying_token,
        &env.contract.address,
    )?;

    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;

    if amount_to_send.is_zero() {
        return Err(ContractError::ZeroAmountToSend {});
    }

    if amount_to_send > balance {
        return Err(ContractError::InsufficientBalance {});
    }

    state.total_shares -= amount_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    let exchange_rate_event = emit_exchange_rate(
        virtual_token_balance - amount_to_send,
        state.total_shares + SHARES_OFFSET,
    )?;

    let underlying_token = state.underlying_token;

    let transfer_msg = WasmMsg::Execute {
        contract_addr: underlying_token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: amount_to_send,
        })?,
        funds: vec![],
    };

    let transfer_cosmos_msg: CosmosMsg = transfer_msg.into();

    let response = Response::new().add_message(transfer_cosmos_msg);

    Ok(response
        .add_attribute("method", "withdraw")
        .add_attribute("amount_to_send", amount_to_send.to_string())
        .add_attribute("total_shares", state.total_shares.to_string())
        .add_event(exchange_rate_event.events[0].clone()))
}

pub fn shares(deps: Deps, user: Addr, strategy: Addr) -> StdResult<SharesResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;

    let response: StakerStrategySharesResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: state.strategy_manager.to_string(),
            msg: to_json_binary(&StrategyManagerQueryMsg::GetStakerStrategyShares {
                staker: user.to_string(),
                strategy: strategy.to_string(),
            })?,
        }))?;

    Ok(SharesResponse {
        total_shares: response.shares,
    })
}

pub fn shares_to_underlying_view(
    deps: Deps,
    env: Env,
    amount_shares: Uint128,
) -> StdResult<Uint128> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    let balance = token_balance(
        &deps.querier,
        &state.underlying_token,
        &env.contract.address,
    )?;

    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;

    Ok(amount_to_send)
}

pub fn underlying_to_share_view(deps: Deps, env: Env, amount: Uint128) -> StdResult<Uint128> {
    let state: StrategyState = STRATEGY_STATE.load(deps.storage)?;
    let balance = token_balance(
        &deps.querier,
        &state.underlying_token,
        &env.contract.address,
    )?;

    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let share_to_send = (amount * virtual_share_amount) / virtual_prior_token_balance;

    Ok(share_to_send)
}

pub fn underlying_to_shares(
    deps: Deps,
    env: Env,
    amount_underlying: Uint128,
) -> StdResult<Uint128> {
    let share_to_send = underlying_to_share_view(deps, env, amount_underlying)?;
    Ok(share_to_send)
}

pub fn user_underlying_view(deps: Deps, env: Env, user: Addr) -> StdResult<Uint128> {
    let strategy = env.contract.address.clone();
    let shares_response = shares(deps, user, strategy.clone())?;
    let user_shares = shares_response.total_shares;

    let amount_to_send = shares_to_underlying_view(deps, env, user_shares)?;

    Ok(amount_to_send)
}

pub fn set_strategy_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut state = STRATEGY_STATE.load(deps.storage)?;
    state.strategy_manager = new_strategy_manager.clone();
    STRATEGY_STATE.save(deps.storage, &state)?;

    let event = Event::new("strategy_manager_set")
        .add_attribute("method", "set_strategy_manager")
        .add_attribute("new_strategy_manager", new_strategy_manager.to_string());

    Ok(Response::new().add_event(event))
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetShares { staker, strategy } => {
            let staker_addr = deps.api.addr_validate(&staker)?;
            let strategy_addr = Addr::unchecked(strategy);

            to_json_binary(&shares(deps, staker_addr, strategy_addr)?)
        }
        QueryMsg::SharesToUnderlyingView { amount_shares } => {
            to_json_binary(&query_shares_to_underlying_view(deps, env, amount_shares)?)
        }
        QueryMsg::UnderlyingToShareView { amount } => {
            to_json_binary(&query_underlying_to_view(deps, env, amount)?)
        }
        QueryMsg::UserUnderlyingView { user } => {
            let user_addr = deps.api.addr_validate(&user)?;
            to_json_binary(&query_user_underlying_view(deps, env, user_addr)?)
        }
        QueryMsg::GetStrategyManager {} => to_json_binary(&query_strategy_manager(deps)?),
        QueryMsg::GetUnderlyingToken {} => to_json_binary(&query_underlying_token(deps)?),
        QueryMsg::GetTotalShares {} => to_json_binary(&query_total_shares(deps)?),
        QueryMsg::Explanation {} => to_json_binary(&query_explanation()?),
        QueryMsg::UnderlyingToShares { amount_underlying } => {
            to_json_binary(&query_underlying_to_shares(deps, env, amount_underlying)?)
        }
        QueryMsg::GetStrategyState {} => to_json_binary(&query_strategy_state(deps)?),
    }
}

pub fn query_strategy_manager(deps: Deps) -> StdResult<StrategyManagerResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(StrategyManagerResponse {
        strate_manager_addr: state.strategy_manager,
    })
}

fn query_underlying_token(deps: Deps) -> StdResult<UnderlyingTokenResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(UnderlyingTokenResponse {
        underlying_token_addr: state.underlying_token,
    })
}

fn query_total_shares(deps: Deps) -> StdResult<TotalSharesResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(TotalSharesResponse {
        total_shares: state.total_shares,
    })
}

fn query_explanation() -> StdResult<ExplanationResponse> {
    Ok(ExplanationResponse {
        explanation:
            "Base Strategy implementation to inherit from for more complex implementations"
                .to_string(),
    })
}

pub fn query_strategy_state(deps: Deps) -> StdResult<StrategyState> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(state)
}

pub fn query_shares_to_underlying_view(
    deps: Deps,
    env: Env,
    amount_shares: Uint128,
) -> StdResult<SharesToUnderlyingResponse> {
    let amount_to_send = shares_to_underlying_view(deps, env, amount_shares)?;

    Ok(SharesToUnderlyingResponse { amount_to_send })
}

pub fn query_underlying_to_view(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> StdResult<UnderlyingToShareResponse> {
    let share_to_send = underlying_to_share_view(deps, env, amount)?;

    Ok(UnderlyingToShareResponse { share_to_send })
}

pub fn query_user_underlying_view(
    deps: Deps,
    env: Env,
    user: Addr,
) -> StdResult<UserUnderlyingResponse> {
    let amount_to_send = user_underlying_view(deps, env, user)?;
    Ok(UserUnderlyingResponse { amount_to_send })
}

pub fn query_underlying_to_shares(
    deps: Deps,
    env: Env,
    amount_underlying: Uint128,
) -> StdResult<UnderlyingToSharesResponse> {
    let share_to_send = underlying_to_shares(deps, env, amount_underlying)?;
    Ok(UnderlyingToSharesResponse { share_to_send })
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn only_strategy_manager(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = STRATEGY_STATE.load(deps.storage)?;

    if info.sender != state.strategy_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn before_deposit(state: &StrategyState, token: &Addr) -> Result<(), ContractError> {
    if token != state.underlying_token {
        return Err(ContractError::InvalidToken {});
    }
    Ok(())
}

fn before_withdrawal(state: &StrategyState, token: &Addr) -> Result<(), ContractError> {
    if token != state.underlying_token {
        return Err(ContractError::InvalidToken {});
    }
    Ok(())
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

fn emit_exchange_rate(
    virtual_token_balance: Uint128,
    virtual_total_shares: Uint128,
) -> StdResult<Response> {
    let exchange_rate = (virtual_token_balance.checked_mul(Uint128::new(1_000_000))?)
        .checked_div(virtual_total_shares)?;

    let event = Event::new("exchange_rate_emitted")
        .add_attribute("exchange_rate", exchange_rate.to_string());

    Ok(Response::new().add_event(event))
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::roles::{PAUSER, UNPAUSER};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, from_json, Binary, ContractResult, CosmosMsg, OwnedDeps, SystemError, SystemResult,
        WasmQuery,
    };
    use cw2::get_contract_version;
    use cw20::TokenInfoResponse;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner = deps.api.addr_make("owner").to_string();

        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();

        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let token = deps.api.addr_make("token").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner.clone(),
            strategy_manager: strategy_manager.clone(),
            underlying_token: token.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
        };

        deps.querier.update_wasm({
            let token_clone = token.clone();
            move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    if contract_addr == &token_clone {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::TokenInfo {} = msg {
                            return SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&TokenInfoResponse {
                                    name: "Mock Token".to_string(),
                                    symbol: "MTK".to_string(),
                                    decimals: 8,
                                    total_supply: Uint128::new(1_000_000),
                                })
                                .unwrap(),
                            ));
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "strategy_manager");
        assert_eq!(res.attributes[1].value, strategy_manager);
        assert_eq!(res.attributes[2].key, "underlying_token");
        assert_eq!(res.attributes[2].value, token);
        assert_eq!(res.attributes[3].key, "underlying_token_decimals");
        assert_eq!(res.attributes[3].value, "8");
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        MessageInfo,
        MessageInfo,
        String,
        String,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner").to_string();
        let owner_info = message_info(&Addr::unchecked(owner.clone()), &[]);

        let pauser = deps.api.addr_make("pauser").to_string();
        let unpauser = deps.api.addr_make("unpauser").to_string();

        let pauser_info = message_info(&Addr::unchecked(pauser.clone()), &[]);
        let unpauser_info = message_info(&Addr::unchecked(unpauser.clone()), &[]);

        let strategy_manager = deps.api.addr_make("strategy_manager").to_string();
        let token = deps.api.addr_make("token").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner.clone(),
            strategy_manager: strategy_manager.clone(),
            underlying_token: token.clone(),
            pauser: pauser.clone(),
            unpauser: unpauser.clone(),
            initial_paused_status: 0,
        };

        deps.querier.update_wasm({
            let token_clone = token.clone();
            move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    if contract_addr == &token_clone {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        match msg {
                            Cw20QueryMsg::TokenInfo {} => {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&TokenInfoResponse {
                                        name: "Mock Token".to_string(),
                                        symbol: "MTK".to_string(),
                                        decimals: 8,
                                        total_supply: Uint128::new(1_000_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                            Cw20QueryMsg::Balance { address: _ } => {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&Cw20BalanceResponse {
                                        balance: Uint128::new(1_000_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                            _ => {}
                        }
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: "not implemented".to_string(),
                            request: to_json_binary(&msg).unwrap(),
                        })
                    } else {
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: "not implemented".to_string(),
                            request: to_json_binary(&msg).unwrap(),
                        })
                    }
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let _res = instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        (
            deps,
            env,
            owner_info,
            pauser_info,
            unpauser_info,
            token,
            strategy_manager,
        )
    }

    #[test]
    fn test_deposit() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let amount = Uint128::new(1_000);
        let msg = ExecuteMsg::Deposit { amount };

        let info = message_info(&Addr::unchecked(strategy_manager), &[]);

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit");
        assert!(res.attributes[1].key == "new_shares");
        assert!(res.attributes[2].key == "total_shares");

        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        assert_eq!(event.ty, "exchange_rate_emitted");
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(event.attributes[0].key, "exchange_rate");
        assert_eq!(event.attributes[0].value, "1000000");

        let exchange_rate = event.attributes[0].value.parse::<u128>().unwrap();
        assert!(exchange_rate > 0, "Exchange rate should be positive");

        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert!(state.total_shares > Uint128::zero());

        let balance = token_balance(
            &QuerierWrapper::new(&deps.querier),
            &Addr::unchecked(token),
            &env.contract.address,
        )
        .unwrap();
        assert_eq!(balance, Uint128::new(1_000_000));
    }

    #[test]
    fn test_withdraw() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let deposit_amount = Uint128::new(1_000);
        let msg_deposit = ExecuteMsg::Deposit {
            amount: deposit_amount,
        };

        let info = message_info(&Addr::unchecked(strategy_manager), &[]);

        execute(deps.as_mut(), env.clone(), info.clone(), msg_deposit).unwrap();

        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.total_shares, Uint128::new(999));

        let withdraw_amount_shares = Uint128::new(1);
        let recipient = deps.api.addr_make("recipient").to_string();

        let msg_withdraw = ExecuteMsg::Withdraw {
            recipient: recipient.clone(),
            token: token.clone(),
            amount_shares: withdraw_amount_shares,
        };

        let res_withdraw = execute(deps.as_mut(), env.clone(), info.clone(), msg_withdraw);
        match res_withdraw {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 3);
                assert_eq!(response.attributes[0].key, "method");
                assert_eq!(response.attributes[0].value, "withdraw");
                assert!(response.attributes[1].key == "amount_to_send");
                assert!(response.attributes[2].key == "total_shares");

                assert_eq!(response.messages.len(), 1);
                match &response.messages[0].msg {
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr, msg, ..
                    }) => {
                        assert_eq!(*contract_addr, token);
                        let cw20_msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                        match cw20_msg {
                            Cw20ExecuteMsg::Transfer {
                                recipient: rec,
                                amount,
                            } => {
                                assert_eq!(rec, recipient.to_string());
                                assert_eq!(amount, Uint128::new(1));
                            }
                            _ => panic!("Unexpected message type"),
                        }
                    }
                    _ => panic!("Unexpected CosmosMsg"),
                }

                assert_eq!(response.events.len(), 1);
                let event = &response.events[0];
                assert_eq!(event.ty, "exchange_rate_emitted");
                assert_eq!(event.attributes.len(), 1);
                assert_eq!(event.attributes[0].key, "exchange_rate");
                assert_eq!(event.attributes[0].value, "1000000");

                let exchange_rate = event.attributes[0].value.parse::<u128>().unwrap();
                assert!(exchange_rate > 0, "Exchange rate should be positive");
            }
            Err(err) => {
                println!("Withdraw failed with error: {:?}", err);
                panic!("Withdraw test failed");
            }
        }
    }

    #[test]
    fn test_only_strategy_manager() {
        let (deps, _env, _info, _pauser_info, _unpauser_info, _token, strategy_manager) =
            instantiate_contract();

        let info_strategy_manager = message_info(&Addr::unchecked(strategy_manager.clone()), &[]);

        let result = only_strategy_manager(deps.as_ref(), &info_strategy_manager.clone());
        assert!(result.is_ok());

        let info_wrong = message_info(&Addr::unchecked("other_manager"), &[]);
        let result_wrong = only_strategy_manager(deps.as_ref(), &info_wrong);
        assert!(result_wrong.is_err());
    }

    #[test]
    fn test_before_deposit() {
        let (mut _deps, _env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let state = StrategyState {
            strategy_manager: Addr::unchecked(strategy_manager),
            underlying_token: Addr::unchecked(token.clone()),
            total_shares: Uint128::zero(),
        };

        let result = before_deposit(&state, &Addr::unchecked(token));
        assert!(result.is_ok());

        let wrong_token = Addr::unchecked("wrong_token");
        let result_wrong = before_deposit(&state, &wrong_token);
        assert!(result_wrong.is_err());
    }

    #[test]
    fn test_before_withdrawal() {
        let (mut _deps, _env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let state = StrategyState {
            strategy_manager: Addr::unchecked(strategy_manager),
            underlying_token: Addr::unchecked(token.clone()),
            total_shares: Uint128::zero(),
        };

        let result = before_withdrawal(&state, &Addr::unchecked(token));
        assert!(result.is_ok());

        let wrong_token = Addr::unchecked("wrong_token");
        let result_wrong = before_withdrawal(&state, &wrong_token);
        assert!(result_wrong.is_err());
    }
    #[test]
    fn test_query_explanation() {
        let (deps, env, _info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let query_msg = QueryMsg::Explanation {};

        let res = query(deps.as_ref(), env, query_msg).unwrap();

        let explanation_response: ExplanationResponse = from_json(res).unwrap();

        assert_eq!(
            explanation_response.explanation,
            "Base Strategy implementation to inherit from for more complex implementations"
                .to_string()
        );
    }

    #[test]
    fn test_shares_to_underlying_view() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, _strategy_manager) =
            instantiate_contract();

        let contract_address = env.contract.address.clone();

        deps.querier.update_wasm({
            let token_clone = token.clone();
            move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    let msg_clone = msg.clone();
                    if contract_addr == &token_clone {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&Cw20BalanceResponse {
                                        balance: Uint128::new(1_000_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg_clone,
                    })
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let amount_shares = Uint128::new(1_000);
        let result = shares_to_underlying_view(deps.as_ref(), env.clone(), amount_shares);

        match result {
            Ok(amount_to_send) => {
                assert_eq!(amount_to_send, Uint128::new(1000));
            }
            Err(e) => {
                panic!("Failed to convert shares to underlying: {:?}", e);
            }
        }
    }

    #[test]
    fn test_underlying_to_share_view() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, _strategy_manager) =
            instantiate_contract();

        let contract_address = env.contract.address.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr, msg, ..
            } => {
                let msg_clone = msg.clone();
                if contract_addr == &token {
                    let msg: Cw20QueryMsg = from_json(msg).unwrap();
                    if let Cw20QueryMsg::Balance { address } = msg {
                        if address == contract_address.to_string() {
                            return SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&Cw20BalanceResponse {
                                    balance: Uint128::new(1_000_000),
                                })
                                .unwrap(),
                            ));
                        }
                    }
                }
                SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: msg_clone,
                })
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "not implemented".to_string(),
                request: Binary::from(b"other".as_ref()),
            }),
        });

        let amount = Uint128::new(1_000);
        let share_to_send = underlying_to_share_view(deps.as_ref(), env.clone(), amount).unwrap();

        assert_eq!(share_to_send, Uint128::new(999));
    }

    #[test]
    fn test_shares() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let contract_address = env.contract.address.clone();
        deps.querier.update_wasm({
            let contract_address = contract_address.clone();
            move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    if *contract_addr == token {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&Cw20BalanceResponse {
                                        balance: Uint128::new(1_000_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let deposit_amount = Uint128::new(1_000);
        let msg_deposit = ExecuteMsg::Deposit {
            amount: deposit_amount,
        };

        let info = message_info(&Addr::unchecked(strategy_manager.clone()), &[]);
        let user = deps.api.addr_make("user").to_string();

        execute(deps.as_mut(), env.clone(), info.clone(), msg_deposit).unwrap();

        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert!(state.total_shares > Uint128::zero());

        deps.querier.update_wasm({
            let contract_address = contract_address.clone();
            let user_address = Addr::unchecked(user.clone());
            move |query| {
                match query {
                    WasmQuery::Smart {
                        contract_addr, msg, ..
                    } => {
                        if contract_addr == &strategy_manager {
                            let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                            match msg {
                                StrategyManagerQueryMsg::GetStakerStrategyShares {
                                    staker,
                                    strategy,
                                } => {
                                    if staker == user_address.to_string()
                                        && strategy == contract_address.to_string()
                                    {
                                        return SystemResult::Ok(ContractResult::Ok(
                                            to_json_binary(&StakerStrategySharesResponse {
                                                shares: Uint128::new(1_000),
                                            })
                                            .unwrap(),
                                        ));
                                    }
                                }
                                _ => {
                                    // Handle other cases if needed
                                }
                            }
                        }
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: "not implemented".to_string(),
                            request: msg.clone(),
                        })
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: Binary::from(b"other".as_ref()),
                    }),
                }
            }
        });

        let query_msg = QueryMsg::GetShares {
            staker: user,
            strategy: contract_address.to_string(),
        };
        let res: SharesResponse =
            from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();

        assert_eq!(res.total_shares, Uint128::new(1_000));
    }

    #[test]
    fn test_user_underlying_view() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, strategy_manager) =
            instantiate_contract();

        let contract_address = env.contract.address.clone();
        let user_addr = deps.api.addr_make("user").to_string();

        // Mock the balance query and staker strategy shares
        deps.querier.update_wasm({
            let user_addr_clone = user_addr.clone();
            let contract_address_clone = contract_address.clone();
            move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    if contract_addr == &token {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address_clone.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&Cw20BalanceResponse {
                                        balance: Uint128::new(1_000_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    } else if contract_addr == &strategy_manager {
                        let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                        if let StrategyManagerQueryMsg::GetStakerStrategyShares {
                            staker,
                            strategy,
                        } = msg
                        {
                            if staker == user_addr_clone.clone()
                                && strategy == contract_address_clone.to_string()
                            {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&StakerStrategySharesResponse {
                                        shares: Uint128::new(1_000),
                                    })
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                }
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let underlying_amount =
            user_underlying_view(deps.as_ref(), env.clone(), Addr::unchecked(user_addr)).unwrap();

        let expected_amount = Uint128::new(1000);
        assert_eq!(underlying_amount, expected_amount);
    }

    #[test]
    fn test_query_strategy_manager() {
        let (deps, env, _info, _pauser_info, _unpauser_info, _token, strategy_manager) =
            instantiate_contract();

        let query_msg = QueryMsg::GetStrategyManager {};
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let strategy_manager_response: StrategyManagerResponse = from_json(res).unwrap();

        let current_strategy_manager = strategy_manager_response.strate_manager_addr;

        assert_eq!(current_strategy_manager, Addr::unchecked(strategy_manager));
    }

    #[test]
    fn test_query_underlying_token() {
        let (deps, env, _info, _pauser_info, _unpauser_info, token, _strategy_manager) =
            instantiate_contract();

        let query_msg = QueryMsg::GetUnderlyingToken {};

        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let underlying_token_response: UnderlyingTokenResponse = from_json(res).unwrap();

        let underlying_token = underlying_token_response.underlying_token_addr;

        assert_eq!(underlying_token, Addr::unchecked(token));
    }

    #[test]
    fn test_query_total_shares() {
        let (deps, env, _info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let query_msg = QueryMsg::GetTotalShares {};
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();

        let total_shares_response: TotalSharesResponse = from_json(res).unwrap();

        assert_eq!(total_shares_response.total_shares, Uint128::zero());
    }

    #[test]
    fn test_emit_exchange_rate() {
        let virtual_token_balance = Uint128::new(1_000_000_000);
        let virtual_total_shares = Uint128::new(1_000_000);

        let expected_exchange_rate = virtual_token_balance
            .checked_mul(Uint128::new(1_000_000))
            .unwrap()
            .checked_div(virtual_total_shares)
            .unwrap();

        let res = emit_exchange_rate(virtual_token_balance, virtual_total_shares).unwrap();

        let expected_event = Event::new("exchange_rate_emitted")
            .add_attribute("exchange_rate", expected_exchange_rate.to_string());

        assert!(res.events.contains(&expected_event));

        println!("{:?}", res);
    }

    #[test]
    fn test_underlying_to_shares() {
        let (mut deps, env, _info, _pauser_info, _unpauser_info, token, _strategy_manager) =
            instantiate_contract();

        let contract_address: Addr = env.contract.address.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr, msg, ..
            } => {
                let msg_clone = msg.clone();
                if contract_addr == &token {
                    let msg: Cw20QueryMsg = from_json(msg).unwrap();
                    if let Cw20QueryMsg::Balance { address } = msg {
                        if address == contract_address.to_string() {
                            return SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&Cw20BalanceResponse {
                                    balance: Uint128::new(1_000_000),
                                })
                                .unwrap(),
                            ));
                        }
                    }
                }
                SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: msg_clone,
                })
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "not implemented".to_string(),
                request: Binary::from(b"other".as_ref()),
            }),
        });

        let amount_underlying = Uint128::new(1_000);
        let result = underlying_to_shares(deps.as_ref(), env.clone(), amount_underlying);

        match result {
            Ok(share_to_send) => {
                assert_eq!(share_to_send, Uint128::new(999));
            }
            Err(e) => {
                panic!("Failed to convert underlying to shares: {:?}", e);
            }
        }
    }

    #[test]
    fn test_migrate_owner_vs_non_owner() {
        let (mut deps, env, info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let migrate_msg = MigrateMsg {};
        let res = migrate(deps.as_mut(), env.clone(), &info, migrate_msg.clone()).unwrap();

        assert_eq!(res, Response::new().add_attribute("method", "migrate"));

        let version = get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_eq!(version.version, CONTRACT_VERSION);

        let non_owner_info = message_info(&Addr::unchecked("not_owner"), &[]);

        let res = migrate(deps.as_mut(), env, &non_owner_info, migrate_msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_pause() {
        let (mut deps, env, _info, pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let pause_msg = ExecuteMsg::Pause {};
        let res = execute(deps.as_mut(), env.clone(), pauser_info.clone(), pause_msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action", "PAUSED")]);

        let paused_state = PAUSED_STATE.load(&deps.storage).unwrap();
        assert_eq!(paused_state, 1);
    }

    #[test]
    fn test_unpause() {
        let (mut deps, env, _info, _pauser_info, unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

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
        let (mut deps, env, info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let new_pauser = deps.api.addr_make("new_pauser").to_string();

        let set_pauser_msg = ExecuteMsg::SetPauser {
            new_pauser: new_pauser.to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), set_pauser_msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "method" && a.value == "set_pauser"));

        let pauser = PAUSER.load(&deps.storage).unwrap();
        assert_eq!(pauser, Addr::unchecked(new_pauser));
    }

    #[test]
    fn test_set_unpauser() {
        let (mut deps, env, info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let new_unpauser = deps.api.addr_make("new_unpauser").to_string();

        let set_unpauser_msg = ExecuteMsg::SetUnpauser {
            new_unpauser: new_unpauser.to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), set_unpauser_msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "method" && a.value == "set_unpauser"));

        let unpauser = UNPAUSER.load(&deps.storage).unwrap();
        assert_eq!(unpauser, Addr::unchecked(new_unpauser));
    }

    #[test]
    fn test_set_strategy_manager() {
        let (mut deps, env, info, _pauser_info, _unpauser_info, _token, _strategy_manager) =
            instantiate_contract();

        let new_strategy_manager = deps.api.addr_make("new_strategy_manager").to_string();

        let set_strategy_manager_msg = ExecuteMsg::SetStrategyManager {
            new_strategy_manager: new_strategy_manager.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), set_strategy_manager_msg)
            .unwrap();

        assert!(res.events.iter().any(|e| e.ty == "strategy_manager_set"));

        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.strategy_manager, Addr::unchecked(new_strategy_manager));
    }
}
