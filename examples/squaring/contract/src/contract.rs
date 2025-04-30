#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request { input } => execute::request(deps, info, input),
        ExecuteMsg::Respond { input, output } => execute::respond(deps, info, input, output),
        ExecuteMsg::Prove { input, operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::prove(deps, info, input, operator)
        }
    }
}

pub mod execute {
    use crate::state::REQUESTS;
    use crate::ContractError;
    use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response};

    pub fn request(
        _deps: DepsMut,
        info: MessageInfo,
        input: i64,
    ) -> Result<Response, ContractError> {
        Ok(Response::new()
            .add_attribute("method", "Request")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("input", input.to_string()))
    }

    pub fn respond(
        deps: DepsMut,
        info: MessageInfo,
        input: i64,
        output: i64,
    ) -> Result<Response, ContractError> {
        let operator = info.sender;
        if REQUESTS.has(deps.storage, (input, &operator)) {
            return Err(ContractError::Responded {});
        }

        REQUESTS.save(deps.storage, (input, &operator), &output)?;

        Ok(Response::new()
            .add_attribute("method", "Respond")
            .add_attribute("operator", operator.to_string())
            .add_attribute("input", input.to_string())
            .add_attribute("output", output.to_string()))
    }

    pub fn prove(
        deps: DepsMut,
        info: MessageInfo,
        input: i64,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetResponse { input, operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            Ok(to_json_binary(&query::get_response(
                deps, input, operator,
            )?)?)
        }
    }
}

pub mod query {
    use crate::state::REQUESTS;
    use crate::ContractError;
    use cosmwasm_std::{Addr, Deps};

    pub fn get_response(deps: Deps, input: i64, operator: Addr) -> Result<i64, ContractError> {
        let result = REQUESTS.may_load(deps.storage, (input, &operator))?;

        if let Some(input) = result {
            return Ok(input);
        }

        Err(ContractError::ResponseNotFound {})
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
        let res = execute::request(deps.as_mut(), aggregator_info, 42).unwrap();

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

            let res = execute::request(deps.as_mut(), sender_info, 69).unwrap();
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
        execute::respond(
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
        let error = execute::respond(deps.as_mut(), info, 1, 4761, "".to_string()).unwrap_err();

        assert_eq!(error, ContractError::Unauthorized {});
    }

    #[test]
    fn query_task_input() {
        let mut deps = mock_dependencies();

        let task_id = {
            let sender = deps.api.addr_make("random");
            let sender_info = message_info(&sender, &[]);

            let res = execute::request(deps.as_mut(), sender_info, 345678).unwrap();
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
