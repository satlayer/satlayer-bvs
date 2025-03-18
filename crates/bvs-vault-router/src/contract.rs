#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::set_registry;
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
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

    let registry = deps.api.addr_validate(&msg.registry)?;
    set_registry(deps.storage, &registry)?;

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
    pub mod vault {
        use crate::error::ContractError;
        use cosmwasm_schema::cw_serde;
        use cosmwasm_std::{Addr, Deps, Env};

        #[cw_serde]
        pub enum VaultInfoQueryMsg {
            VaultInfo {},
        }

        #[cw_serde]
        pub struct VaultInfoResponse {
            pub router: String,
        }

        /// Asserts that the vault contains the QueryMsg::VaultInfo and is connected to the router.
        pub fn assert_vault_info(deps: &Deps, env: Env, vault: Addr) -> Result<(), ContractError> {
            let response: VaultInfoResponse = deps
                .querier
                .query_wasm_smart(vault.to_string(), &VaultInfoQueryMsg::VaultInfo {})?;
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
            to_json_binary(&query::is_validating(deps, operator)?)
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
    use bvs_registry::msg::QueryMsg;
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
    pub fn is_validating(deps: Deps, operator: Addr) -> StdResult<bool> {
        // Currently, all vaults are not delegated.
        // To be implemented in M3, by connecting to the ` bvs_registry ` contract.
        // The `bvs_registry` contract will store Operator and Vault relationships.
        let registry = state::get_registry(deps.storage)?;
        let is_operator_active: bool = deps.querier.query_wasm_smart(
            registry.to_string(),
            &QueryMsg::IsOperatorActive {
                0: operator.to_string(),
            },
        )?;

        Ok(is_operator_active)
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
    use super::*;
    use super::{
        execute::{
            set_vault,
            vault::{VaultInfoQueryMsg, VaultInfoResponse},
        },
        query::{is_validating, is_whitelisted, list_vaults},
    };
    use crate::msg::InstantiateMsg;
    use crate::state::{Vault, REGISTRY, VAULTS};
    use bvs_registry::msg::{IsOperatorActiveResponse, QueryMsg as RegistryQueryMsg};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, Attribute, ContractResult, Event, OwnedDeps, QuerierResult, SystemError,
        SystemResult, WasmQuery,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let pauser = deps.api.addr_make("pauser");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
            pauser: pauser.to_string(),
        };

        let response = instantiate(deps.as_mut(), env, owner_info, msg).unwrap();

        assert_eq!(
            response.attributes,
            vec![
                Attribute::new("method", "instantiate"),
                Attribute::new("owner", owner.to_string()),
                Attribute::new("pauser", pauser.to_string()),
            ]
        );
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let pauser = deps.api.addr_make("pauser");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
            pauser: pauser.to_string(),
        };

        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        (deps, env, owner_info)
    }

    #[test]
    fn test_set_vault() {
        let (mut deps, env, owner_info) = instantiate_contract();

        let vault = deps.api.addr_make("vault");
        let vault_contract_addr = deps.api.addr_make("cosmos2contract");

        // whitelist is false
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault.clone(),
                false,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.attributes.len(), 0);
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("VaultUpdated")
                    .add_attribute("vault", vault.clone())
                    .add_attribute("whitelisted", "false")
            );

            let vault = VAULTS
                .may_load(deps.as_ref().storage, &vault)
                .unwrap()
                .unwrap();
            assert_eq!(vault.whitelisted, false);
        }

        deps.querier
            .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                if let WasmQuery::Smart { contract_addr, msg } = req {
                    if *contract_addr == deps.api.addr_make("vault").to_string() {
                        let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                        match msg {
                            VaultInfoQueryMsg::VaultInfo {} => {
                                let response = VaultInfoResponse {
                                    router: vault_contract_addr.to_string(),
                                };
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.to_string(),
                        })
                    }
                } else {
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "Unsupported query".to_string(),
                    })
                }
            });

        let vault = deps.api.addr_make("vault");

        // whitelist is true and set successfully
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault.clone(),
                true,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.attributes.len(), 0);
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("VaultUpdated")
                    .add_attribute("vault", vault.clone())
                    .add_attribute("whitelisted", "true")
            );

            let vault = VAULTS
                .may_load(deps.as_ref().storage, &vault)
                .unwrap()
                .unwrap();
            assert_eq!(vault.whitelisted, true);
        }

        // whitelist is true and failed to set: No such contract
        let empty_vault = deps.api.addr_make("empty_vault");
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                empty_vault.clone(),
                true,
            );
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(
                err.to_string(),
                format!(
                    "Generic error: Querier system error: No such contract: {}",
                    empty_vault.to_string()
                )
            );
        }

        // whitelist is true and failed to set: Vault is not connected to the router
        let new_vault = deps.api.addr_make("new_vault");
        {
            deps.querier
                .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                    if let WasmQuery::Smart { contract_addr, msg } = req {
                        if *contract_addr == deps.api.addr_make("vault").to_string() {
                            let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                            match msg {
                                VaultInfoQueryMsg::VaultInfo {} => {
                                    let response = VaultInfoResponse {
                                        router: new_vault.to_string(),
                                    };
                                    SystemResult::Ok(ContractResult::Ok(
                                        to_json_binary(&response).unwrap(),
                                    ))
                                }
                            }
                        } else {
                            SystemResult::Err(SystemError::NoSuchContract {
                                addr: contract_addr.to_string(),
                            })
                        }
                    } else {
                        SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Unsupported query".to_string(),
                        })
                    }
                });

            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault.clone(),
                true,
            );
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(
                err.to_string(),
                ContractError::VaultError {
                    msg: "Vault is not connected to the router".to_string()
                }
                .to_string()
            );
        }
    }

    #[test]
    fn test_query_is_whitelisted() {
        let mut deps = mock_dependencies();

        let vault = deps.api.addr_make("vault");
        VAULTS
            .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
            .unwrap();

        let response = is_whitelisted(deps.as_ref(), vault).unwrap();
        assert!(response)
    }

    #[test]
    fn test_query_is_delegated() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let registry = deps.api.addr_make("registry");

        REGISTRY.save(&mut deps.storage, &registry).unwrap();

        deps.querier
            .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                if let WasmQuery::Smart { contract_addr, msg } = req {
                    if *contract_addr == deps.api.addr_make("registry").to_string() {
                        let msg: RegistryQueryMsg = from_json(msg).unwrap();
                        match msg {
                            RegistryQueryMsg::IsOperatorActive(operator) => {
                                let response = IsOperatorActiveResponse(false);
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                            _ => SystemResult::Err(SystemError::Unknown {}),
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.to_string(),
                        })
                    }
                } else {
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "Unsupported query".to_string(),
                    })
                }
            });

        let response = is_validating(deps.as_ref(), operator).unwrap();
        assert!(!response)
    }

    #[test]
    fn test_query_list_vaults() {
        let deps = mock_dependencies();

        let response = list_vaults(deps.as_ref(), 0, None).unwrap();
        assert_eq!(response.0.len(), 0)
    }
}
