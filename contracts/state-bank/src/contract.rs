use crate::{
    error::ContractError, msg::ExecuteMsg, msg::InstantiateMsg, msg::QueryMsg, state::VALUES,
};

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let response = Response::new().add_attribute("method", "instantiate");

    Ok(response)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Set { key, value } => execute_set(deps, info, key, value),
    }
}

pub fn execute_set(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    value: i64,
) -> StdResult<Response> {
    VALUES.save(deps.storage, key.clone(), &value)?;

    let event = Event::new("UpdateState")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("key", key.clone())
        .add_attribute("value", value.to_string());

    Ok(Response::new()
        .add_attribute("method", "set")
        .add_attribute("key", key)
        .add_attribute("value", value.to_string())
        .add_event(event))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Get { key } => query_value(deps, key),
    }
}

fn query_value(deps: Deps, key: String) -> StdResult<Binary> {
    let value = VALUES.may_load(deps.storage, key)?;
    to_json_binary(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies, mock_env},
        to_json_binary, Addr, CosmosMsg, WasmMsg,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let msg = InstantiateMsg {};

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.attributes.len());
        assert_eq!(("method", "instantiate"), res.attributes[0]);
    }

    #[test]
    fn test_set_and_get() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("alice"), &[]);
        let msg = ExecuteMsg::Set {
            key: "temperature".to_string(),
            value: 25,
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(3, res.attributes.len());
        assert_eq!(("method", "set"), res.attributes[0]);
        assert_eq!(("key", "temperature"), res.attributes[1]);
        assert_eq!(("value", "25"), res.attributes[2]);

        assert_eq!(1, res.events.len());
        assert_eq!("UpdateState", res.events[0].ty);
        assert_eq!(
            vec![("sender", "alice"), ("key", "temperature"), ("value", "25"),],
            res.events[0].attributes
        );

        let query_msg = QueryMsg::Get {
            key: "temperature".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let value: Option<i32> = from_json(res).unwrap();
        assert_eq!(Some(25), value);

        let query_msg = QueryMsg::Get {
            key: "non_existent".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let value: Option<i32> = from_json(res).unwrap();
        assert_eq!(None, value);
    }

    fn _create_set_msg(contract_addr: String, key: String, value: i64) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::Set { key, value };
        let wasm_msg = WasmMsg::Execute {
            contract_addr,
            msg: to_json_binary(&msg)?,
            funds: vec![],
        };
        Ok(CosmosMsg::Wasm(wasm_msg))
    }

    #[test]
    fn test_create_set_msg() {
        let contract_addr = "contract123".to_string();
        let key = "pressure".to_string();
        let value = 1013;

        let cosmos_msg = _create_set_msg(contract_addr.clone(), key.clone(), value).unwrap();

        match cosmos_msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr,
                msg,
                funds,
            }) => {
                assert_eq!(addr, contract_addr);
                assert_eq!(funds, vec![]);

                let parsed_msg: ExecuteMsg = from_json(msg).unwrap();
                match parsed_msg {
                    ExecuteMsg::Set { key: k, value: v } => {
                        assert_eq!(k, key);
                        assert_eq!(v, value);
                    }
                }
            }
            _ => panic!("Unexpected CosmosMsg type"),
        }
    }
}
