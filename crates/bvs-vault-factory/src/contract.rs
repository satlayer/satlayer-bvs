#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{REGISTRY, ROUTER};
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
            let cw20_token = deps.api.addr_validate(&cw20)?;
            execute::deploy_cw20_vault(deps, env, info, cw20_token)
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
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultCodeIds {} => to_json_binary(&query::get_available_code_ids(_deps)?),
    }
}

mod execute {
    use super::*;
    use crate::{
        auth,
        state::{VaultType, CODE_IDS},
    };
    use cosmwasm_std::{Addr, Event, Response};

    pub fn deploy_cw20_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cw20: Addr,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        let msg = Cw20InstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: info.sender.clone().to_string(),
            cw20_contract: cw20.to_string(),
        };

        let code_id = match CODE_IDS.load(deps.storage, VaultType::Cw20Vault) {
            Ok(code_id) => code_id,
            Err(_) => return Err(ContractError::CodeIdNotFound {}),
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
            .add_event(
                Event::new("deploy_vault_cw20")
                    .add_attribute("cw20", cw20.to_string())
                    .add_attribute("operator", info.sender.to_string()),
            ))
    }

    pub fn deploy_bank_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        denom: String,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        let msg = BankVaultInstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: info.sender.clone().to_string(),
            denom: denom.clone(),
        };

        let code_id = match CODE_IDS.load(deps.storage, VaultType::BankVault) {
            Ok(code_id) => code_id,
            Err(_) => return Err(ContractError::CodeIdNotFound {}),
        };

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("{} Vault Bank", denom),
        };

        let event = Event::new("deploy_vault_bank")
            .add_attribute("denom", denom)
            .add_attribute("operator", info.sender.to_string());

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_event(event))
    }

    pub fn set_code_id(
        deps: DepsMut,
        info: MessageInfo,
        code_id: u64,
        label: VaultType,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info).map_err(ContractError::Ownership)?;

        CODE_IDS.save(deps.storage, label, &code_id)?;

        let event = Event::new("add_code_id").add_attribute("code_id", code_id.to_string());

        Ok(Response::new().add_event(event))
    }
}

mod query {
    use std::collections::BTreeMap;

    use crate::{msg::VaultCodeIdsResponse, state::CODE_IDS};

    use super::*;
    use cosmwasm_std::Deps;

    pub fn get_available_code_ids(deps: Deps) -> StdResult<VaultCodeIdsResponse> {
        let code_ids: BTreeMap<String, u64> = CODE_IDS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| {
                let (vault_type, code_id) = item?;
                Ok((vault_type.to_string(), code_id))
            })
            .collect::<StdResult<_>>()?;

        Ok(VaultCodeIdsResponse { code_ids })
    }
}
