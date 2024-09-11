use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, STRATEGIES};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;

use crate::query::MsgInstantiateContractResponse;
use common::base::{InstantiateMsg as StrategyInstantiateMsg, StrategyState};

use common::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use common::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};

const CONTRACT_NAME: &str = "strategy-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: deps.api.addr_validate(&msg.initial_owner)?,
        strategy_code_id: msg.strategy_code_id,
        only_owner_can_create: msg.only_owner_can_create,
        strategy_manager: deps.api.addr_validate(&msg.strategy_manager)?,
        pauser: deps.api.addr_validate(&msg.pauser)?,
        unpauser: deps.api.addr_validate(&msg.unpauser)?,
    };

    CONFIG.save(deps.storage, &config)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateStrategy {
            token,
            pauser,
            unpauser,
        } => {
            let token_addr = deps.api.addr_validate(&token)?;
            let pauser_addr = deps.api.addr_validate(&pauser)?;
            let unpauser_addr = deps.api.addr_validate(&unpauser)?;

            deploy_new_strategy(deps, env, info, token_addr, pauser_addr, unpauser_addr)
        }
        ExecuteMsg::UpdateConfig {
            new_owner,
            strategy_code_id,
            only_owner_can_create,
        } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;

            update_config(
                deps,
                info,
                new_owner_addr,
                strategy_code_id,
                only_owner_can_create,
            )
        }
    }
}

pub fn deploy_new_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: Addr,
    pauser: Addr,
    unpauser: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let instantiate_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: config.strategy_code_id,
        msg: to_json_binary(&StrategyInstantiateMsg {
            initial_owner: config.owner.to_string(),
            strategy_manager: config.strategy_manager.to_string(),
            underlying_token: token.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            initial_paused_status: 0,
        })?,
        funds: vec![],
        label: format!("Strategy for {}", token),
    };

    let sub_msg = SubMsg::reply_on_success(CosmosMsg::Wasm(instantiate_msg), 1);

    STRATEGIES.save(deps.storage, &token, &Addr::unchecked(""))?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("method", "create_strategy")
        .add_attribute("token_address", token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => handle_instantiate_reply(deps, msg),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let contract_address = extract_contract_address_from_reply(&msg)?;

    let token_address = STRATEGIES
        .keys(deps.storage, None, None, Order::Ascending)
        .last() 
        .ok_or(StdError::not_found("Token"))??;

    STRATEGIES.save(
        deps.storage,
        &token_address,
        &Addr::unchecked(contract_address.clone()),
    )?;

    Ok(Response::new()
        .add_attribute("method", "reply_instantiate")
        .add_attribute("new_strategy_address", contract_address)
        .add_attribute("token_address", token_address.to_string()))
}

fn extract_contract_address_from_reply(msg: &Reply) -> Result<String, ContractError> {
    let res = msg.result.clone().into_result().map_err(|e| {
        println!("InstantiateError: {:?}", e);
        ContractError::InstantiateError {}
    })?;    

    let data = res
        .msg_responses
        .get(0)
        .ok_or(ContractError::MissingInstantiateData {})?;

    let instantiate_response = cw_utils::parse_instantiate_response_data(&Binary::from(data.value.clone()))
        .map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse instantiate data")
        })?;

    Ok(instantiate_response.contract_address)
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,           
    strategy_code_id: u64,       
    only_owner_can_create: bool,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = new_owner; 
    config.strategy_code_id = strategy_code_id; 
    config.only_owner_can_create = only_owner_can_create; 

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetStrategy { token } => {
            let token_addr = deps.api.addr_validate(&token)?;
            to_json_binary(&query_strategy(deps, token_addr)?)
        }
    }
}

// fn query_config(deps: Deps) -> StdResult<Config> {
//     let config = CONFIG.load(deps.storage)?;
//     Ok(config)
// }

fn query_strategy(deps: Deps, token_address: Addr) -> StdResult<Addr> {
    let strategy = STRATEGIES.load(deps.storage, &token_address)?;
    Ok(strategy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_binary, Addr, Binary, DepsMut, Env, MessageInfo, Response, OwnedDeps, Uint128};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };

    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let initial_owner = deps.api.addr_make("initial_owner");
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            strategy_code_id: 1,
            only_owner_can_create: true,
            strategy_manager: strategy_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            initial_paused_status: 0,
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        deps
    }

    #[test]
    fn test_deploy_new_strategy() {
        let mut deps = setup_contract();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let token = deps.api.addr_make("token");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let msg = ExecuteMsg::CreateStrategy {
            token: token.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
        };

        let result = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(result.is_ok());
        let response: Response = result.unwrap();
    
        assert_eq!(response.messages.len(), 1);
        let msg = &response.messages[0];
        if let CosmosMsg::Wasm(WasmMsg::Instantiate { label, .. }) = &msg.msg {
            assert_eq!(*label, format!("Strategy for {}", token));
        } else {
            panic!("Expected WasmMsg::Instantiate");
        }
    
        let strategy_addr = STRATEGIES.load(&deps.storage, &Addr::unchecked(token)).unwrap();
        assert_eq!(strategy_addr, &Addr::unchecked(""));
    }
}