#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

const CONTRACT_NAME: &str = "crates.io:bvs-squaring";
const CONTRACT_VERSION: &str = "0.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        router: deps.api.addr_validate(&msg.router)?,
        registry: deps.api.addr_validate(&msg.registry)?,
        owner: deps.api.addr_validate(&msg.owner)?,
    };
    CONFIG.save(deps.storage, &config)?;

    // Register this contract as a Service in BVS Registry
    let register_as_service: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
        contract_addr: msg.registry,
        msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::RegisterAsService {
            // Metadata of the service
            metadata: bvs_registry::msg::Metadata {
                name: Some("The Squaring Company".to_string()),
                uri: Some("https://the-squaring-company.com".to_string()),
            },
        })?,
        funds: vec![],
    }
    .into();

    Ok(Response::new()
        .add_message(register_as_service)
        .add_attribute("method", "instantiate")
        .add_attribute("registry", config.registry)
        .add_attribute("router", config.router)
        .add_attribute("owner", config.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request { input } => execute::request(deps, env, info, input),
        ExecuteMsg::Respond { input, output } => execute::respond(deps, env, info, input, output),
        ExecuteMsg::Compute { input, operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::compute(deps, env, info, input, operator)
        }
        ExecuteMsg::SlashingCancel { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_cancel(deps, env, info, operator)
        }
        ExecuteMsg::SlashingLock { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_lock(deps, env, info, operator)
        }
        ExecuteMsg::SlashingFinalize { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_finalize(deps, env, info, operator)
        }
        ExecuteMsg::RegisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::register_operator(deps, env, info, operator)
        }
        ExecuteMsg::DeregisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::deregister_operator(deps, env, info, operator)
        }
        ExecuteMsg::EnableSlashing {} => execute::enable_slashing(deps, env, info),
        ExecuteMsg::DisableSlashing {} => execute::disable_slashing(deps, env, info),
    }
}

