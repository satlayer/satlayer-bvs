use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{BlacklistStatusResponse, StrategyResponse};
use crate::state::{Config, CONFIG, DEPLOYED_STRATEGIES, IS_BLACKLISTED};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;

use bvs_base::base::InstantiateMsg as StrategyInstantiateMsg;
use bvs_base::pausable::{only_when_not_paused, pause, unpause, PAUSED_STATE};
use bvs_base::roles::{check_pauser, check_unpauser, set_pauser, set_unpauser};
use bvs_base::strategy::ExecuteMsg as StrategyManagerExecuteMsg;

const CONTRACT_NAME: &str = "strategy-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAUSED_NEW_STRATEGIES: u8 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: deps.api.addr_validate(&msg.initial_owner)?,
        strategy_code_id: msg.strategy_code_id,
        strategy_manager: deps.api.addr_validate(&msg.strategy_manager)?,
        pauser: deps.api.addr_validate(&msg.pauser)?,
        unpauser: deps.api.addr_validate(&msg.unpauser)?,
    };

    CONFIG.save(deps.storage, &config)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let unpauser = deps.api.addr_validate(&msg.unpauser)?;

    set_pauser(deps.branch(), pauser)?;
    set_unpauser(deps.branch(), unpauser)?;

    PAUSED_STATE.save(deps.storage, &msg.initial_paused_status)?;

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
        ExecuteMsg::DeployNewStrategy {
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
        } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;

            update_config(deps, info, new_owner_addr, strategy_code_id)
        }
        ExecuteMsg::BlacklistTokens { tokens } => {
            let token_addrs = validate_addresses(deps.api, &tokens)?;
            blacklist_tokens(deps, info, token_addrs)
        }
        ExecuteMsg::RemoveStrategiesFromWhitelist { strategies } => {
            let strategies_addrs = validate_addresses(deps.api, &strategies)?;
            remove_strategies_from_whitelist(deps, info, strategies_addrs)
        }
        ExecuteMsg::SetThirdPartyTransfersForBidden { strategy, value } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            set_third_party_transfers_forbidden(deps, env, info, strategy_addr, value)
        }
        ExecuteMsg::WhitelistStrategies {
            strategies_to_whitelist,
            third_party_transfers_forbidden_values,
        } => {
            let strategies_to_whitelist_addr =
                validate_addresses(deps.api, &strategies_to_whitelist)?;
            whitelist_strategies(
                deps,
                info,
                strategies_to_whitelist_addr,
                third_party_transfers_forbidden_values,
            )
        }
        ExecuteMsg::SetStrategyManager {
            new_strategy_manager,
        } => {
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

pub fn deploy_new_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: Addr,
    pauser: Addr,
    unpauser: Addr,
) -> Result<Response, ContractError> {
    only_when_not_paused(deps.as_ref(), PAUSED_NEW_STRATEGIES)?;

    only_owner(deps.as_ref(), &info)?;

    let config = CONFIG.load(deps.storage)?;

    let is_blacklisted = IS_BLACKLISTED
        .may_load(deps.storage, &token)?
        .unwrap_or(false);
    if is_blacklisted {
        return Err(ContractError::TokenBlacklisted {});
    }

    let existing_strategy = DEPLOYED_STRATEGIES
        .may_load(deps.storage, &token)?
        .unwrap_or(Addr::unchecked(""));
    if existing_strategy != Addr::unchecked("") {
        return Err(ContractError::StrategyAlreadyExists {});
    }

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

    DEPLOYED_STRATEGIES.save(deps.storage, &token, &Addr::unchecked(""))?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("method", "create_strategy")
        .add_attribute("token_address", token))
}

pub fn blacklist_tokens(
    deps: DepsMut,
    info: MessageInfo,
    tokens: Vec<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    let mut strategies_to_remove: Vec<Addr> = Vec::new();

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
    }

    let mut response = Response::new()
        .add_attribute("method", "blacklist_tokens")
        .add_attribute("tokens_blacklisted", format!("{:?}", tokens));

    if !strategies_to_remove.is_empty() {
        let msg = StrategyManagerExecuteMsg::RemoveStrategiesFromWhitelist {
            strategies: strategies_to_remove.iter().map(|s| s.to_string()).collect(),
        };

        let exec_msg = WasmMsg::Execute {
            contract_addr: config.strategy_manager.to_string(),
            msg: to_json_binary(&msg)?,
            funds: vec![],
        };

        response = response.add_message(CosmosMsg::Wasm(exec_msg));
    }

    Ok(response)
}

