use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SharesResponse},
    state::{StrategyState, STRATEGY_STATE},
    strategy_manager::QueryMsg as StrategyManagerQueryMsg,
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmQuery,
    WasmMsg, CosmosMsg, Event
};
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

    let state = StrategyState {
        strategy_manager: msg.strategy_manager.clone(),
        underlying_token: msg.underlying_token.clone(),
        total_shares: Uint128::zero(),
    };
    STRATEGY_STATE.save(deps.storage, &state)?;

    let underlying_token = msg.underlying_token.clone();

    let token_info: cw20::TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: underlying_token.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    let decimals = token_info.decimals;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("underlying_token", state.underlying_token.to_string())
        .add_event(Event::new("instantiate")
            .add_attribute("underlying_token_address", state.underlying_token.to_string())
            .add_attribute("underlying_token_decimals", decimals.to_string())))
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
        ExecuteMsg::Withdraw { recipient, amount_shares } => withdraw(deps, env, info, recipient, amount_shares),
    }
}

fn _ensure_strategy_manager(info: &MessageInfo, strategy_manager: &Addr) -> Result<(), ContractError> {
    if info.sender != strategy_manager {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn _before_deposit(state: &StrategyState, token: &Addr) -> Result<(), ContractError> {
    if token != state.underlying_token {
        return Err(ContractError::InvalidToken {});
    }
    Ok(())
}

fn _before_withdrawal(state: &StrategyState, token: &Addr) -> Result<(), ContractError> {
    if token != state.underlying_token {
        return Err(ContractError::InvalidToken {});
    }
    Ok(())
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    _ensure_strategy_manager(&info, &state.strategy_manager)?;
    _before_deposit(&state, &state.underlying_token)?;

    let balance = _token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;

    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let new_shares = (amount * virtual_share_amount) / virtual_prior_token_balance;

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    state.total_shares += new_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    let exchange_rate_event = _emit_exchange_rate(virtual_token_balance, state.total_shares + SHARES_OFFSET)?;

    let mut response = Response::new();

    response = response.add_attributes(vec![
        ("method", "deposit"),
        ("new_shares", &new_shares.to_string()),
        ("total_shares", &state.total_shares.to_string()),
    ]);

    response.events.extend(exchange_rate_event.events);

    Ok(response)     
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount_shares: Uint128,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    _ensure_strategy_manager(&info, &state.strategy_manager)?;
    _before_withdrawal(&state, &state.underlying_token)?;

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let balance = _token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;

    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;

    state.total_shares -= amount_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    let exchange_rate_event = _emit_exchange_rate(virtual_token_balance - amount_to_send, state.total_shares + SHARES_OFFSET)?;

    let mut response = _after_withdrawal(&state.underlying_token, &recipient, amount_to_send)?;

    response = response.add_attributes(vec![
        ("method", "withdraw"),
        ("amount_to_send", amount_to_send.to_string().as_str()),
        ("total_shares", state.total_shares.to_string().as_str()),
    ]);

    response.events.extend(exchange_rate_event.events);

    Ok(response)
}

fn _after_withdrawal(token: &Addr, recipient: &Addr, amount: Uint128) -> Result<Response, ContractError> {
    let msg = WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        })?,
        funds: vec![],
    };

    Ok(Response::new().add_message(CosmosMsg::Wasm(msg)))
}

pub fn shares(deps: Deps, user: Addr, strategy: Addr) -> StdResult<SharesResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;

    let shares: Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.strategy_manager.to_string(),
        msg: to_json_binary(&StrategyManagerQueryMsg::GetStakerStrategyShares { staker: user.clone(), strategy })?,
    }))?;

    Ok(SharesResponse { total_shares: shares })
}

pub fn shares_to_underlying_view(deps: Deps, env: Env, amount_shares: Uint128) -> StdResult<Uint128> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    let balance = _token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    
    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;

    Ok(amount_to_send)
}

pub fn underlying_to_share_view(deps: Deps, env: Env, amount: Uint128) -> StdResult<Uint128> {
    let state: StrategyState = STRATEGY_STATE.load(deps.storage)?;
    let balance = _token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    
    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let share_to_send = (amount * virtual_share_amount) / virtual_prior_token_balance;

    Ok(share_to_send)        
}

pub fn underlying_to_shares(deps: Deps, env: Env, amount_underlying: Uint128) -> StdResult<Uint128> {
    let share_to_send = underlying_to_share_view(deps, env, amount_underlying)?;
    Ok(share_to_send)        
}

