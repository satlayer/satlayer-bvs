#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
};

use crate::state::{Config, CONFIG};
use cosmwasm_std::{to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response};

const CONTRACT_NAME: &str = "crates.io:bvs-governance";
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
                name: Some("The Governing Project".to_string()),
                uri: Some("https://the-governance.com".to_string()),
            },
        })?,
        funds: vec![],
    }
    .into();

    cw3_fixed_multisig::contract::instantiate(deps, _env, _info, msg.cw3_instantiate_msg)?;

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
        ExecuteMsg::Base(msg) => cw3_base::execute(deps, env, info, msg).map_err(Into::into),
        ExecuteMsg::Extended(msg) => {
            todo!("Extended execute message not implemented: {:?}", msg)
        }
    }
}

mod cw3_base {
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    use cw3_fixed_multisig::{msg::ExecuteMsg, msg::QueryMsg, ContractError};

    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        cw3_fixed_multisig::contract::execute(deps, env, info, msg)
    }

    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        cw3_fixed_multisig::contract::query(deps, env, msg)
    }
}
