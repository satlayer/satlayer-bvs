use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{AGGREGATOR, CREATED_TASKS, MAX_ID, RESPONDED_TASKS},
};

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
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

    let response = Response::new().add_attribute("method", "instantiate");
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateNewTask { input } => execute::create_new_task(deps, info, input),
        ExecuteMsg::RespondToTask {
            task_id,
            result,
            operators,
        } => execute::respond_to_task(deps, info, task_id, result, operators),
    }
}

pub mod execute {
    use crate::state::{AGGREGATOR, CREATED_TASKS, MAX_ID, RESPONDED_TASKS};
    use crate::ContractError;
    use cosmwasm_std::{DepsMut, Event, MessageInfo, Response};

    pub fn create_new_task(
        deps: DepsMut,
        info: MessageInfo,
        input: i64,
    ) -> Result<Response, ContractError> {
        let id = MAX_ID.may_load(deps.storage)?;
        let new_id = id.unwrap_or(0) + 1;

        MAX_ID.save(deps.storage, &new_id)?;
        CREATED_TASKS.save(deps.storage, new_id, &input)?;

        let state_env = Event::new("UpdateState")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("key", format!("taskId.{}", new_id))
            .add_attribute("value", input.to_string());

        let offchain_env = Event::new("ExecuteBVSOffchain")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("task_id", new_id.to_string());

        let task_event = Event::new("NewTaskCreated")
            .add_attribute("taskId", new_id.to_string())
            .add_attribute("input", input.to_string());

        Ok(Response::new()
            .add_attribute("method", "CreateNewTask")
            .add_attribute("input", input.to_string())
            .add_attribute("taskId", new_id.to_string())
            .add_event(task_event)
            .add_event(state_env)
            .add_event(offchain_env))
    }

    pub fn respond_to_task(
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

        // .add_event(
        //         Event::new("UpdateState")
        //             .add_attribute("sender", sender.to_string())
        //             .add_attribute("key", key)
        //             .add_attribute("value", value.to_string()),
        //     )

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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetTaskInput { task_id } => query::task_input(deps, task_id),
        QueryMsg::GetTaskResult { task_id } => query::task_result(deps, task_id),
        QueryMsg::GetLatestTaskId {} => query::latest_task_id(deps),
    }
}

pub mod query {
    use crate::state::{CREATED_TASKS, MAX_ID, RESPONDED_TASKS};
    use crate::ContractError;
    use cosmwasm_std::{to_json_binary, Binary, Deps};

    pub fn task_input(deps: Deps, task_id: u64) -> Result<Binary, ContractError> {
        let result = CREATED_TASKS.may_load(deps.storage, task_id)?;

        if let Some(input) = result {
            return Ok(to_json_binary(&input)?);
        }

        Err(ContractError::NoValueFound {})
    }

    pub fn task_result(deps: Deps, task_id: u64) -> Result<Binary, ContractError> {
        let result = RESPONDED_TASKS.may_load(deps.storage, task_id)?;

        if let Some(result) = result {
            return Ok(to_json_binary(&result)?);
        }

        Err(ContractError::NoValueFound {})
    }

    pub fn latest_task_id(deps: Deps) -> Result<Binary, ContractError> {
        let result = MAX_ID.load(deps.storage)?;

        Ok(to_json_binary(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env},
        Addr, Coin, MessageInfo,
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
    }

    #[test]
    fn create_new_task() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a new task
        let create_msg = ExecuteMsg::CreateNewTask { input: 42 };
        let res = execute(deps.as_mut(), env, info, create_msg).unwrap();

        // Check if the task was created successfully
        assert_eq!(3, res.events.len());

        // Check if MAX_ID was incremented
        assert_eq!(MAX_ID.load(deps.as_ref().storage).unwrap(), 1);

        // Check if the task was saved
        assert_eq!(CREATED_TASKS.load(deps.as_ref().storage, 1).unwrap(), 42);
    }

    #[test]
    fn respond_to_task() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            aggregator: Addr::unchecked("aggregator"),
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
