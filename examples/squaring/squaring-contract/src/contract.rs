use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state,
};

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let aggregator = deps.api.addr_validate(&msg.aggregator)?;
    state::AGGREGATOR.save(deps.storage, &aggregator)?;

    state::MAX_ID.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("aggregator", aggregator))
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
    use std::ops::Add;

    pub fn create_new_task(
        deps: DepsMut,
        info: MessageInfo,
        input: i64,
    ) -> Result<Response, ContractError> {
        let new_id = MAX_ID.may_load(deps.storage)?.unwrap_or(0).add(1);

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
        if responded_result.is_some() {
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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetTaskInput { task_id } => {
            Ok(to_json_binary(&query::task_input(deps, task_id)?)?)
        }
        QueryMsg::GetTaskResult { task_id } => {
            Ok(to_json_binary(&query::task_result(deps, task_id)?)?)
        }
        QueryMsg::GetLatestTaskId {} => Ok(to_json_binary(&query::latest_task_id(deps)?)?),
    }
}

pub mod query {
    use crate::state::{CREATED_TASKS, MAX_ID, RESPONDED_TASKS};
    use crate::ContractError;
    use cosmwasm_std::{Deps, StdResult};

    pub fn task_input(deps: Deps, task_id: u64) -> Result<i64, ContractError> {
        let result = CREATED_TASKS.may_load(deps.storage, task_id)?;

        if let Some(input) = result {
            return Ok(input);
        }

        Err(ContractError::NoValueFound {})
    }

    pub fn task_result(deps: Deps, task_id: u64) -> Result<i64, ContractError> {
        let result = RESPONDED_TASKS.may_load(deps.storage, task_id)?;

        if let Some(result) = result {
            return Ok(result);
        }

        Err(ContractError::NoValueFound {})
    }

    pub fn latest_task_id(deps: Deps) -> StdResult<u64> {
        let result = MAX_ID.load(deps.storage)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let sender_info = message_info(&sender, &[]);
        let aggregator = deps.api.addr_make("aggregator");
        let msg = InstantiateMsg {
            aggregator: aggregator.to_string(),
        };

        let res = instantiate(deps.as_mut(), env, sender_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_eq!(version.version, CONTRACT_VERSION);

        // Check if the state is properly initialized
        assert_eq!(state::MAX_ID.load(deps.as_ref().storage).unwrap(), 0);
        assert_eq!(
            state::AGGREGATOR.load(deps.as_ref().storage).unwrap(),
            aggregator,
        );
    }

    #[test]
    fn create_new_task() {
        let mut deps = mock_dependencies();

        let caller = deps.api.addr_make("anyone");
        let aggregator_info = message_info(&caller, &[]);
        let res = execute::create_new_task(deps.as_mut(), aggregator_info, 42).unwrap();

        assert_eq!(3, res.events.len());

        // Check if MAX_ID was incremented
        assert_eq!(state::MAX_ID.load(deps.as_ref().storage).unwrap(), 1);

        // Check if the task was saved
        assert_eq!(
            state::CREATED_TASKS.load(deps.as_ref().storage, 1).unwrap(),
            42
        );
    }

    #[test]
    fn respond_to_task() {
        let mut deps = mock_dependencies();

        let aggregator = deps.api.addr_make("aggregator");
        state::AGGREGATOR
            .save(&mut deps.storage, &aggregator)
            .unwrap();

        let task_id = {
            let sender = deps.api.addr_make("random");
            let sender_info = message_info(&sender, &[]);

            let res = execute::create_new_task(deps.as_mut(), sender_info, 69).unwrap();
            let task_id = &res
                .events
                .iter()
                .find(|event| event.ty == "NewTaskCreated")
                .unwrap()
                .attributes
                .iter()
                .find(|attr| attr.key == "taskId")
                .unwrap()
                .value;

            task_id.parse::<u64>().unwrap()
        };

        let aggregator_info = message_info(&aggregator, &[]);
        execute::respond_to_task(
            deps.as_mut(),
            aggregator_info,
            task_id,
            4761,
            "operator1".to_string(),
        )
        .unwrap();

        assert_eq!(
            state::RESPONDED_TASKS
                .load(deps.as_ref().storage, 1)
                .unwrap(),
            4761
        );
    }

    #[test]
    fn respond_to_task_unauthorized() {
        let mut deps = mock_dependencies();

        {
            let aggregator = deps.api.addr_make("aggregator");

            state::AGGREGATOR
                .save(&mut deps.storage, &aggregator)
                .unwrap();
        }

        let not_aggregator = deps.api.addr_make("not_aggregator");
        let info = message_info(&not_aggregator, &[]);
        let error =
            execute::respond_to_task(deps.as_mut(), info, 1, 4761, "".to_string()).unwrap_err();

        assert_eq!(error, ContractError::Unauthorized {});
    }

    #[test]
    fn query_task_input() {
        let mut deps = mock_dependencies();

        let task_id = {
            let sender = deps.api.addr_make("random");
            let sender_info = message_info(&sender, &[]);

            let res = execute::create_new_task(deps.as_mut(), sender_info, 345678).unwrap();
            let task_id = &res
                .events
                .iter()
                .find(|event| event.ty == "NewTaskCreated")
                .unwrap()
                .attributes
                .iter()
                .find(|attr| attr.key == "taskId")
                .unwrap()
                .value;

            task_id.parse::<u64>().unwrap()
        };

        // Query the task input
        let value = query::task_input(deps.as_ref(), task_id).unwrap();
        assert_eq!(value, 345678);
    }
}