pub fn user_underlying_view(deps: Deps, env: Env, user: Addr) -> StdResult<Uint128> {
    let strategy = env.contract.address.clone(); 
    let user_shares = shares(deps, user, strategy.clone())?.total_shares;

    let amount_to_send = shares_to_underlying_view(deps, env, user_shares)?;

    Ok(amount_to_send)
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

fn _emit_exchange_rate(virtual_token_balance: Uint128, virtual_total_shares: Uint128) -> StdResult<Response> {
    let exchange_rate = (virtual_token_balance.checked_mul(Uint128::new(1_000_000))?)
        .checked_div(virtual_total_shares)?;

    let event = Event::new("exchange_rate_emitted")
        .add_attribute("exchange_rate", exchange_rate.to_string());

    Ok(Response::new().add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetShares { staker, strategy } => to_json_binary(&shares(deps, staker, strategy)?),
        QueryMsg::SharesToUnderlyingView { amount_shares } => to_json_binary(&shares_to_underlying_view(deps, env, amount_shares)?),
        QueryMsg::UnderlyingToShareView { amount } => to_json_binary(&underlying_to_share_view(deps, env, amount)?),
        QueryMsg::UserUnderlyingView { user } => to_json_binary(&user_underlying_view(deps, env, user)?),
        QueryMsg::GetStrategyManager {} => query_strategy_manager(deps),
        QueryMsg::GetUnderlyingToken {} => query_underlying_token(deps),
        QueryMsg::GetTotalShares {} => query_total_shares(deps),
        QueryMsg::Explanation {} => to_json_binary(&query_explanation()?),
        QueryMsg::UnderlyingToShares { amount_underlying } => {
            let share_to_send = underlying_to_shares(deps, env, amount_underlying)?;
            to_json_binary(&share_to_send)
        },
        QueryMsg::GetStrategyState {} => to_json_binary(&query_strategy_state(deps)?),
    }
}

fn query_strategy_manager(deps: Deps) -> StdResult<Binary> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    to_json_binary(&state.strategy_manager)
}

fn query_underlying_token(deps: Deps) -> StdResult<Binary> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    to_json_binary(&state.underlying_token)
}

fn query_total_shares(deps: Deps) -> StdResult<Binary> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    to_json_binary(&state.total_shares)
}

fn query_explanation() -> StdResult<String> {
    Ok("Base Strategy implementation to inherit from for more complex implementations".to_string())
}

