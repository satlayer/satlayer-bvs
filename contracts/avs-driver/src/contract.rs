use crate::{error::ContractError, msg::ExecuteMsg, msg::InstantiateMsg, msg::QueryMsg};

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
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
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ExecuteAvsOffchain { task_id } => execute_set(info, task_id),
    }
}

pub fn execute_set(info: MessageInfo, task_id: u64) -> StdResult<Response> {
    let event = Event::new("ExecuteAVSOffchain")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("taskId", task_id.to_string());

    Ok(Response::new()
        .add_attribute("method", "ExecuteAvsOffchain")
        .add_attribute("taskId", task_id.to_string())
        .add_event(event))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
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
    fn test_executeavsoffchain() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("alice"), &[]);
        let msg = ExecuteMsg::ExecuteAvsOffchain { task_id: 1000 };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(2, res.attributes.len());
        assert_eq!(("method", "ExecuteAvsOffchain"), res.attributes[0]);
        assert_eq!(("taskId", "1000"), res.attributes[1]);

        assert_eq!(1, res.events.len());
        assert_eq!("ExecuteAVSOffchain", res.events[0].ty);
        assert_eq!(
            vec![("sender", "alice"), ("taskId", "1000"),],
            res.events[0].attributes
        );
    }

    #[test]
    fn test_create_executeavsoffchain_msg() {
        let contract_addr = "contract123".to_string();
        let task_id = 100;

        let msg = ExecuteMsg::ExecuteAvsOffchain { task_id };

        let cosmos_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_json_binary(&msg).unwrap(),
            funds: vec![],
        });

        match cosmos_msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr,
                msg,
                funds,
            }) => {
                assert_eq!(addr, contract_addr);
                assert_eq!(funds, vec![]);

                let parsed_msg: ExecuteMsg = from_json(&msg).unwrap();
                match parsed_msg {
                    ExecuteMsg::ExecuteAvsOffchain { task_id: id } => {
                        assert_eq!(id, task_id);
                    }
                }
            }
            _ => panic!("Unexpected CosmosMsg type"),
        }
    }
}