pub mod execute {
    use crate::contract::expensive_computation;
    use crate::state::{CONFIG, REQUESTS, RESPONSES};
    use crate::ContractError;
    use bvs_registry::msg::StatusResponse;
    use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response};

    pub fn request(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        input: i64,
    ) -> Result<Response, ContractError> {
        REQUESTS.save(deps.storage, &input, &info.sender)?;

        Ok(Response::new()
            .add_attribute("method", "Request")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("input", input.to_string()))
    }

    pub fn respond(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        input: i64,
        output: i64,
    ) -> Result<Response, ContractError> {
        let operator = info.sender;
        if !REQUESTS.has(deps.storage, &input) {
            return Err(ContractError::RequestNotFound {});
        }

        let config = CONFIG.load(deps.storage)?;
        let query_status = bvs_registry::msg::QueryMsg::Status {
            service: env.contract.address.to_string(),
            operator: operator.to_string(),
            timestamp: Some(env.block.time.seconds()),
        };
        let StatusResponse(status) = deps
            .querier
            .query_wasm_smart(config.registry, &query_status)?;
        if status != bvs_registry::RegistrationStatus::Active as u8 {
            return Err(ContractError::Unauthorized {});
        }

        if RESPONSES.has(deps.storage, (input, &operator)) {
            return Err(ContractError::Responded {});
        }

        RESPONSES.save(deps.storage, (input, &operator), &output)?;

        Ok(Response::new()
            .add_attribute("method", "Respond")
            .add_attribute("operator", operator.to_string())
            .add_attribute("input", input.to_string())
            .add_attribute("output", output.to_string()))
    }

    pub fn compute(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        input: i64,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let prev_output = {
            if let Some(prev_output) = RESPONSES.may_load(deps.storage, (input, &operator))? {
                prev_output
            } else {
                return Err(ContractError::ResponseNotFound {});
            }
        };

        // To prove, we re-compute the result on-chain to verify the result.
        let new_output = expensive_computation(input);
        if prev_output == new_output {
            return Err(ContractError::InvalidProve {});
        }

        let config = CONFIG.load(deps.storage)?;

        // Save the new output to the storage
        RESPONSES.save(deps.storage, (input, &operator), &new_output)?;

        let request_slashing = bvs_vault_router::msg::ExecuteMsg::RequestSlashing(
            bvs_vault_router::msg::RequestSlashingPayload {
                operator: operator.to_string(),
                // We slash 0.01% of the operator's vault.
                bips: 1,
                timestamp: env.block.time,
                metadata: bvs_vault_router::msg::SlashingMetadata {
                    reason: "Invalid Prove".to_string(),
                },
            },
        );
        let execute_slashing = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.router.to_string(),
            msg: to_json_binary(&request_slashing)?,
            funds: vec![],
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::reply_always(execute_slashing, 1))
            .add_attribute("method", "Compute")
            .add_attribute("operator", operator.to_string())
            .add_attribute("input", input.to_string())
            .add_attribute("prev_output", prev_output.to_string())
            .add_attribute("new_output", new_output.to_string()))
    }

    pub fn slashing_cancel(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        // TODO(fuxingloh): implement SlashingCancel

        Ok(Response::new()
            .add_attribute("method", "SlashingCancel")
            .add_attribute("operator", operator.to_string()))
    }

    pub fn slashing_lock(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        // TODO(fuxingloh): implement SlashingLock
        Ok(Response::new()
            .add_attribute("method", "SlashingLock")
            .add_attribute("operator", operator.to_string()))
    }

    pub fn slashing_finalize(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        // TODO(fuxingloh): implement SlashingFinalize
        Ok(Response::new()
            .add_attribute("method", "SlashingFinalize")
            .add_attribute("operator", operator.to_string()))
    }

    pub fn register_operator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
                operator: operator.to_string(),
            })?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "RegisterOperator")
            .add_attribute("operator", operator.to_string()))
    }

    pub fn deregister_operator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(
                &bvs_registry::msg::ExecuteMsg::DeregisterOperatorFromService {
                    operator: operator.to_string(),
                },
            )?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "DeregisterOperator")
            .add_attribute("operator", operator.to_string()))
    }

    pub fn enable_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::EnableSlashing {
                slashing_parameters: bvs_registry::SlashingParameters {
                    destination: Some(env.contract.address),
                    max_slashing_bips: 1,
                    resolution_window: 60,
                },
            })?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "EnableSlashing"))
    }

    pub fn disable_slashing(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::DisableSlashing {})?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "DisableSlashing"))
    }
}