pub fn remove_strategies_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    strategies: Vec<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    let msg = StrategyManagerExecuteMsg::RemoveStrategiesFromWhitelist {
        strategies: strategies.iter().map(|s| s.to_string()).collect(),
    };

    let exec_msg = WasmMsg::Execute {
        contract_addr: config.strategy_manager.to_string(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(exec_msg))
        .add_attribute("method", "remove_strategies_from_whitelist"))
}

pub fn set_third_party_transfers_forbidden(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy: Addr,
    value: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    let msg = StrategyManagerExecuteMsg::SetThirdPartyTransfersForbidden {
        strategy: strategy.to_string(),
        value,
    };

    let exec_msg = WasmMsg::Execute {
        contract_addr: config.strategy_manager.to_string(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(exec_msg))
        .add_attribute("method", "set_third_party_transfers_forbidden")
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("value", value.to_string()))
}

pub fn set_strategy_manager(
    deps: DepsMut,
    info: MessageInfo,
    new_strategy_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut config = CONFIG.load(deps.storage)?;

    config.strategy_manager = new_strategy_manager.clone();

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "set_strategy_manager")
        .add_attribute("new_strategy_manager", new_strategy_manager.to_string()))
}

pub fn whitelist_strategies(
    deps: DepsMut,
    info: MessageInfo,
    strategies_to_whitelist: Vec<Addr>,
    third_party_transfers_forbidden_values: Vec<bool>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    if strategies_to_whitelist.len() != third_party_transfers_forbidden_values.len() {
        return Err(ContractError::InvalidInput {});
    }

    let msg = StrategyManagerExecuteMsg::AddStrategiesToWhitelist {
        strategies: strategies_to_whitelist
            .iter()
            .map(|s| s.to_string())
            .collect(),
        third_party_transfers_forbidden_values,
    };

    let exec_msg = WasmMsg::Execute {
        contract_addr: config.strategy_manager.to_string(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(exec_msg))
        .add_attribute("method", "whitelist_strategies"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => handle_instantiate_reply(deps, msg),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

fn handle_instantiate_reply(mut deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let contract_address = extract_contract_address_from_reply(&msg, &mut deps)?;

    let token_address = DEPLOYED_STRATEGIES
        .keys(deps.storage, None, None, Order::Ascending)
        .last()
        .ok_or(StdError::not_found("Token"))??;

    let strategies_to_whitelist = [Addr::unchecked(contract_address.clone())];
    let third_party_transfers_forbidden_values = vec![false];

    let msg = StrategyManagerExecuteMsg::AddStrategiesToWhitelist {
        strategies: strategies_to_whitelist
            .iter()
            .map(|s| s.to_string())
            .collect(),
        third_party_transfers_forbidden_values,
    };

    let exec_msg = WasmMsg::Execute {
        contract_addr: config.strategy_manager.to_string(),
        msg: to_json_binary(&msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("method", "reply_instantiate")
        .add_attribute("new_strategy_address", contract_address)
        .add_attribute("token_address", token_address.to_string())
        .add_message(CosmosMsg::Wasm(exec_msg)))
}

fn extract_contract_address_from_reply(
    msg: &Reply,
    deps: &mut DepsMut,
) -> Result<String, ContractError> {
    let res = msg.result.clone().into_result().map_err(|e| {
        println!("InstantiateError: {:?}", e);
        ContractError::InstantiateError {}
    })?;

    let data = res
        .msg_responses
        .first()
        .ok_or(ContractError::MissingInstantiateData {})?;

    let instantiate_response = cw_utils::parse_instantiate_response_data(&data.value.clone())
        .map_err(|_| {
            StdError::parse_err(
                "MsgInstantiateContractResponse",
                "failed to parse instantiate data",
            )
        })?;

    let contract_address = instantiate_response.contract_address.clone();

    let token_address = DEPLOYED_STRATEGIES
        .keys(deps.storage, None, None, Order::Ascending)
        .last()
        .ok_or(StdError::not_found("Token"))??;

    set_strategy_for_token(
        deps,
        Addr::unchecked(token_address.clone()),
        Addr::unchecked(contract_address.clone()),
    )?;

    Ok(instantiate_response.contract_address)
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
    strategy_code_id: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = new_owner;
    config.strategy_code_id = strategy_code_id;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    only_owner(deps.as_ref(), &info)?;

    config.owner = new_owner.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStrategy { token } => {
            let token_addr = deps.api.addr_validate(&token)?;
            to_json_binary(&query_strategy(deps, token_addr)?)
        }
        QueryMsg::IsTokenBlacklisted { token } => {
            let token_addr = deps.api.addr_validate(&token)?;
            to_json_binary(&query_blacklist_status(deps, token_addr)?)
        }
    }
}

fn query_strategy(deps: Deps, token: Addr) -> StdResult<StrategyResponse> {
    let strategy = DEPLOYED_STRATEGIES.load(deps.storage, &token)?;
    Ok(StrategyResponse { strategy })
}

fn query_blacklist_status(deps: Deps, token: Addr) -> StdResult<BlacklistStatusResponse> {
    let is_blacklisted = IS_BLACKLISTED
        .may_load(deps.storage, &token)?
        .unwrap_or(false);
    Ok(BlacklistStatusResponse { is_blacklisted })
}

fn validate_addresses(api: &dyn Api, addresses: &[String]) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}

fn set_strategy_for_token(
    deps: &mut DepsMut,
    token: Addr,
    strategy: Addr,
) -> Result<Response, ContractError> {
    DEPLOYED_STRATEGIES.save(deps.storage, &token, &strategy)?;

    Ok(Response::new()
        .add_attribute("method", "set_strategy_for_token")
        .add_attribute("token", token.to_string())
        .add_attribute("strategy", strategy.to_string()))
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, Addr, Binary, ContractResult, MessageInfo, OwnedDeps, Response, SystemError,
        SystemResult, WasmQuery,
    };

    fn setup_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Addr,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();

        let initial_owner = deps.api.addr_make("initial_owner");

        let info = message_info(&initial_owner, &[]);
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            strategy_code_id: 1,
            strategy_manager: strategy_manager.to_string(),
            pauser: pauser.to_string(),
            unpauser: unpauser.to_string(),
            initial_paused_status: 0,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        (deps, initial_owner, info)
    }

    #[test]
    fn test_deploy_new_strategy() {
        let (mut deps, _initial_owner, info) = setup_contract();
        let token = deps.api.addr_make("token");
        let pauser = deps.api.addr_make("pauser");
        let unpauser = deps.api.addr_make("unpauser");

        let msg = ExecuteMsg::DeployNewStrategy {
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

        let strategy_addr = DEPLOYED_STRATEGIES
            .load(&deps.storage, &Addr::unchecked(token))
            .unwrap();
        assert_eq!(strategy_addr, &Addr::unchecked(""));
    }

    #[test]
    fn test_blacklist_tokens() {
        let (mut deps, initial_owner, _) = setup_contract();

        let info = message_info(&initial_owner, &[]);

        let token1 = deps.api.addr_make("token1_address");
        let token2 = deps.api.addr_make("token2_address");

        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        DEPLOYED_STRATEGIES
            .save(deps.as_mut().storage, &token1, &strategy1)
            .unwrap();
        DEPLOYED_STRATEGIES
            .save(deps.as_mut().storage, &token2, &strategy2)
            .unwrap();

        deps.querier.update_wasm(|query| match query {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
                ..
            } => {
                if contract_addr == "strategy_manager" {
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&"RemoveStrategiesFromWhitelist executed successfully")
                            .unwrap(),
                    ))
                } else {
                    SystemResult::Err(SystemError::InvalidResponse {
                        error: "unknown address".to_string(),
                        response: Binary::default(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::InvalidResponse {
                error: "unsupported query".to_string(),
                response: Binary::default(),
            }),
        });

        let msg = ExecuteMsg::BlacklistTokens {
            tokens: vec![token1.to_string(), token2.to_string()],
        };

        let result = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(result.is_ok());
        let response: Response = result.unwrap();

        assert_eq!(response.messages.len(), 1);
        let msg = &response.messages[0];

        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr, msg, ..
        }) = &msg.msg
        {
            assert_eq!(
                contract_addr.to_string(),
                deps.api.addr_make("strategy_manager").to_string()
            );

            let remove_msg: StrategyManagerExecuteMsg = from_json(msg).unwrap();
            if let StrategyManagerExecuteMsg::RemoveStrategiesFromWhitelist { strategies } =
                remove_msg
            {
                assert_eq!(
                    strategies,
                    vec![strategy1.to_string(), strategy2.to_string()]
                );
            } else {
                panic!("Expected RemoveStrategiesFromWhitelist message");
            }
        } else {
            panic!("Expected WasmMsg::Execute");
        }

        let is_blacklisted1 = IS_BLACKLISTED.load(deps.as_mut().storage, &token1).unwrap();
        let is_blacklisted2 = IS_BLACKLISTED.load(deps.as_mut().storage, &token2).unwrap();
        assert!(is_blacklisted1);
        assert!(is_blacklisted2);
    }

    #[test]
    fn test_set_strategy_manager() {
        let (mut deps, initial_owner, _) = setup_contract();

        let info = message_info(&initial_owner, &[]);

        let new_strategy_manager = deps.api.addr_make("new_strategy_manager");

        let msg = ExecuteMsg::SetStrategyManager {
            new_strategy_manager: new_strategy_manager.to_string(),
        };

        let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(result.is_ok());
        let response: Response = result.unwrap();

        assert_eq!(response.attributes.len(), 2);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(response.attributes[0].value, "set_strategy_manager");
        assert_eq!(response.attributes[1].key, "new_strategy_manager");
    }

    #[test]
    fn test_update_config() {
        let (mut deps, initial_owner, _) = setup_contract();

        let info = message_info(&initial_owner, &[]);

        let new_owner = deps.api.addr_make("new_owner");
        let new_strategy_code_id = 2;

        let msg = ExecuteMsg::UpdateConfig {
            new_owner: new_owner.to_string(),
            strategy_code_id: new_strategy_code_id,
        };

        let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(result.is_ok());
        let response: Response = result.unwrap();

        assert_eq!(response.attributes.len(), 1);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(response.attributes[0].value, "update_config");

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.owner, new_owner);
        assert_eq!(config.strategy_code_id, new_strategy_code_id);

        let unauthorized_user = message_info(&Addr::unchecked("unauthorized_user"), &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: new_owner.to_string(),
            strategy_code_id: new_strategy_code_id,
        };

        let result = execute(deps.as_mut(), mock_env(), unauthorized_user, msg);
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
        let (mut deps, initial_owner, _) = setup_contract();

        let info = message_info(&initial_owner, &[]);

        let new_owner = deps.api.addr_make("new_owner");

        let msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.to_string(),
        };

        let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(result.is_ok());
        let response: Response = result.unwrap();

        assert_eq!(response.attributes.len(), 2);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(response.attributes[0].value, "transfer_ownership");
        assert_eq!(response.attributes[1].key, "new_owner");
        assert_eq!(response.attributes[1].value, new_owner.to_string());

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.owner, new_owner);

        let unauthorized_user = message_info(&deps.api.addr_make("unauthorized_user"), &[]);
        let msg = ExecuteMsg::TransferOwnership {
            new_owner: deps.api.addr_make("another_owner").to_string(),
        };

        let result = execute(deps.as_mut(), mock_env(), unauthorized_user, msg);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_query_blacklist_status() {
        let (mut deps, _initial_owner, _) = setup_contract();

        let token1 = deps.api.addr_make("token1_address");
        let token2 = deps.api.addr_make("token2_address");

        IS_BLACKLISTED
            .save(deps.as_mut().storage, &token1, &true)
            .unwrap();

        IS_BLACKLISTED
            .save(deps.as_mut().storage, &token2, &false)
            .unwrap();

        let msg = QueryMsg::IsTokenBlacklisted {
            token: token1.to_string(),
        };

        let bin = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: BlacklistStatusResponse = from_json(&bin).unwrap();

        assert!(res.is_blacklisted);

        let msg = QueryMsg::IsTokenBlacklisted {
            token: token2.to_string(),
        };

        let bin = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: BlacklistStatusResponse = from_json(&bin).unwrap();

        assert!(!res.is_blacklisted);
    }

    #[test]
    fn test_query_strategy_combined() {
        let (mut deps, _initial_owner, _) = setup_contract();

        let token = deps.api.addr_make("token_address");
        let strategy = deps.api.addr_make("strategy_address");

        DEPLOYED_STRATEGIES
            .save(deps.as_mut().storage, &token, &strategy)
            .unwrap();

        let msg = QueryMsg::GetStrategy {
            token: token.to_string(),
        };

        let bin = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: StrategyResponse = from_json(&bin).unwrap();

        assert_eq!(res.strategy, strategy);

        let token_not_found = deps.api.addr_make("non_existing_token");

        let msg = QueryMsg::GetStrategy {
            token: token_not_found.to_string(),
        };

        let result = query(deps.as_ref(), mock_env(), msg);

        assert!(result.is_err());
        if let Err(StdError::NotFound { .. }) = result {
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_set_strategy_for_token() {
        let (mut deps, _initial_owner, _) = setup_contract();

        let token = deps.api.addr_make("token_address");
        let strategy = deps.api.addr_make("strategy_address");

        let result = set_strategy_for_token(&mut deps.as_mut(), token.clone(), strategy.clone());

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.attributes.len(), 3);
        assert_eq!(response.attributes[0].key, "method");
        assert_eq!(response.attributes[0].value, "set_strategy_for_token");
        assert_eq!(response.attributes[1].key, "token");
        assert_eq!(response.attributes[1].value, token.to_string());
        assert_eq!(response.attributes[2].key, "strategy");
        assert_eq!(response.attributes[2].value, strategy.to_string());

        let stored_strategy = DEPLOYED_STRATEGIES.load(&deps.storage, &token).unwrap();
        assert_eq!(stored_strategy, strategy);
    }
}
