#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::ROUTER;
use bvs_library::ownership;
use bvs_pauser;
use bvs_vault_bank::msg::InstantiateMsg as BankVaultInstantiateMsg;
use bvs_vault_cw20::msg::InstantiateMsg as Cw20InstantiateMsg;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

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

    let router = deps.api.addr_validate(&msg.router)?;
    ROUTER.save(deps.storage, &router)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner)
        .add_attribute("pauser", pauser)
        .add_attribute("router", router))
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
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

mod execute {
    use super::*;
    use cosmwasm_std::Response;

    pub fn deploy_cw20_contract(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cw20: String,
        code_id: u64,
    ) -> Result<Response, ContractError> {
        //TODO: Implement operator authorized only, currently blocked

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
        //TODO: Implement operator authorized only, currently blocked

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
