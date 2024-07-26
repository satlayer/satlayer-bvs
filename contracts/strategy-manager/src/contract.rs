use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SharesResponse},
    state::{StrategyManagerState, STRATEGY_MANAGER_STATE},
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:strategy-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = StrategyManagerState {
        delegation_manager: msg.delegation_manager,
        eigen_pod_manager: msg.eigen_pod_manager,
        slasher: msg.slasher,
    };
    STRATEGY_MANAGER_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("delegation_manager", state.delegation_manager.to_string())
        .add_attribute("eigen_pod_manager", state.eigen_pod_manager.to_string())
        .add_attribute("slasher", state.slasher.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { strategy, amount } => deposit(deps, env, info, strategy, amount),
        ExecuteMsg::Withdraw { strategy, amount_shares } => withdraw(deps, env, info, strategy, amount_shares),
    }
}

fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategy: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    // 在这里你可以添加更多的逻辑，例如验证策略的白名单

    let token_balance = query_token_balance(&deps.querier, &strategy, &env.contract.address)?;
    let new_shares = calculate_new_shares(&state, amount, token_balance)?;

    // 更新存储中的股份信息
    // 省略：更新具体的用户股份逻辑

    Ok(Response::new()
        .add_attribute("method", "deposit")
        .add_attribute("new_shares", new_shares.to_string()))
}

fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategy: Addr,
    amount_shares: Uint128,
) -> Result<Response, ContractError> {
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    // 在这里你可以添加更多的逻辑，例如验证策略的白名单

    if amount_shares > state.total_shares {
        return Err(ContractError::InsufficientShares {});
    }

    let token_balance = query_token_balance(&deps.querier, &strategy, &env.contract.address)?;
    let amount_to_send = calculate_amount_to_send(&state, amount_shares, token_balance)?;

    // 更新存储中的股份信息
    // 省略：更新具体的用户股份逻辑

    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: amount_to_send,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("method", "withdraw")
        .add_attribute("amount_to_send", amount_to_send.to_string()))
}

fn calculate_new_shares(state: &StrategyManagerState, amount: Uint128, balance: Uint128) -> Result<Uint128, ContractError> {
    // 计算新的股份逻辑，类似于你提供的示例
    let new_shares = amount * state.total_shares / (balance + Uint128::new(1));
    if new_shares.is_zero() {
        return Err(ContractError::ZeroNewShares {});
    }
    Ok(new_shares)
}

fn calculate_amount_to_send(state: &StrategyManagerState, amount_shares: Uint128, balance: Uint128) -> StdResult<Uint128> {
    // 计算要发送的金额，类似于你提供的示例
    let amount_to_send = balance * amount_shares / (state.total_shares + Uint128::new(1));
    Ok(amount_to_send)
}

fn query_token_balance(querier: &cosmwasm_std::QuerierWrapper, token: &Addr, account: &Addr) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&cosmwasm_std::QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
        contract_addr: token.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account.to_string(),
        })?,
    }))?;
    Ok(res.balance)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetShares { user } => to_binary(&query_shares(deps, env, user)?),
    }
}

fn query_shares(deps: Deps, _env: Env, user: Addr) -> StdResult<SharesResponse> {
    // 查询用户的股份信息
    let state = STRATEGY_MANAGER_STATE.load(deps.storage)?;

    // 假设从策略管理器合约中查询股份信息
    let shares_response: SharesResponse = deps.querier.query(&cosmwasm_std::QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
        contract_addr: state.delegation_manager.to_string(),
        msg: to_binary(&QueryMsg::GetShares { user: user.clone() })?,
    }))?;

    Ok(SharesResponse { total_shares: shares_response.total_shares })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            delegation_manager: Addr::unchecked("delegation_manager"),
            eigen_pod_manager: Addr::unchecked("eigen_pod_manager"),
            slasher: Addr::unchecked("slasher"),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].value, "instantiate");
    }

    #[test]
    fn test_deposit() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            delegation_manager: Addr::unchecked("delegation_manager"),
            eigen_pod_manager: Addr::unchecked("eigen_pod_manager"),
            slasher: Addr::unchecked("slasher"),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let deposit_msg = ExecuteMsg::Deposit {
            strategy: Addr::unchecked("strategy"),
            amount: Uint128::new(1000),
        };
        let info = mock_info("user", &[]);
        let res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].value, "deposit");
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            delegation_manager: Addr::unchecked("delegation_manager"),
            eigen_pod_manager: Addr::unchecked("eigen_pod_manager"),
            slasher: Addr::unchecked("slasher"),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let withdraw_msg = ExecuteMsg::Withdraw {
            strategy: Addr::unchecked("strategy"),
            amount_shares: Uint128::new(1000),
        };
        let info = mock_info("user", &[]);
        let res = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].value, "withdraw");
    }

    #[test]
    fn query_shares_test() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            delegation_manager: Addr::unchecked("delegation_manager"),
            eigen_pod_manager: Addr::unchecked("eigen_pod_manager"),
            slasher: Addr::unchecked("slasher"),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let query_msg = QueryMsg::GetShares {
            user: Addr::unchecked("user"),
        };
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let res: SharesResponse = from_binary(&bin).unwrap();
        assert_eq!(res.total_shares, Uint128::zero());
    }
}
