#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use crate::state::{Config, CONFIG};
use cosmwasm_std::{to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response};

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
    };
    CONFIG.save(deps.storage, &config)?;

    // Register this contract as a Service in BVS Registry
    let register: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
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
        // Fire-off the register message here
        .add_message(register)
        .add_attribute("method", "instantiate")
        .add_attribute("registry", config.registry)
        .add_attribute("router", config.router))
}

// TODO(fuxingloh): set slashing parameters in migrate

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request { input } => execute::request(deps, info, input),
        ExecuteMsg::Respond { input, output } => execute::respond(deps, info, input, output),
        ExecuteMsg::SlashingProve { input, operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_prove(deps, env, info, input, operator)
        }
        ExecuteMsg::SlashingCancel { .. } => Ok(Response::new()),
        ExecuteMsg::SlashingLock { .. } => Ok(Response::new()),
        ExecuteMsg::SlashingFinalize { .. } => Ok(Response::new()),
        ExecuteMsg::RegisterOperator { .. } => Ok(Response::new()),
    }
}

pub mod execute {
    use crate::contract::expensive_computation;
    use crate::state::{REQUESTS, RESPONSES};
    use crate::ContractError;
    use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response};

    pub fn request(
        deps: DepsMut,
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
        info: MessageInfo,
        input: i64,
        output: i64,
    ) -> Result<Response, ContractError> {
        if !REQUESTS.has(deps.storage, &input) {
            return Err(ContractError::RequestNotFound {});
        }

        let operator = info.sender;
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

    pub fn slashing_prove(
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

        // Save the new output to the storage
        RESPONSES.save(deps.storage, (input, &operator), &new_output)?;

        let request_slashing = bvs_vault_router::msg::ExecuteMsg::RequestSlashing(
            bvs_vault_router::msg::RequestSlashingPayload {
                operator: operator.to_string(),
                bips: 1,
                timestamp: env.block.time,
                metadata: bvs_vault_router::msg::SlashingMetadata {
                    reason: "Invalid Prove".to_string(),
                },
            },
        );
        let slashing_msg: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: "bbn1m2f0ctm657e22p843lgm9pnwlqtnuf3jgln7uyqrw6sy7nd5pc5qaasfud"
                .to_string(),
            msg: to_json_binary(&request_slashing)?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(slashing_msg)
            .add_attribute("method", "Prove")
            .add_attribute("operator", operator.to_string())
            .add_attribute("input", input.to_string())
            .add_attribute("prev_output", prev_output.to_string())
            .add_attribute("new_output", new_output.to_string()))
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
        );
    }

    #[test]
    fn test_request() {
        let mut deps = mock_dependencies();

        let caller = deps.api.addr_make("anyone");
        let caller_info = message_info(&caller, &[]);
        let res = execute::request(deps.as_mut(), caller_info, 42).unwrap();

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

        {
            let caller = deps.api.addr_make("anyone");
            let caller_info = message_info(&caller, &[]);
            execute::request(deps.as_mut(), caller_info, 13).unwrap();
        }

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        let res = execute::respond(deps.as_mut(), operator_info, 13, 169).unwrap();
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
}
