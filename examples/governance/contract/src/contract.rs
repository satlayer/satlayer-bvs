/// Example: Social governance as BVS contract integrated with SatLayer protocol
/// The contract is a wrapper around the cw3 spec.
/// Slashing is proposal will to be made by one of the multisig members
/// Each of the Satlayer slashing phases will need to be proposed
/// voted on and executed by the multisig members in this example.
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use crate::state::{Config, CONFIG};
use cosmwasm_std::{to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response};

const CONTRACT_NAME: &str = "crates.io:bvs-governance";
const CONTRACT_VERSION: &str = "0.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        router: deps.api.addr_validate(&msg.router)?,
        registry: deps.api.addr_validate(&msg.registry)?,
        owner: deps.api.addr_validate(&msg.owner)?,
    };

    CONFIG.save(deps.storage, &config)?;

    // Register this contract as a Service in BVS Registry
    let register_as_service: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
        contract_addr: msg.registry.clone(),
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

    cw3_fixed_multisig::contract::instantiate(deps.branch(), _env, _info, msg.cw3_instantiate_msg)?;
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::Base(msg) => {
            cw3_base_execute::execute(deps, env, info, msg).map_err(Into::into)
        }
        ExecuteMsg::Extended(msg) => {
            // In a production BVS contract
            // A project would implement its one extended execute messages
            // for custom functionality
            todo!("Extended execute message not implemented: {:?}", msg)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, query: QueryMsg) -> Result<Binary, ContractError> {
    match query {
        QueryMsg::Base(msg) => cw3_base_query::query(deps, env, msg).map_err(Into::into),
        QueryMsg::Extended(msg) => {
            todo!("Extended query message not implemented: {:?}", msg)
        }
    }
}

mod cw3_base_execute {
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

    use cw3_fixed_multisig::{msg::ExecuteMsg, ContractError};

    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        cw3_fixed_multisig::contract::execute(deps, env, info, msg)
    }
}

mod cw3_base_query {
    use cosmwasm_std::{Binary, Deps, Env, StdResult};
    use cw3_fixed_multisig::msg::QueryMsg;

    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        cw3_fixed_multisig::contract::query(deps, env, msg)
    }
}

mod extended_execute {

    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

    use crate::{error::ContractError, msg::ExtendedExecuteMsg};

    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: ExtendedExecuteMsg,
    ) -> Result<Response, ContractError> {
        todo!()
    }
}
