use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{AGGREGATOR, BVS_DRIVER, CREATED_TASKS, MAX_ID, RESPONDED_TASKS, STATE_BANK},
};

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Response, WasmMsg,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "BVS Squaring Example";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    MAX_ID.save(deps.storage, &0)?;
    AGGREGATOR.save(deps.storage, &msg.aggregator)?;
    STATE_BANK.save(deps.storage, &msg.state_bank)?;
    BVS_DRIVER.save(deps.storage, &msg.bvs_driver)?;

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
        ExecuteMsg::CreateNewTask { input } => create_new_task(deps, info, input),
        ExecuteMsg::RespondToTask {
            task_id,
            result,
            operators,
        } => respond_to_task(deps, info, task_id, result, operators),
        _ => Ok(Response::default()),
    }
}

fn create_new_task(
    deps: DepsMut,
    _info: MessageInfo,
    input: i64,
) -> Result<Response, ContractError> {
    let id = MAX_ID.may_load(deps.storage)?;
    let new_id = id.unwrap_or(0) + 1;

    MAX_ID.save(deps.storage, &new_id)?;

    CREATED_TASKS.save(deps.storage, new_id, &input)?;

    // Call the state bank contract to save the task input
    let state_bank_msg = {
        let msg = ExecuteMsg::Set {
            key: format!("taskId.{}", new_id),
            value: input.to_string(),
        };

        let state_bank_address = STATE_BANK.load(deps.storage)?;
        let wasm_msg = WasmMsg::Execute {
            contract_addr: state_bank_address.into_string(),
            msg: to_json_binary(&msg)?,
            funds: vec![],
        };

        CosmosMsg::Wasm(wasm_msg)
    };

    // Call the bvs driver contract to execute the task off-chain
    let bvs_driver_msg = {
        let msg = ExecuteMsg::ExecuteBvsOffchain {
            task_id: new_id.to_string(),
        };

        let bvs_driver_address = BVS_DRIVER.load(deps.storage)?;
        let wasm_msg = WasmMsg::Execute {
            contract_addr: bvs_driver_address.into_string(),
            msg: to_json_binary(&msg)?,
            funds: vec![],
        };

        CosmosMsg::Wasm(wasm_msg)
    };

    // emit event
    let event = Event::new("NewTaskCreated")
        .add_attribute("taskId", new_id.to_string())
        .add_attribute("input", input.to_string());

    Ok(Response::new()
        .add_message(state_bank_msg)
        .add_message(bvs_driver_msg)
        .add_attribute("method", "CreateNewTask")
        .add_attribute("input", input.to_string())
        .add_attribute("taskId", new_id.to_string())
        .add_event(event))
}

fn respond_to_task(
    deps: DepsMut,
    info: MessageInfo,
    task_id: u64,
    result: i64,
    operators: String,
) -> Result<Response, ContractError> {
    let aggregator = AGGREGATOR.load(deps.storage)?;

    // only aggregator can respond to task
    if info.sender != aggregator {
        return Err(ContractError::Unauthorized {});
    }

    let responded_result = RESPONDED_TASKS.may_load(deps.storage, task_id)?;

    // result already submitted
    if let Some(_) = responded_result {
        return Err(ContractError::ResultSubmitted {});
    }

    // save task result
    RESPONDED_TASKS.save(deps.storage, task_id, &result)?;

    // emit event
    let event = Event::new("TaskResponded")
        .add_attribute("taskId", task_id.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("operators", operators);

    Ok(Response::new()
        .add_attribute("method", "RespondToTask")
        .add_attribute("taskId", task_id.to_string())
        .add_attribute("result", result.to_string())
        .add_event(event))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetTaskInput { task_id } => query_task_input(deps, task_id),
        QueryMsg::GetTaskResult { task_id } => query_task_result(deps, task_id),
        QueryMsg::GetLatestTaskId {} => query_latest_task_id(deps),
    }
}

fn query_task_input(deps: Deps, task_id: u64) -> Result<Binary, ContractError> {
    let result = CREATED_TASKS.may_load(deps.storage, task_id)?;

    if let Some(input) = result {
        return Ok(to_json_binary(&input)?);
    }

    Err(ContractError::NoValueFound {})
}

fn query_task_result(deps: Deps, task_id: u64) -> Result<Binary, ContractError> {
    let result = RESPONDED_TASKS.may_load(deps.storage, task_id)?;

    if let Some(result) = result {
        return Ok(to_json_binary(&result)?);
    }

    Err(ContractError::NoValueFound {})
}