pub fn query_strategy_state(deps: Deps) -> StdResult<StrategyState> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{from_json, OwnedDeps, SystemResult, ContractResult, SystemError, Binary, WasmQuery};
    use cw20::TokenInfoResponse;

    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
        };

        deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        match msg {
                            Cw20QueryMsg::TokenInfo {} => {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&TokenInfoResponse {
                                    name: "Mock Token".to_string(),
                                    symbol: "MTK".to_string(),
                                    decimals: 8,
                                    total_supply: Uint128::new(1_000_000),
                                }).unwrap()));
                            },
                            Cw20QueryMsg::Balance { address:_ } => {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse {
                                    balance: Uint128::new(1_000_000),
                                }).unwrap()));
                            },
                            _ => {}
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        deps
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
        };

        deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::TokenInfo {} = msg {
                            return SystemResult::Ok(ContractResult::Ok(to_json_binary(&TokenInfoResponse {
                                name: "Mock Token".to_string(),
                                symbol: "MTK".to_string(),
                                decimals: 8,
                                total_supply: Uint128::new(1_000_000),
                            }).unwrap()));
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "strategy_manager");
        assert_eq!(res.attributes[1].value, "manager");
        assert_eq!(res.attributes[2].key, "underlying_token");
        assert_eq!(res.attributes[2].value, "token");

        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        assert_eq!(event.ty, "instantiate");
        assert_eq!(event.attributes.len(), 2);
        assert_eq!(event.attributes[0].key, "underlying_token_address");
        assert_eq!(event.attributes[0].value, "token");
        assert_eq!(event.attributes[1].key, "underlying_token_decimals");
        assert_eq!(event.attributes[1].value, "8");
    }

    #[test]
    fn test_deposit() {
        let mut deps = setup_contract();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("manager"), &[]);
    
        let amount = Uint128::new(1_000);
        let msg = ExecuteMsg::Deposit { amount };
    
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
        assert_eq!(event.attributes[0].value, "1000000000");
    
        let exchange_rate = event.attributes[0].value.parse::<u128>().unwrap();
        assert!(exchange_rate > 0, "Exchange rate should be positive");
    
        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert!(state.total_shares > Uint128::zero());
    
        let balance = _token_balance(&QuerierWrapper::new(&deps.querier), &Addr::unchecked("token"), &env.contract.address).unwrap();
        assert_eq!(balance, Uint128::new(1_000_000)); 
    }    

    #[test]
    fn test_withdraw() {
        let mut deps = setup_contract();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("manager"), &[]);
    
        let deposit_amount = Uint128::new(1_000);
        let msg_deposit = ExecuteMsg::Deposit { amount: deposit_amount };
        execute(deps.as_mut(), env.clone(), info.clone(), msg_deposit).unwrap();
    
        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.total_shares, Uint128::new(1));
    
        let withdraw_amount_shares = Uint128::new(1);
        let recipient = Addr::unchecked("recipient");
        let msg_withdraw = ExecuteMsg::Withdraw { recipient: recipient.clone(), amount_shares: withdraw_amount_shares };
    
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
                    CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) => {
                        assert_eq!(contract_addr, "token");
                        let msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                        match msg {
                            Cw20ExecuteMsg::Transfer { recipient: rec, amount } => {
                                assert_eq!(rec, recipient.to_string());
                                assert_eq!(amount, Uint128::new(1_000));
                            },
                            _ => panic!("Unexpected message type"),
                        }
                    },
                    _ => panic!("Unexpected CosmosMsg"),
                }
    
                assert_eq!(response.events.len(), 1); // One for withdrawal and one for exchange rate
                let event = &response.events[0]; // Second event should be the exchange rate
                assert_eq!(event.ty, "exchange_rate_emitted");
                assert_eq!(event.attributes.len(), 1);
                assert_eq!(event.attributes[0].key, "exchange_rate");
                assert_eq!(event.attributes[0].value, "1000000000");

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
    fn test_ensure_strategy_manager() {
        let info = message_info(&Addr::unchecked("manager"), &[]);
        let strategy_manager = Addr::unchecked("manager");

        let result = _ensure_strategy_manager(&info, &strategy_manager);
        assert!(result.is_ok());

        let info_wrong = message_info(&Addr::unchecked("other_manager"), &[]);
        let result_wrong = _ensure_strategy_manager(&info_wrong, &strategy_manager);
        assert!(result_wrong.is_err());
    }

    #[test]
    fn test_before_deposit() {
        let state = StrategyState {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
            total_shares: Uint128::zero(),
        };

        let token = Addr::unchecked("token");
        let result = _before_deposit(&state, &token);
        assert!(result.is_ok());

        let wrong_token = Addr::unchecked("wrong_token");
        let result_wrong = _before_deposit(&state, &wrong_token);
        assert!(result_wrong.is_err());
    }

    #[test]
    fn test_before_withdrawal() {
        let state = StrategyState {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
            total_shares: Uint128::zero(),
        };

        let token = Addr::unchecked("token");
        let result = _before_withdrawal(&state, &token);
        assert!(result.is_ok());

        let wrong_token = Addr::unchecked("wrong_token");
        let result_wrong = _before_withdrawal(&state, &wrong_token);
        assert!(result_wrong.is_err());
    }

    #[test]
    fn test_query_explanation() {
        let deps: OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier> = setup_contract();
        let env = mock_env();
    
        let query_msg = QueryMsg::Explanation {};
    
        let res = query(deps.as_ref(), env, query_msg).unwrap();
    
        let explanation: String = from_json(res).unwrap();
        assert_eq!(explanation, "Base Strategy implementation to inherit from for more complex implementations".to_string());
    }    

    #[test]
    fn test_shares_to_underlying_view() {
        let mut deps = setup_contract();
        let env = mock_env();

        let contract_address = env.contract.address.clone();

        deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    let msg_clone = msg.clone(); 
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1_000_000) }).unwrap()));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg_clone,
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let query_msg = QueryMsg::SharesToUnderlyingView { amount_shares: Uint128::new(1_000) };
        let response = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let amount_to_send: Uint128 = from_json(response).unwrap();
        
        assert_eq!(amount_to_send, Uint128::new(1_001_000));
    }

    #[test]
    fn test_underlying_to_share_view() {
        let mut deps = setup_contract();
        let env = mock_env();
    
        let contract_address = env.contract.address.clone();
    
           deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    let msg_clone = msg.clone(); 
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1_000_000) }).unwrap()));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg_clone,
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let query_msg = QueryMsg::UnderlyingToShareView { amount: Uint128::new(1_000) };
        let response = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let share_to_send: Uint128 = from_json(response).unwrap();

        assert_eq!(share_to_send, Uint128::new(1));
    }

    #[test]
    fn test_shares() {
        let mut deps = setup_contract();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("manager"), &[]);
    
        let contract_address = env.contract.address.clone();
        deps.querier.update_wasm({
            let contract_address = contract_address.clone(); 
            move |query| {
                match query {
                    WasmQuery::Smart { contract_addr, msg, .. } => {
                        if contract_addr == "token" {
                            let msg: Cw20QueryMsg = from_json(msg).unwrap();
                            if let Cw20QueryMsg::Balance { address } = msg {
                                if address == contract_address.to_string() {
                                    return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1_000_000) }).unwrap()));
                                }
                            }
                        }
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: "not implemented".to_string(),
                            request: msg.clone(),
                        })
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: Binary::from(b"other".as_ref()),
                    }),
                }
            }
        });
    
        let deposit_amount = Uint128::new(1_000);
        let msg_deposit = ExecuteMsg::Deposit { amount: deposit_amount };
        execute(deps.as_mut(), env.clone(), info.clone(), msg_deposit).unwrap();
    
        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert!(state.total_shares > Uint128::zero());
    
        deps.querier.update_wasm({
            let contract_address = contract_address.clone(); 
            move |query| {
                match query {
                    WasmQuery::Smart { contract_addr, msg, .. } => {
                        if contract_addr == "manager" {
                            let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                            match msg {
                                StrategyManagerQueryMsg::GetStakerStrategyShares { staker, strategy } => {
                                    if staker == Addr::unchecked("user") && strategy == contract_address {
                                        return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Uint128::new(1_000)).unwrap()));
                                    }
                                },
                                _ => {
                                    // Handle other cases if needed
                                },
                            }
                        }
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: "not implemented".to_string(),
                            request: msg.clone(),
                        })
                    },
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: Binary::from(b"other".as_ref()),
                    }),
                }
            }
        });
    
        let query_msg = QueryMsg::GetShares { staker: Addr::unchecked("user"), strategy: contract_address.clone() };
        let res: SharesResponse = from_json(query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        
        assert_eq!(res.total_shares, Uint128::new(1_000));
    }

    #[test]
    fn test_user_underlying_view() {
        let mut deps = setup_contract();
        let env = mock_env();
    
        let contract_address = env.contract.address.clone();
    
        deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1_000_000) }).unwrap()));
                            }
                        }
                    } else if contract_addr == "manager" {
                        let msg: StrategyManagerQueryMsg = from_json(msg).unwrap();
                        if let StrategyManagerQueryMsg::GetStakerStrategyShares { staker, strategy } = msg {
                            if staker == Addr::unchecked("user") && strategy == contract_address {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Uint128::new(1_000)).unwrap()));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg.clone(),
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });
    
        let user_addr = Addr::unchecked("user");
        let query_msg = QueryMsg::UserUnderlyingView { user: user_addr };
    
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let underlying_amount: Uint128 = from_json(res).unwrap();
    
        let expected_amount = Uint128::new(1_001_000); 
        assert_eq!(underlying_amount, expected_amount);
    }    

    #[test]
    fn test_query_strategy_manager() {
        let deps = setup_contract();
        let env = mock_env();
    
        let query_msg = QueryMsg::GetStrategyManager {};
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    
        let strategy_manager: Addr = from_json(res).unwrap();
        assert_eq!(strategy_manager, Addr::unchecked("manager"));
    }
    
    #[test]
    fn test_query_underlying_token() {
        let deps = setup_contract();
        let env = mock_env();
    
        let query_msg = QueryMsg::GetUnderlyingToken {};
    
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let underlying_token: Addr = from_json(res).unwrap();
    
        assert_eq!(underlying_token, Addr::unchecked("token"));
    }    

    #[test]
    fn test_query_total_shares() {
        let deps = setup_contract();
        let env = mock_env();
    
        let query_msg = QueryMsg::GetTotalShares {};
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let total_shares: Uint128 = from_json(res).unwrap();
    
        assert_eq!(total_shares, Uint128::zero());
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

        let res = _emit_exchange_rate(virtual_token_balance, virtual_total_shares).unwrap();

        let expected_event = Event::new("exchange_rate_emitted")
            .add_attribute("exchange_rate", expected_exchange_rate.to_string());

        assert!(res.events.contains(&expected_event));

        println!("{:?}", res);
    }

    #[test]
    fn test_underlying_to_shares() {
        let mut deps = setup_contract();
        let env = mock_env();

        let contract_address: Addr = env.contract.address.clone();

        deps.querier.update_wasm(move |query| {
            match query {
                WasmQuery::Smart { contract_addr, msg, .. } => {
                    let msg_clone = msg.clone(); 
                    if contract_addr == "token" {
                        let msg: Cw20QueryMsg = from_json(msg).unwrap();
                        if let Cw20QueryMsg::Balance { address } = msg {
                            if address == contract_address.to_string() {
                                return SystemResult::Ok(ContractResult::Ok(to_json_binary(&Cw20BalanceResponse { balance: Uint128::new(1_000_000) }).unwrap()));
                            }
                        }
                    }
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "not implemented".to_string(),
                        request: msg_clone,
                    })
                },
                _ => SystemResult::Err(SystemError::InvalidRequest {
                    error: "not implemented".to_string(),
                    request: Binary::from(b"other".as_ref()),
                }),
            }
        });

        let query_msg = QueryMsg::UnderlyingToShares { amount_underlying: Uint128::new(1_000) };
        let response = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let share_to_send: Uint128 = from_json(response).unwrap();
        
        assert_eq!(share_to_send, Uint128::new(1)); 
    }
}