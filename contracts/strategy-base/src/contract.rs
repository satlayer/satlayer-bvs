use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SharesResponse, StakerStrategySharesResponse},
    state::{StrategyState, STRATEGY_STATE},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery, CosmosMsg,

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
        strategy_manager: msg.strategy_manager,
        underlying_token: msg.underlying_token,
        total_shares: Uint128::zero(),
    };
    STRATEGY_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", state.strategy_manager.to_string())
        .add_attribute("underlying_token", state.underlying_token.to_string()))
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

fn ensure_strategy_manager(info: &MessageInfo, strategy_manager: &Addr) -> Result<(), ContractError> {
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

    ensure_strategy_manager(&info, &state.strategy_manager)?;
    _before_deposit(&state, &state.underlying_token)?;

    let balance = query_token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    let new_shares = calculate_new_shares(&state, amount, balance)?;

    state.total_shares += new_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "deposit")
        .add_attribute("new_shares", new_shares.to_string())
        .add_attribute("total_shares", state.total_shares.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount_shares: Uint128,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    ensure_strategy_manager(&info, &state.strategy_manager)?;
    _before_withdrawal(&state, &state.underlying_token)?;

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let balance = query_token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    let amount_to_send = calculate_amount_to_send(&state, amount_shares, balance)?;

    state.total_shares -= amount_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    let mut response = _after_withdrawal(&state.underlying_token, &recipient, amount_to_send)?;

    response = response.add_attributes(vec![
        ("method", "withdraw"),
        ("amount_to_send", amount_to_send.to_string().as_str()),
        ("total_shares", state.total_shares.to_string().as_str()),
    ]);

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

fn calculate_new_shares(state: &StrategyState, amount: Uint128, balance: Uint128) -> Result<Uint128, ContractError> {
    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let new_shares = (amount * virtual_share_amount) / virtual_prior_token_balance;

    // Debug print statements
    println!("calculate_new_shares - state.total_shares: {}", state.total_shares);
    println!("calculate_new_shares - SHARES_OFFSET: {}", SHARES_OFFSET);
    println!("calculate_new_shares - virtual_share_amount: {}", virtual_share_amount);
    println!("calculate_new_shares - balance: {}", balance);
    println!("calculate_new_shares - BALANCE_OFFSET: {}", BALANCE_OFFSET);
    println!("calculate_new_shares - virtual_token_balance: {}", virtual_token_balance);
    println!("calculate_new_shares - amount: {}", amount);
    println!("calculate_new_shares - virtual_prior_token_balance: {}", virtual_prior_token_balance);
    println!("calculate_new_shares - new_shares: {}", new_shares);

    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }

    Ok(new_shares)
}

fn calculate_amount_to_send(state: &StrategyState, amount_shares: Uint128, balance: Uint128) -> StdResult<Uint128> {
    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;
    
    Ok(amount_to_send)
}

fn query_token_balance(querier: &QuerierWrapper, token: &Addr, account: &Addr) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::Balance {
            address: account.to_string(),
        })?,
    }))?;
    Ok(res.balance)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetShares { user } => to_json_binary(&query_shares(deps, env, user)?),
    }
}

fn query_shares(deps: Deps, env: Env, user: Addr) -> StdResult<SharesResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    let shares_response = query_staker_strategy_shares(&deps.querier, &state.strategy_manager, &user, &env.contract.address)?;
    Ok(SharesResponse { total_shares: shares_response.shares })
}

fn query_staker_strategy_shares(querier: &QuerierWrapper, strategy_manager: &Addr, user: &Addr, _strategy: &Addr) -> StdResult<StakerStrategySharesResponse> {
    let msg = to_json_binary(&QueryMsg::GetShares {
        user: user.clone(),
    })?;

    let res: StakerStrategySharesResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: strategy_manager.to_string(),
        msg,
    }))?;

    Ok(res)
}

pub fn query_explanation() -> StdResult<String> {
    Ok("Base Strategy implementation to inherit from for more complex implementations".to_string())
}

pub fn shares_to_underlying_view(deps: Deps, env: Env, amount_shares: Uint128) -> StdResult<Uint128> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    let balance = query_token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    
    let virtual_total_shares = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;

    Ok(amount_to_send)
}

pub fn underlying_to_share_view(deps: Deps, env: Env, amount: Uint128) -> StdResult<Uint128> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    let balance = query_token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    
    let virtual_share_amount = state.total_shares + SHARES_OFFSET;
    let virtual_token_balance = balance + BALANCE_OFFSET;
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let share_to_send = (amount * virtual_share_amount) / virtual_prior_token_balance;

    Ok(share_to_send)        
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{from_json, OwnedDeps, SystemResult, ContractResult, SystemError, Binary, WasmQuery};

    // Helper function to setup contract with initial state
    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
        };

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

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "strategy_manager");
        assert_eq!(res.attributes[1].value, "manager");
        assert_eq!(res.attributes[2].key, "underlying_token");
        assert_eq!(res.attributes[2].value, "token");
    }

    #[test]
    fn test_deposit() {
        let mut deps = setup_contract();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("manager"), &[]);

        let contract_address = env.contract.address.clone();

        // Mock balance query response
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

        let amount = Uint128::new(1_000);
        let msg = ExecuteMsg::Deposit { amount };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit");
        assert!(res.attributes[1].key == "new_shares");
        assert!(res.attributes[2].key == "total_shares");

        // Query state to check total_shares
        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert!(state.total_shares > Uint128::zero());

        // Verify the balance query was made
        let balance = query_token_balance(&QuerierWrapper::new(&deps.querier), &Addr::unchecked("token"), &env.contract.address).unwrap();
        assert_eq!(balance, Uint128::new(1_000_000)); 
    }

    #[test]
    fn test_withdraw() {
        let mut deps = setup_contract();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("manager"), &[]);
        
        let contract_address = env.contract.address.clone();
        
        // Mock balance query response
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
        
        let deposit_amount = Uint128::new(1_000);
        let msg_deposit = ExecuteMsg::Deposit { amount: deposit_amount };
        execute(deps.as_mut(), env.clone(), info.clone(), msg_deposit).unwrap();
        
        // Verify the shares were added to total_shares
        let state = STRATEGY_STATE.load(&deps.storage).unwrap();
        assert_eq!(state.total_shares, Uint128::new(1)); // Ensure total_shares is as expected
    
        let withdraw_amount_shares = Uint128::new(1); // Adjust the withdraw amount to match available shares
    
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
                
                // Check if a transfer message was added
                assert_eq!(response.messages.len(), 1);
                match &response.messages[0].msg {
                    CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }) => {
                        assert_eq!(contract_addr, "token");
                        let msg: Cw20ExecuteMsg = from_json(msg).unwrap();
                        match msg {
                            Cw20ExecuteMsg::Transfer { recipient: rec, amount } => {
                                assert_eq!(rec, recipient.to_string());
                                assert_eq!(amount, Uint128::new(1_000)); // Adjust expected amount based on logic
                            },
                            _ => panic!("Unexpected message type"),
                        }
                    },
                    _ => panic!("Unexpected CosmosMsg"),
                }
            }
            Err(err) => {
                println!("Withdraw failed with error: {:?}", err);
                panic!("Withdraw test failed");
            }
        }
    }
}