fn query_latest_task_id(deps: Deps) -> Result<Binary, ContractError> {
    let result = MAX_ID.load(deps.storage)?;

    Ok(to_json_binary(&result)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env},
        Addr, Coin, CosmosMsg, MessageInfo, WasmMsg,
    };

    fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
        MessageInfo {
            sender: Addr::unchecked(sender),
            funds: funds.to_vec(),
        }
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Check if the contract version is set
        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_eq!(version.version, CONTRACT_VERSION);

        // Check if the state is properly initialized
        assert_eq!(MAX_ID.load(deps.as_ref().storage).unwrap(), 0);
        assert_eq!(
            AGGREGATOR.load(deps.as_ref().storage).unwrap(),
            Addr::unchecked("aggregator")
        );
        assert_eq!(
            STATE_BANK.load(deps.as_ref().storage).unwrap(),
            Addr::unchecked("state_bank")
        );
        assert_eq!(
            BVS_DRIVER.load(deps.as_ref().storage).unwrap(),
            Addr::unchecked("bvs_driver")
        );
    }

    #[test]
    fn test_create_new_task() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        let res = execute(deps.as_mut(), env, info, create_msg).unwrap();

        // Check if the task was created successfully
        assert_eq!(2, res.messages.len());
        assert_eq!(1, res.events.len());

        // Check if MAX_ID was incremented
        assert_eq!(MAX_ID.load(deps.as_ref().storage).unwrap(), 1);

        // Check if the task was saved
        assert_eq!(CREATED_TASKS.load(deps.as_ref().storage, 1).unwrap(), 42);

        // Check if the correct messages were created
        let state_bank_msg = &res.messages[0];
        let bvs_driver_msg = &res.messages[1];

        match &state_bank_msg.msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr, msg, ..
            }) => {
                assert_eq!(contract_addr, "state_bank");
                let parsed_msg: ExecuteMsg = from_json(msg).unwrap();
                match parsed_msg {
                    ExecuteMsg::Set { key, value } => {
                        assert_eq!(key, "taskId.1");
                        assert_eq!(value, "42");
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }

        match &bvs_driver_msg.msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr, msg, ..
            }) => {
                assert_eq!(contract_addr, "bvs_driver");
                let parsed_msg: ExecuteMsg = from_json(msg).unwrap();
                match parsed_msg {
                    ExecuteMsg::ExecuteBvsOffchain { task_id } => {
                        assert_eq!(task_id, "1");
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_respond_to_task() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        execute(deps.as_mut(), env.clone(), info, create_msg).unwrap();

        // Respond to the task
        let respond_msg = ExecuteMsg::RespondToTask {
            task_id: 1,
            result: 84,
            operators: "operator1".to_string(),
        };
        let aggregator_info = mock_info("aggregator", &[]);
        let res = execute(deps.as_mut(), env, aggregator_info, respond_msg).unwrap();

        // Check if the response was saved successfully
        assert_eq!(0, res.messages.len());
        assert_eq!(1, res.events.len());

        // Check if the task result was saved
        assert_eq!(RESPONDED_TASKS.load(deps.as_ref().storage, 1).unwrap(), 84);
    }

    #[test]
    fn respond_to_task_unauthorized() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        execute(deps.as_mut(), env.clone(), info, create_msg).unwrap();

        // Try to respond to the task with an unauthorized address
        let respond_msg = ExecuteMsg::RespondToTask {
            task_id: 1,
            result: 84,
            operators: "operator1".to_string(),
        };
        let unauthorized_info = mock_info("unauthorized", &[]);
        let res = execute(deps.as_mut(), env, unauthorized_info, respond_msg);

        // Check if the response was rejected
        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::Unauthorized {} => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn query_task_input() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        execute(deps.as_mut(), env.clone(), info, create_msg).unwrap();

        // Query the task input
        let query_msg = QueryMsg::GetTaskInput { task_id: 1 };
        let res = query(deps.as_ref(), env, query_msg).unwrap();
        let value: i64 = from_json(&res).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn query_task_result() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
            state_bank: Addr::unchecked("state_bank"),
            bvs_driver: Addr::unchecked("bvs_driver"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        execute(deps.as_mut(), env.clone(), info, create_msg).unwrap();

        // Respond to the task
        let respond_msg = ExecuteMsg::RespondToTask {
            task_id: 1,
            result: 84,
            operators: "operator1".to_string(),
        };
        let aggregator_info = mock_info("aggregator", &[]);
        execute(deps.as_mut(), env.clone(), aggregator_info, respond_msg).unwrap();

        // Query the task result
        let query_msg = QueryMsg::GetTaskResult { task_id: 1 };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let value: i64 = from_json(&res).unwrap();
        assert_eq!(value, 84);

        // Query the latest task ID
        let query_msg = QueryMsg::GetLatestTaskId {};
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let value: u64 = from_json(&res).unwrap();
        assert_eq!(value, 1);
    }
}
