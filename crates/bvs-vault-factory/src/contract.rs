#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::ROUTER;
use bvs_library::ownership;
use bvs_pauser;
use bvs_registry;
use bvs_vault_bank::msg::InstantiateMsg as BankVaultInstantiateMsg;
use bvs_vault_cw20::msg::InstantiateMsg as Cw20InstantiateMsg;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

const CONTRACT_NAME: &str = concat!("crate:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner)
        .add_attribute("pauser", pauser))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_pauser::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::DeployCw20 { cw20, code_id } => {
            deps.api.addr_validate(&cw20)?;
            execute::deploy_cw20_contract(deps, env, info, cw20, code_id)
        }
        ExecuteMsg::DeployBank { denom, code_id } => {
            execute::deploy_vault_bank(deps, env, info, denom, code_id)
        }
        ExecuteMsg::SetVaults { router, registry } => {
            deps.api.addr_validate(&router)?;
            deps.api.addr_validate(&registry)?;
            let router = Addr::unchecked(router);
            let registry = Addr::unchecked(registry);
            execute::set_vaults(deps, info, router, registry)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

mod execute {
    use crate::state::REGISTRY;

    use super::*;
    use cosmwasm_std::{Addr, QueryRequest, Response, WasmQuery};

    pub fn set_vaults(
        deps: DepsMut,
        info: MessageInfo,
        router: Addr,
        registry: Addr,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info).map_err(ContractError::Ownership)?;

        ROUTER.save(deps.storage, &router)?;
        REGISTRY.save(deps.storage, &registry)?;

        Ok(Response::new()
            .add_attribute("method", "set_router")
            .add_attribute("router", router))
    }

    pub fn deploy_cw20_contract(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cw20: String,
        code_id: u64,
    ) -> Result<Response, ContractError> {
        if ROUTER.load(deps.storage).is_err() || REGISTRY.load(deps.storage).is_err() {
            return Err(ContractError::NotReady {});
        }

        let msg = bvs_registry::msg::QueryMsg::IsOperator(info.sender.to_string());

        let query = WasmQuery::Smart {
            contract_addr: REGISTRY.load(deps.storage)?.to_string(),
            msg: to_json_binary(&msg)?,
        };

        let is_operator: bvs_registry::msg::IsOperatorResponse =
            deps.querier.query(&QueryRequest::Wasm(query))?;

        if !is_operator.0 {
            return Err(ContractError::Unauthorized {});
        }

        let msg = Cw20InstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: info.sender.clone().to_string(),
            cw20_contract: cw20.clone(),
        };

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("{} Vault CW20", cw20),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_attribute("method", "deploy_cw20_contract")
            .add_attribute("cw20", cw20)
            .add_attribute("operator", info.sender.to_string()))
    }

    pub fn deploy_vault_bank(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        denom: String,
        code_id: u64,
    ) -> Result<Response, ContractError> {
        if ROUTER.load(deps.storage).is_err() || REGISTRY.load(deps.storage).is_err() {
            return Err(ContractError::NotReady {});
        }

        let msg = bvs_registry::msg::QueryMsg::IsOperator(info.sender.to_string());

        let query = WasmQuery::Smart {
            contract_addr: REGISTRY.load(deps.storage)?.to_string(),
            msg: to_json_binary(&msg)?,
        };

        let is_operator: bvs_registry::msg::IsOperatorResponse =
            deps.querier.query(&QueryRequest::Wasm(query))?;

        if !is_operator.0 {
            return Err(ContractError::Unauthorized {});
        }

        let msg = BankVaultInstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: info.sender.clone().to_string(),
            denom: denom.clone(),
        };

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("{} Vault Bank", denom),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_attribute("method", "deploy_vault_bank")
            .add_attribute("denom", denom)
            .add_attribute("operator", info.sender.to_string()))
    }
}

mod query {}
