#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crate:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::SetVault { vault, whitelisted } => {
            let vault = deps.api.addr_validate(&vault)?;
            execute::set_vault(deps, env, info, vault, whitelisted)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

mod execute {
    use crate::contract::execute::vault::assert_vault_info;
    use crate::error::ContractError;
    use crate::state;
    use bvs_library::ownership;
    use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response};

    /// Set the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    /// After `whitelisting` a vault, the router allows the vault to accept deposits.
    /// See [`query::is_whitelisted`] for more information.
    pub fn set_vault(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        vault: Addr,
        whitelisted: bool,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        // For whitelisted vault, we assert that the vault is connected to the router.
        if whitelisted {
            assert_vault_info(&deps.as_ref(), _env, vault.clone())?;
        }

        state::VAULTS.save(deps.storage, &vault, &state::Vault { whitelisted })?;

        Ok(Response::new().add_event(
            Event::new("VaultUpdated")
                .add_attribute("vault", vault)
                .add_attribute("whitelisted", whitelisted.to_string()),
        ))
    }

    /// Snipped implementation of Vault's API
    mod vault {
        use crate::error::ContractError;
        use cosmwasm_schema::cw_serde;
        use cosmwasm_std::{Addr, Deps, Env};

        #[cw_serde]
        enum QueryMsg {
            VaultInfo {},
        }

        #[cw_serde]
        struct VaultInfoResponse {
            router: String,
        }

        /// Asserts that the vault contains the QueryMsg::VaultInfo and is connected to the router.
        pub fn assert_vault_info(deps: &Deps, env: Env, vault: Addr) -> Result<(), ContractError> {
            let response: VaultInfoResponse = deps
                .querier
                .query_wasm_smart(vault.to_string(), &QueryMsg::VaultInfo {})?;
            if response.router == env.contract.address.to_string() {
                Ok(())
            } else {
                Err(ContractError::VaultError {
                    msg: "Vault is not connected to the router".to_string(),
                })
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsWhitelisted { vault } => {
            let vault = deps.api.addr_validate(&vault)?;
            to_json_binary(&query::is_whitelisted(deps, vault)?)
        }
        QueryMsg::IsValidating { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_delegated(deps, operator)?)
        }
        QueryMsg::ListVaults { limit, start_after } => {
            let limit = limit.map_or(100, |v| v.min(100));
            let start_after = start_after
                .map(|s| deps.api.addr_validate(&s))
                .transpose()?;
            to_json_binary(&query::list_vaults(deps, limit, start_after)?)
        }
    }
}

mod query {
    use crate::msg::{Vault, VaultListResponse};
    use crate::state;
    use cosmwasm_std::{Addr, Deps, StdResult};
    use cw_storage_plus::Bound;

    /// Returns whether the vault is whitelisted or not.
    /// Called by the vault to check if a vault can accept deposits.
    /// Only whitelisted vaults can accept deposits.
    pub fn is_whitelisted(deps: Deps, vault: Addr) -> StdResult<bool> {
        state::VAULTS
            .may_load(deps.storage, &vault)
            .map(|v| v.map(|v| v.whitelisted).unwrap_or(false))
    }

    /// Returns whether the operator is delegated or not.
    /// Called by vaults to check if they are delegated.
    /// Delegated vaults must queue withdrawals.
    pub fn is_delegated(_deps: Deps, _operator: Addr) -> StdResult<bool> {
        // Currently, all vaults are not delegated.
        // To be implemented in M3, by connecting to the ` bvs_registry ` contract.
        // The `bvs_registry` contract will store Operator and Vault relationships.
        Ok(false)
    }

    /// List all vaults in the router.
    /// Support pagination.
    pub fn list_vaults(
        deps: Deps,
        limit: u32,
        start_after: Option<Addr>,
    ) -> StdResult<VaultListResponse> {
        let range_max = start_after.as_ref().map(Bound::exclusive);
        let items = state::VAULTS.range(
            deps.storage,
            None,
            range_max,
            cosmwasm_std::Order::Descending,
        );

        let vaults = items
            .take(limit.try_into().unwrap())
            .map(|item| {
                let (k, v) = item?;
                Ok(Vault {
                    vault: k,
                    whitelisted: v.whitelisted,
                })
            })
            .collect::<StdResult<_>>()?;
        Ok(VaultListResponse(vaults))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
