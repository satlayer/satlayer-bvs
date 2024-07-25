use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SharesResponse},
    state::{StrategyState, STRATEGY_STATE},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, Uint128, Uint256, WasmMsg, WasmQuery, BalanceResponse, CosmosMsg,

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

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    ensure_strategy_manager(&info, &state.strategy_manager)?;

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

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let balance = query_token_balance(&deps.querier, &state.underlying_token, &env.contract.address)?;
    let amount_to_send = calculate_amount_to_send(&state, amount_shares, balance)?;

    state.total_shares -= amount_shares;
    STRATEGY_STATE.save(deps.storage, &state)?;

    _after_withdrawal(&state.underlying_token, &recipient, amount_to_send)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_attribute("amount_to_send", amount_to_send.to_string())
        .add_attribute("total_shares", state.total_shares.to_string()))
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetShares {} => to_json_binary(&query_shares(deps)?),
    }
}

fn query_shares(deps: Deps) -> StdResult<SharesResponse> {
    let state = STRATEGY_STATE.load(deps.storage)?;
    Ok(SharesResponse {
        total_shares: state.total_shares,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

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
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("manager", &[]);

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let deposit_msg = ExecuteMsg::Deposit {
            amount: Uint128::new(1_000),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), deposit_msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "deposit");
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("manager", &[]);

        let msg = InstantiateMsg {
            strategy_manager: Addr::unchecked("manager"),
            underlying_token: Addr::unchecked("token"),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let deposit_msg = ExecuteMsg::Deposit {
            amount: Uint128::new(1_000),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), deposit_msg).unwrap();

        let withdraw_msg = ExecuteMsg::Withdraw {
            recipient: Addr::unchecked("recipient"),
            amount_shares: Uint128::new(500),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), withdraw_msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "withdraw");
    }
}