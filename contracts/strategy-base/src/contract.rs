use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SharesResponse},
    state::{StrategyState, STRATEGY_STATE},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256, Addr, Coin,
};
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

    let state = StrategyState {
        strategy_manager: msg.strategy_manager,
        underlying_token: msg.underlying_token,
        total_shares: Uint256::zero(),
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
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    if info.sender != state.strategy_manager {
        return Err(ContractError::Unauthorized {});
    }

    let new_shares = calculate_new_shares(&state, amount)?;
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
    amount_shares: Uint256,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    if info.sender != state.strategy_manager {
        return Err(ContractError::Unauthorized {});
    }

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let amount_to_send = calculate_amount_to_send(&state, amount_shares)?;
    state.total_shares -= amount_shares;

    STRATEGY_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_attribute("amount_to_send", amount_to_send.to_string())
        .add_attribute("total_shares", state.total_shares.to_string()))
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    let mut state = STRATEGY_STATE.load(deps.storage)?;

    if info.sender != state.strategy_manager {
        return Err(ContractError::Unauthorized {});
    }

    state.strategy_manager = new_owner.clone();

    STRATEGY_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

fn calculate_new_shares(state: &StrategyState, amount: Uint256) -> StdResult<Uint256> {
    let virtual_share_amount = state.total_shares + Uint256::from(1_000u128);
    let virtual_token_balance = state.total_shares + Uint256::from(1_000u128);
    let virtual_prior_token_balance = virtual_token_balance - amount;
    let new_shares = (amount * virtual_share_amount) / virtual_prior_token_balance;
    if new_shares.is_zero() {
        return Err(StdError::generic_err("new_shares cannot be zero"));
    }
    Ok(new_shares)
}

fn calculate_amount_to_send(state: &StrategyState, amount_shares: Uint256) -> StdResult<Uint256> {
    let virtual_total_shares = state.total_shares + Uint256::from(1_000u128);
    let virtual_token_balance = state.total_shares + Uint256::from(1_000u128);
    let amount_to_send = (virtual_token_balance * amount_shares) / virtual_total_shares;
    Ok(amount_to_send)
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
            strategy_manager: "manager".to_string(),
            underlying_token: "token".to_string(),
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
            strategy_manager: "manager".to_string(),
            underlying_token: "token".to_string(),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let deposit_msg = ExecuteMsg::Deposit {
            amount: Uint256::from(1_000u128),
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
            strategy_manager: "manager".to_string(),
            underlying_token: "token".to_string(),
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let deposit_msg = ExecuteMsg::Deposit {
            amount: Uint256::from(1_000u128),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), deposit_msg).unwrap();

        let withdraw_msg = ExecuteMsg::Withdraw {
            recipient: Addr::unchecked("recipient"),
            amount_shares: Uint256::from(500u128),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), withdraw_msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "withdraw");
    }
}
