#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{REGISTRY, ROUTER};
use bvs_library::ownership;
use bvs_pauser;
use bvs_vault_bank::msg::InstantiateMsg as BankVaultInstantiateMsg;
use bvs_vault_cw20::msg::InstantiateMsg as Cw20InstantiateMsg;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

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

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let router = deps.api.addr_validate(&msg.router)?;
    let registry = deps.api.addr_validate(&msg.registry)?;

    ROUTER.save(deps.storage, &router)?;
    REGISTRY.save(deps.storage, &registry)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner.to_string())
        .add_attribute("pauser", pauser.to_string())
        .add_attribute("router", router.to_string()))
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
        ExecuteMsg::DeployCw20 { cw20 } => {
            let cw20 = deps.api.addr_validate(&cw20)?;
            execute::deploy_cw20_vault(deps, env, info, cw20)
        }
        ExecuteMsg::DeployBank { denom } => execute::deploy_bank_vault(deps, env, info, denom),
        ExecuteMsg::SetCodeId {
            code_id,
            vault_type,
        } => execute::set_code_id(deps, info, code_id, vault_type),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CodeId { vault_type } => Ok(to_json_binary(&query::code_id(deps, vault_type)?)?),
    }
}

mod execute {
    use super::*;
    use crate::msg::VaultType;
    use crate::state::get_code_id;
    use crate::{auth, state};
    use cosmwasm_std::{Addr, Event, Response};

    pub fn deploy_cw20_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cw20: Addr,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        let operator = info.sender;
        let msg = Cw20InstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: operator.to_string(),
            cw20_contract: cw20.to_string(),
        };

        let code_id = get_code_id(deps.storage, &VaultType::Cw20)?;

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("BVS CW20 Vault: {}", cw20),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_event(
                Event::new("DeployVault")
                    .add_attribute("type", "cw20")
                    .add_attribute("cw20", cw20.to_string())
                    .add_attribute("operator", operator.to_string()),
            ))
    }

    pub fn deploy_bank_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        denom: String,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        let operator = info.sender;
        let msg = BankVaultInstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: operator.to_string(),
            denom: denom.clone(),
        };

        let code_id = get_code_id(deps.storage, &VaultType::Bank)?;

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("BVS Bank Vault: {}", denom),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_event(
                Event::new("DeployVault")
                    .add_attribute("type", "bank")
                    .add_attribute("denom", denom)
                    .add_attribute("operator", operator.to_string()),
            ))
    }

    pub fn set_code_id(
        deps: DepsMut,
        info: MessageInfo,
        code_id: u64,
        vault_type: VaultType,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        state::set_code_id(deps.storage, &vault_type, &code_id)?;

        Ok(Response::new().add_event(
            Event::new("SetCodeId")
                .add_attribute("code_id", code_id.to_string())
                .add_attribute("vault_type", vault_type.to_string()),
        ))
    }
}

mod query {
    use super::*;
    use crate::msg::VaultType;
    use crate::state;
    use cosmwasm_std::Deps;

    pub fn code_id(deps: Deps, vault_type: VaultType) -> Result<u64, ContractError> {
        state::get_code_id(deps.storage, &vault_type)
    }
}
