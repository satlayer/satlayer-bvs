use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::ValueResponse,
    state::{IS_AVS_CONTRACT_REGISTERED, VALUES},
};

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult,
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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Set { key, value } => execute_set(deps, info, key, value),
        ExecuteMsg::AddRegisteredAvsContract { address } => {
            add_registered_avs_contract(deps, info, Addr::unchecked(address))
        }
    }
}

pub fn execute_set(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    value: i64,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let is_registered = IS_AVS_CONTRACT_REGISTERED
        .may_load(deps.storage, &sender)?
        .unwrap_or(false);

    if !is_registered {
        return Err(ContractError::AvsContractNotRegistered {});
    }

    VALUES.save(deps.storage, key.clone(), &value)?;

    Ok(Response::new().add_event(
        Event::new("execute_set")
            .add_attribute("sender", sender.to_string())
            .add_attribute("key", key)
            .add_attribute("value", value.to_string()),
    ))
}

pub fn add_registered_avs_contract(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    IS_AVS_CONTRACT_REGISTERED.save(deps.storage, &Addr::unchecked(address.clone()), &true)?;

    Ok(Response::new().add_event(
        Event::new("add_registered_avs_contract")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("address", address.to_string()),
    ))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Get { key } => query_value(deps, key),
    }
}

fn query_value(deps: Deps, key: String) -> StdResult<Binary> {
    let result = VALUES.may_load(deps.storage, key)?;

    if let Some(value) = result {
        return to_json_binary(&ValueResponse { value });
    }

    Err(StdError::generic_err("Value not found"))
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

        let admin_info = message_info(&Addr::unchecked("admin"), &[]);
        let register_msg = ExecuteMsg::AddRegisteredAvsContract {
            address: "alice".to_string(),
        };
        execute(deps.as_mut(), env.clone(), admin_info, register_msg).unwrap();

        let info = message_info(&Addr::unchecked("alice"), &[]);
        let msg = ExecuteMsg::Set {
            key: "temperature".to_string(),
            value: 25,
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(1, res.events.len());
        assert_eq!("execute_set", res.events[0].ty);
        assert_eq!(
            vec![("sender", "alice"), ("key", "temperature"), ("value", "25"),],
            res.events[0].attributes
        );

        let query_msg = QueryMsg::Get {
            key: "temperature".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        println!("value {}", res);
        let res: ValueResponse = from_json(res).unwrap();
        assert_eq!(25, res.value);

        let query_msg = QueryMsg::Get {
            key: "non_existent".to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg);
        assert!(res.is_err());

        let unregistered_info = message_info(&Addr::unchecked("bob"), &[]);
        let unregistered_msg = ExecuteMsg::Set {
            key: "pressure".to_string(),
            value: 1013,
        };
        let unregistered_res = execute(deps.as_mut(), env, unregistered_info, unregistered_msg);
        assert!(unregistered_res.is_err());
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
                    _ => panic!("Unexpected ExecuteMsg type"),
                }
            }
            _ => panic!("Unexpected CosmosMsg type"),
        }
    }

    #[test]
    fn test_add_registered_avs_contract() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("admin"), &[]);
        let avs_contract_address = "avs_contract_123";

        let msg = ExecuteMsg::AddRegisteredAvsContract {
            address: avs_contract_address.to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(1, res.events.len());
        assert_eq!("add_registered_avs_contract", res.events[0].ty);
        assert_eq!(
            vec![("sender", "admin"), ("address", avs_contract_address),],
            res.events[0].attributes
        );

        let is_registered = IS_AVS_CONTRACT_REGISTERED
            .may_load(
                deps.as_ref().storage,
                &Addr::unchecked(avs_contract_address),
            )
            .unwrap()
            .unwrap_or(false);
        assert!(is_registered);

        let set_msg = ExecuteMsg::Set {
            key: "temperature".to_string(),
            value: 25,
        };
        let set_info = message_info(&Addr::unchecked(avs_contract_address), &[]);
        let set_res = execute(deps.as_mut(), env, set_info, set_msg).unwrap();

        assert_eq!(1, set_res.events.len());
        assert_eq!("execute_set", set_res.events[0].ty);
        assert_eq!(
            vec![
                ("sender", avs_contract_address),
                ("key", "temperature"),
                ("value", "25"),
            ],
            set_res.events[0].attributes
        );
    }
}