/// This function is an example of an expensive computation with
/// off-chain computing and on-chain objectively verifiable slashing.
/// You want to perform this off-chain to reduce gas costs.
/// When a malicious operator tries to cheat,
/// the on-chain verification can objectively verify the result by recomputing it on-chain.
pub fn expensive_computation(input: i64) -> i64 {
    // In this example, we don't check for overflow.
    input * input
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: cosmwasm_std::Reply) -> StdResult<Response> {
    // Not handled. To allow slashing to fail gracefully, e.g. slashing is occupied and in-progress.
    Ok(Response::default())
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
    use crate::state::RESPONSES;
    use crate::ContractError;
    use cosmwasm_std::{Addr, Deps};

    pub fn get_response(deps: Deps, input: i64, operator: Addr) -> Result<i64, ContractError> {
        let result = RESPONSES.may_load(deps.storage, (input, &operator))?;

        if let Some(input) = result {
            return Ok(input);
        }

        Err(ContractError::ResponseNotFound {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{ContractResult, QuerierResult, SystemResult};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let sender_info = message_info(&sender, &[]);

        let router = deps.api.addr_make("router");
        let registry = deps.api.addr_make("registry");
        let init_msg = InstantiateMsg {
            router: router.to_string(),
            registry: registry.to_string(),
            owner: sender.to_string(),
        };
        let res = instantiate(deps.as_mut(), env, sender_info, init_msg).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_message(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: registry.to_string(),
                    msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::RegisterAsService {
                        metadata: bvs_registry::msg::Metadata {
                            name: Some("The Squaring Company".to_string()),
                            uri: Some("https://the-squaring-company.com".to_string()),
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                })
                .add_attribute("method", "instantiate")
                .add_attribute("registry", registry.to_string())
                .add_attribute("router", router.to_string())
                .add_attribute("owner", sender.to_string())
        );
    }

    #[test]
    fn test_request() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let caller = deps.api.addr_make("anyone");
        let caller_info = message_info(&caller, &[]);
        let res = execute::request(deps.as_mut(), env, caller_info, 42).unwrap();

        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "Request")
                .add_attribute("sender", caller.to_string())
                .add_attribute("input", "42")
        );

        let requested_by = state::REQUESTS.load(&deps.storage, &42).unwrap();
        assert_eq!(requested_by, caller);
    }

    #[test]
    fn test_respond() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        {
            let caller = deps.api.addr_make("anyone");
            let caller_info = message_info(&caller, &[]);
            execute::request(deps.as_mut(), env.clone(), caller_info, 13).unwrap();

            let config = Config {
                router: deps.api.addr_make("router"),
                registry: deps.api.addr_make("registry"),
                owner: deps.api.addr_make("owner"),
            };
            CONFIG.save(deps.as_mut().storage, &config).unwrap();

            deps.querier.update_wasm(|_| -> QuerierResult {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&1).unwrap()).into())
            });
        }

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        let res = execute::respond(deps.as_mut(), env, operator_info, 13, 169).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "Respond")
                .add_attribute("operator", operator.to_string())
                .add_attribute("input", "13")
                .add_attribute("output", "169")
        );

        let result = state::RESPONSES
            .load(&deps.storage, (13, &operator))
            .unwrap();
        assert_eq!(result, 169);

        let response = query::get_response(deps.as_ref(), 13, operator).unwrap();
        assert_eq!(response, 169);
    }

    #[test]
    fn test_compute() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        {
            let caller = deps.api.addr_make("anyone");
            let caller_info = message_info(&caller, &[]);
            execute::request(deps.as_mut(), env.clone(), caller_info, 13).unwrap();

            let config = Config {
                router: deps.api.addr_make("router"),
                registry: deps.api.addr_make("registry"),
                owner: deps.api.addr_make("owner"),
            };
            CONFIG.save(deps.as_mut().storage, &config).unwrap();

            deps.querier.update_wasm(|_| -> QuerierResult {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&1).unwrap()).into())
            });
        }

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        execute::respond(deps.as_mut(), env.clone(), operator_info, 13, 13).unwrap();

        let anyone = deps.api.addr_make("anyone");

        let config = Config {
            router: deps.api.addr_make("router"),
            registry: deps.api.addr_make("registry"),
            owner: deps.api.addr_make("owner"),
        };
        CONFIG.save(deps.as_mut().storage, &config).unwrap();

        let res = execute::compute(
            deps.as_mut(),
            mock_env(),
            message_info(&anyone, &[]),
            13,
            operator.clone(),
        )
        .unwrap();

        let expected = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.router.to_string(),
            msg: to_json_binary(&bvs_vault_router::msg::ExecuteMsg::RequestSlashing(
                bvs_vault_router::msg::RequestSlashingPayload {
                    operator: operator.to_string(),
                    bips: 1,
                    timestamp: mock_env().block.time,
                    metadata: bvs_vault_router::msg::SlashingMetadata {
                        reason: "Invalid Prove".to_string(),
                    },
                },
            ))
            .unwrap(),
            funds: vec![],
        };

        assert_eq!(
            res,
            Response::new()
                .add_submessage(cosmwasm_std::SubMsg::reply_always(expected, 1))
                .add_attribute("method", "Compute")
                .add_attribute("operator", operator.to_string())
                .add_attribute("input", "13")
                .add_attribute("prev_output", "13")
                .add_attribute("new_output", "169")
        );
    }
}
