#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{REGISTRY, ROUTER};
use bvs_library::ownership;
use bvs_pauser;
use bvs_vault_bank::msg::InstantiateMsg as BankVaultInstantiateMsg;
use bvs_vault_bank_tokenized::msg::InstantiateMsg as BankTokenizedVaultInstantiateMsg;
use bvs_vault_cw20::msg::InstantiateMsg as Cw20InstantiateMsg;
use bvs_vault_cw20_tokenized::msg::InstantiateMsg as Cw20TokenizedVaultInstantiateMsg;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

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
        .add_attribute("router", router.to_string())
        .add_attribute("registry", registry.to_string()))
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
        ExecuteMsg::DeployCw20Tokenized { symbol, name, cw20 } => {
            let cw20 = deps.api.addr_validate(&cw20)?;
            execute::deploy_cw20_tokenized_vault(deps, env, info, symbol, name, cw20)
        }
        ExecuteMsg::DeployBank { denom } => execute::deploy_bank_vault(deps, env, info, denom),
        ExecuteMsg::DeployBankTokenized {
            denom,
            decimals,
            symbol,
            name,
        } => execute::deploy_bank_tokenized_vault(deps, env, info, denom, decimals, symbol, name),
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
            label: format!("BVS CW20 Vault: {cw20}"),
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
            label: format!("BVS Bank Vault: {denom}"),
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

    pub fn deploy_cw20_tokenized_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        symbol: String,
        name: String,
        cw20: Addr,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        if !symbol.starts_with("sat") {
            return Err(ContractError::VaultError {
                msg: "Symbol must start with 'sat'".to_string(),
            });
        }

        let operator = info.sender;
        let msg = Cw20TokenizedVaultInstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: operator.to_string(),
            symbol: symbol.clone(),
            name: name.clone(),
            cw20_contract: cw20.to_string(),
        };

        let code_id = get_code_id(deps.storage, &VaultType::Cw20Tokenized)?;

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: "BVS CW20 Tokenized Vault".to_string(),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_event(
                Event::new("DeployVault")
                    .add_attribute("type", "cw20_tokenized")
                    .add_attribute("cw20", cw20.to_string())
                    .add_attribute("symbol", symbol)
                    .add_attribute("name", name)
                    .add_attribute("operator", operator.to_string()),
            ))
    }

    pub fn deploy_bank_tokenized_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        denom: String,
        decimals: u8,
        symbol: String,
        name: String,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        if !symbol.starts_with("sat") {
            return Err(ContractError::VaultError {
                msg: "Symbol must start with 'sat'".to_string(),
            });
        }

        let operator = info.sender;
        let msg = BankTokenizedVaultInstantiateMsg {
            pauser: bvs_pauser::api::get_pauser(deps.storage)?.to_string(),
            router: ROUTER.load(deps.storage)?.to_string(),
            operator: operator.to_string(),
            denom: denom.clone(),
            decimals,
            symbol: symbol.clone(),
            name: name.clone(),
        };

        let code_id = get_code_id(deps.storage, &VaultType::BankTokenized)?;

        let instantiate_msg = cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label: format!("BVS Bank Tokenized Vault: {denom}"),
        };

        Ok(Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(instantiate_msg))
            .add_event(
                Event::new("DeployVault")
                    .add_attribute("type", "bank_tokenized")
                    .add_attribute("denom", denom)
                    .add_attribute("decimals", decimals.to_string())
                    .add_attribute("symbol", symbol)
                    .add_attribute("name", name)
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

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 2.0.0 (new)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use bvs_library::ownership;
    use cosmwasm_std::Event;

    use crate::contract::{execute, query};

    #[test]
    fn test_set_code_id() {
        let mut deps = cosmwasm_std::testing::mock_dependencies();
        let owner = deps.api.addr_make("owner");

        ownership::set_owner(deps.as_mut().storage, &owner).unwrap();

        let info = cosmwasm_std::testing::message_info(&owner, &[]);
        let code_id = 123;
        let vault_type = crate::msg::VaultType::Cw20;

        let res = execute::set_code_id(deps.as_mut(), info, code_id, vault_type.clone()).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(1, res.events.len());

        let event = &res.events[0];
        let expected_event = Event::new("SetCodeId")
            .add_attribute("code_id", code_id.to_string())
            .add_attribute("vault_type", vault_type.to_string());

        assert_eq!(expected_event, *event);

        let code_id = query::code_id(deps.as_ref(), vault_type).unwrap();
        assert_eq!(123, code_id);
    }
}
