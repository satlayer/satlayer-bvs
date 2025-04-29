#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::set_registry;
use bvs_library::ownership;
use bvs_library::time::DAYS;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_SLASHABLE_DELAY: u64 = 7 * DAYS;
const MAX_STRING_BYTES: usize = 400;

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
        ExecuteMsg::SetWithdrawalLockPeriod {
            0: withdrawal_lock_period,
        } => execute::set_withdrawal_lock_period(deps, env, info, withdrawal_lock_period),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
        ExecuteMsg::SlashingRequest(payload) => execute::slashing_request(deps, env, info, payload),
    }
}

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 1.0.0 to 2.0.0
/// New `OPERATOR_VAULTS: Map<(&Addr, &Addr), ()>` is created to allow vaults to be queried by
/// operator. The existing `VAULTS` iterated over and added to `OPERATOR_VAULTS`.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let old_version =
        cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match old_version.major {
        1 => migrate::vaults_to_index_operator(deps),
        _ => Ok(Response::default()),
    }
}

mod migrate {
    use crate::state::{OPERATOR_VAULTS, VAULTS};

    use super::*;

    pub fn vaults_to_index_operator(deps: DepsMut) -> Result<Response, ContractError> {
        let vaults = VAULTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        for vault in vaults {
            let vault_info = vault::get_vault_info(deps.as_ref(), &vault)?;

            OPERATOR_VAULTS.save(deps.storage, (&vault_info.operator, &vault), &())?;
        }

        Ok(Response::default())
    }
}

mod execute {
    use super::*;
    use crate::error::ContractError;
    use crate::msg::SlashingRequestPayload;
    use crate::state::{self, DEFAULT_WITHDRAWAL_LOCK_PERIOD};
    use crate::state::{SlashingRequest, WITHDRAWAL_LOCK_PERIOD};
    use crate::ContractError::InvalidSlashingRequest;
    use bvs_library::addr::Operator;
    use bvs_library::ownership;
    use bvs_registry::msg::{
        IsOperatorOptedInToSlashingResponse, SlashingParametersResponse, StatusResponse,
    };
    use bvs_registry::RegistrationStatus;
    use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response, Uint64};

    /// Set the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    /// After `whitelisting` a vault, the router allows the vault to accept deposits.
    /// See [`query::is_whitelisted`] for more information.
    pub fn set_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        vault: Addr,
        whitelisted: bool,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        // Only for whitelisted vault:
        // - we assert that the vault is connected to the router.
        // - we save the operator to vault mapping
        //   if a vault is never whitelisted, it won't be added to the operator mapping.
        // Otherwise for non-whitelisted, the `state::VAULTS` will only be updated.
        // This is to allow the vault to be effectively removed
        // by setting `whitelisted: false` without checks from the router in
        // case the vault is malformed or broken.
        if whitelisted {
            let vault_info = vault::get_vault_info(deps.as_ref(), &vault)?;

            // The vault is not connected to this router.
            if vault_info.router != env.contract.address {
                return Err(ContractError::VaultError {
                    msg: "Vault is not connected to the router".to_string(),
                });
            }

            state::OPERATOR_VAULTS.save(deps.storage, (&vault_info.operator, &vault), &())?;
        }

        state::VAULTS.save(deps.storage, &vault, &state::Vault { whitelisted })?;

        Ok(Response::new().add_event(
            Event::new("VaultUpdated")
                .add_attribute("vault", vault)
                .add_attribute("whitelisted", whitelisted.to_string()),
        ))
    }

    pub fn set_withdrawal_lock_period(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        withdrawal_lock_period: Uint64,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        if withdrawal_lock_period.is_zero() {
            return Err(ContractError::VaultError {
                msg: "Cannot set new withdrawal lock period to zero".to_string(),
            });
        }

        let prev_withdrawal_lock_period = WITHDRAWAL_LOCK_PERIOD
            .may_load(deps.storage)?
            .unwrap_or(DEFAULT_WITHDRAWAL_LOCK_PERIOD);

        WITHDRAWAL_LOCK_PERIOD.save(deps.storage, &withdrawal_lock_period)?;

        Ok(Response::new().add_event(
            Event::new("SetWithdrawalLockPeriod")
                .add_attribute(
                    "prev_withdrawal_lock_period",
                    prev_withdrawal_lock_period.to_string(),
                )
                .add_attribute(
                    "new_withdrawal_lock_period",
                    withdrawal_lock_period.to_string(),
                ),
        ))
    }

    pub fn slashing_request(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        data: SlashingRequestPayload,
    ) -> Result<Response, ContractError> {
        // service is the sender
        let service = info.sender;

        let operator: Operator = deps.api.addr_validate(data.operator.as_str())?;

        let registry = state::get_registry(deps.storage)?;

        // TODO: check if needed, need to add a lower bound?
        // ensure that metadata.reason is not empty
        if data.metadata.reason.is_empty() {
            return Err(InvalidSlashingRequest {
                msg: "Reason is empty.".to_string(),
            });
        }

        // ensure that metadata.reason does not exceed X bytes
        if data.metadata.reason.len() > MAX_STRING_BYTES {
            return Err(InvalidSlashingRequest {
                msg: "Reason is too long.".to_string(),
            });
        }

        // ensure timestamp of slash must not be more than MAX_SLASHABLE_DELAY before and not in the future
        if data.timestamp < env.block.time.minus_seconds(MAX_SLASHABLE_DELAY)
            || data.timestamp > env.block.time
        {
            return Err(InvalidSlashingRequest {
                msg: "Slash timestamp is outside of the allowable slash period.".to_string(),
            });
        }

        // require active status between operator and service
        let StatusResponse(operator_service_status) = deps.querier.query_wasm_smart(
            registry.to_string(),
            &bvs_registry::msg::QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: Some(data.timestamp.seconds()),
            },
        )?;
        if operator_service_status != u8::from(RegistrationStatus::Active) {
            return Err(InvalidSlashingRequest {
                msg: "Service and Operator are not active at timestamp.".to_string(),
            });
        }

        // get slashing params of the service at the given timestamp, also checks if slashing is enabled
        let SlashingParametersResponse(slashing_parameters) = deps.querier.query_wasm_smart(
            registry.clone(),
            &bvs_registry::msg::QueryMsg::SlashingParameters {
                service: service.to_string(),
                timestamp: Some(data.timestamp.seconds()),
            },
        )?;
        let slashing_parameters = match slashing_parameters {
            Some(x) => x,
            None => {
                return Err(InvalidSlashingRequest {
                    msg: "Service has not enabled slashing at timestamp.".to_string(),
                })
            }
        };

        // ensure bips must not exceed max_slashing_bips set by service
        if data.bips > slashing_parameters.max_slashing_bips {
            return Err(InvalidSlashingRequest {
                msg: "Slashing bips is over max_slashing_bips set.".to_string(),
            });
        }

        // ensure that the operator has opted in to slashing at the given timestamp.
        let IsOperatorOptedInToSlashingResponse(is_operated_opted_in) =
            deps.querier.query_wasm_smart(
                registry,
                &bvs_registry::msg::QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: Some(data.timestamp.seconds()),
                },
            )?;

        if !is_operated_opted_in {
            return Err(InvalidSlashingRequest {
                msg: "Operator has not opted-in to slashing at timestamp.".to_string(),
            });
        }

        // get current active slashing request for (service, operator) pair
        let active_slashing_requests =
            state::get_active_slashing_requests(deps.storage, &service, &operator)?;

        // if active slashing request exists and
        // if request_expiry > now (in the future) => request hasn't expired,
        // so the service has to manually cancel active request (throw Err)
        if let Some(active_slashing_requests) = active_slashing_requests {
            if active_slashing_requests.request_expiry > env.block.time {
                return Err(InvalidSlashingRequest {
                    msg: "Service has current active slashing request for the operator."
                        .to_string(),
                });
            }
        }

        // else, it will be overriden by the current slash request

        // New_request_expiry will be using `resolution_window`
        // value from the timestamp's slashing_parameters,
        // instead of the most recent slashing param.
        // This ensures that both parties agree upon all parameters used.
        let new_request_expiry = env
            .block
            .time
            .plus_seconds(slashing_parameters.resolution_window * 2);
        let new_request = SlashingRequest::new(data.clone(), env.block.time, new_request_expiry);

        // save slash data
        let slashing_id =
            state::save_slashing_request(deps.storage, &service, &operator, &new_request)?;

        Ok(Response::new().add_event(
            Event::new("SlashingRequest")
                .add_attribute("service", service)
                .add_attribute("operator", operator)
                .add_attribute("slashing_id", slashing_id.to_string())
                .add_attribute("reason", data.metadata.reason),
        ))
    }
}

/// Snipped implementation of Vault's API
pub mod vault {
    use crate::error::ContractError;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Deps};

    #[cw_serde]
    pub enum VaultInfoQueryMsg {
        VaultInfo {},
    }

    /// This is a snippet of the struct to avoid circular dependencies.
    /// This should be kept in sync with the original struct.
    /// See [`VaultInfoResponse`] for more information.
    #[cw_serde]
    pub struct VaultInfoResponse {
        /// The `vault-router` contract address
        pub router: Addr,

        /// The `operator` that this vault is delegated to
        pub operator: Addr,
    }

    pub fn get_vault_info(deps: Deps, vault: &Addr) -> Result<VaultInfoResponse, ContractError> {
        match deps
            .querier
            .query_wasm_smart(vault.to_string(), &VaultInfoQueryMsg::VaultInfo {})
        {
            Ok(response) => Ok(response),
            Err(_) => Err(ContractError::VaultError {
                msg: format!("No such contract: {}", vault).to_string(),
            }),
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
        QueryMsg::ListVaultsByOperator {
            operator,
            limit,
            start_after,
        } => {
            let limit = limit.map_or(100, |v| v.min(100));
            let start_after = start_after
                .map(|s| deps.api.addr_validate(&s))
                .transpose()?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::list_vaults_by_operator(
                deps,
                operator,
                limit,
                start_after,
            )?)
        }
        QueryMsg::WithdrawalLockPeriod {} => {
            to_json_binary(&query::get_withdrawal_lock_period(deps)?)
        }
        QueryMsg::SlashingRequestId { service, operator } => {
            let service = deps.api.addr_validate(&service)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::slashing_request_id(deps, service, operator)?)
        }
        QueryMsg::SlashingRequest(id) => to_json_binary(&query::slashing_request(deps, id)?),
    }
}

mod query {
    use crate::msg::{
        SlashingRequestIdResponse, SlashingRequestResponse, Vault, VaultListResponse,
    };
    use crate::state::{
        self, SlashingRequestId, DEFAULT_WITHDRAWAL_LOCK_PERIOD, SLASHING_REQUESTS,
        SLASHING_REQUEST_IDS,
    };
    use bvs_registry::msg::QueryMsg;
    use cosmwasm_std::{Addr, Deps, StdResult, Uint64};
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
        let registry = state::get_registry(deps.storage)?;
        let is_operator_active: bool = deps.querier.query_wasm_smart(
            registry.to_string(),
            &QueryMsg::IsOperatorActive(operator.to_string()),
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

    pub fn get_withdrawal_lock_period(deps: Deps) -> StdResult<Uint64> {
        let value = state::WITHDRAWAL_LOCK_PERIOD
            .may_load(deps.storage)?
            .unwrap_or(DEFAULT_WITHDRAWAL_LOCK_PERIOD);

        Ok(value)
    }

    pub fn list_vaults_by_operator(
        deps: Deps,
        operator: Addr,
        limit: u32,
        start_after: Option<Addr>,
    ) -> StdResult<VaultListResponse> {
        let items = state::OPERATOR_VAULTS.prefix(&operator);

        let range_max = start_after.as_ref().map(Bound::exclusive);
        let items = items.range(
            deps.storage,
            None,
            range_max,
            cosmwasm_std::Order::Descending,
        );

        let vaults = items
            .take(limit as usize)
            .map(|item| {
                let (vault_address, _) = item?;
                let whitelisted = state::VAULTS
                    .load(deps.storage, &vault_address)?
                    .whitelisted;
                Ok(Vault {
                    vault: vault_address,
                    whitelisted,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(VaultListResponse(vaults))
    }

    pub fn slashing_request_id(
        deps: Deps,
        service: Addr,
        operator: Addr,
    ) -> StdResult<SlashingRequestIdResponse> {
        let active_slashing_request_id =
            SLASHING_REQUEST_IDS.may_load(deps.storage, (&service, &operator))?;
        Ok(SlashingRequestIdResponse(active_slashing_request_id))
    }

    pub fn slashing_request(
        deps: Deps,
        id: impl Into<SlashingRequestId>,
    ) -> StdResult<SlashingRequestResponse> {
        let active_slashing_request = SLASHING_REQUESTS.may_load(deps.storage, id.into())?;
        Ok(SlashingRequestResponse(active_slashing_request))
    }
}

#[cfg(test)]
mod tests {
    use super::vault::{VaultInfoQueryMsg, VaultInfoResponse};
    use super::*;
    use super::{
        execute::{set_vault, set_withdrawal_lock_period},
        query::{get_withdrawal_lock_period, is_validating, is_whitelisted, list_vaults},
    };
    use crate::msg::SlashingMetadata;
    use crate::msg::SlashingRequestPayload;
    use crate::msg::{InstantiateMsg, SlashingRequestIdResponse, SlashingRequestResponse};
    use crate::state::{SlashingRequest, SLASHING_REQUESTS};
    use crate::state::{
        SlashingRequestId, Vault, OPERATOR_VAULTS, REGISTRY, SLASHING_REQUEST_IDS, VAULTS,
    };
    use bvs_registry::msg::{IsOperatorActiveResponse, QueryMsg as RegistryQueryMsg};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, Attribute, ContractResult, Event, HexBinary, OwnedDeps, QuerierResult,
        SystemError, SystemResult, Uint64, WasmQuery,
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

        let moved_env = env.clone();
        deps.querier
            .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                if let WasmQuery::Smart { contract_addr, msg } = req {
                    if *contract_addr == deps.api.addr_make("vault").to_string() {
                        let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                        match msg {
                            VaultInfoQueryMsg::VaultInfo {} => {
                                let response = VaultInfoResponse {
                                    router: moved_env.contract.address.clone(),
                                    operator: deps.api.addr_make("operator"),
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

        let vault_contract_addr = deps.api.addr_make("vault");
        let operator_addr = deps.api.addr_make("operator");

        // whitelist is false
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault_contract_addr.clone(),
                false,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.attributes.len(), 0);
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("VaultUpdated")
                    .add_attribute("vault", vault_contract_addr.clone())
                    .add_attribute("whitelisted", "false")
            );

            let vault = VAULTS
                .load(deps.as_ref().storage, &vault_contract_addr)
                .unwrap();
            assert!(!vault.whitelisted);

            let is_none = OPERATOR_VAULTS
                .may_load(
                    deps.as_ref().storage,
                    (&operator_addr, &vault_contract_addr),
                )
                .unwrap()
                .is_none();
            assert!(is_none);
        }

        let vault = deps.api.addr_make("vault");
        let operator_addr = deps.api.addr_make("operator");

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

            let vault = VAULTS.load(deps.as_ref().storage, &vault).unwrap();
            assert!(vault.whitelisted);

            OPERATOR_VAULTS
                .load(
                    deps.as_ref().storage,
                    (&operator_addr, &vault_contract_addr),
                )
                .unwrap();
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
                format!("Vault error: No such contract: {}", empty_vault)
            );
        }

        // whitelist is true and failed to set: Vault is not connected to the router
        let _new_vault = deps.api.addr_make("new_vault");
        {
            deps.querier
                .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                    if let WasmQuery::Smart { contract_addr, msg } = req {
                        if *contract_addr == deps.api.addr_make("vault").to_string() {
                            let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                            match msg {
                                VaultInfoQueryMsg::VaultInfo {} => {
                                    let response = VaultInfoResponse {
                                        router: vault_contract_addr.clone(),
                                        operator: operator_addr.clone(),
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
    fn test_set_and_get_withdrawal_lock_period() {
        let (mut deps, env, owner_info) = instantiate_contract();

        let withdrawal_lock_period = Uint64::new(120);

        // set withdrawal lock period successfully
        {
            let result = set_withdrawal_lock_period(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                withdrawal_lock_period,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("SetWithdrawalLockPeriod")
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        Uint64::new(604800).to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period.to_string()
                    )
            );
        }

        let withdrawal_lock_period1 = Uint64::new(150);

        // update withdrawal lock period successfully
        {
            let result = set_withdrawal_lock_period(
                deps.as_mut(),
                env.clone(),
                owner_info,
                withdrawal_lock_period1,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("SetWithdrawalLockPeriod")
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        withdrawal_lock_period.to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    )
            );
        }

        // query withdrawal lock period
        {
            let result = get_withdrawal_lock_period(deps.as_ref()).unwrap();
            assert_eq!(result, withdrawal_lock_period1);
        }

        // wrong permission to update withdrawal lock period successfully
        {
            let user_info = MessageInfo {
                sender: deps.api.addr_make("user"),
                funds: vec![],
            };
            let result =
                set_withdrawal_lock_period(deps.as_mut(), env, user_info, withdrawal_lock_period1);
            assert!(result.is_err());
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
                            RegistryQueryMsg::IsOperatorActive(_operator) => {
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

    #[test]
    fn test_query_list_vaults_by_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let mut vaults = Vec::new();

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault{}", i));
            vaults.push(vault.clone());
            VAULTS
                .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
                .unwrap();
            OPERATOR_VAULTS
                .save(&mut deps.storage, (&operator, &vault), &())
                .unwrap();
        }

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 100, None).unwrap();
        assert_eq!(response.0.len(), 10);

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 5, None).unwrap();
        assert_eq!(response.0.len(), 5);

        let mut response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 10, None).unwrap();
        assert_eq!(response.0.len(), 10);

        // check if those address came back the same
        // let's sort them first for easier comparison
        vaults.sort();
        response.0.sort_by(|a, b| a.vault.cmp(&b.vault));

        for i in 0..10 {
            assert_eq!(response.0[i].vault, vaults[i]);
        }

        // let's test pagination sync this time
        {
            let response1 =
                query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 5, None).unwrap();
            assert_eq!(response1.0.len(), 5);

            let next_start_after = response1.0[4].vault.clone();

            let response2 = query::list_vaults_by_operator(
                deps.as_ref(),
                operator.clone(),
                5,
                Some(next_start_after),
            )
            .unwrap();

            assert_eq!(response2.0.len(), 5);

            let mut total_vaults = response1
                .0
                .iter()
                .chain(response2.0.iter())
                .collect::<Vec<_>>();

            total_vaults.sort_by(|a, b| a.vault.cmp(&b.vault));

            for i in 0..10 {
                assert_eq!(total_vaults[i].vault, vaults[i]);
            }
        }

        // let's have another operator with its own vaults
        let operator2 = deps.api.addr_make("operator2");

        let mut vaults2 = Vec::new();

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault2{}", i));
            vaults2.push(vault.clone());
            VAULTS
                .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
                .unwrap();
            OPERATOR_VAULTS
                .save(&mut deps.storage, (&operator2, &vault), &())
                .unwrap();
        }

        let mut response =
            query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 10);

        // check if those address came back the same
        // let's sort them first for easier comparison
        vaults2.sort();
        response.0.sort_by(|a, b| a.vault.cmp(&b.vault));

        for i in 0..10 {
            assert_eq!(response.0[i].vault, vaults2[i]);
        }

        // let's test pagination sync for operator2's vaults as well
        {
            let response1 =
                query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 5, None).unwrap();
            assert_eq!(response1.0.len(), 5);

            let next_start_after = response1.0[4].vault.clone();

            let response2 = query::list_vaults_by_operator(
                deps.as_ref(),
                operator2.clone(),
                5,
                Some(next_start_after),
            )
            .unwrap();

            assert_eq!(response2.0.len(), 5);

            let mut total_vaults = response1
                .0
                .iter()
                .chain(response2.0.iter())
                .collect::<Vec<_>>();

            total_vaults.sort_by(|a, b| a.vault.cmp(&b.vault));

            for i in 0..10 {
                assert_eq!(total_vaults[i].vault, vaults2[i]);
            }
        }

        // we should have a total of 20 vaults
        let response = query::list_vaults(deps.as_ref(), 100, None).unwrap();
        assert_eq!(response.0.len(), 20);

        let non_operator = deps.api.addr_make("non_operator");

        let response =
            query::list_vaults_by_operator(deps.as_ref(), non_operator.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 0);
    }

    #[test]
    fn test_map_vault_migration() {
        let mut deps = mock_dependencies();

        let operator1 = deps.api.addr_make("operator");
        let vault1 = deps.api.addr_make("vault1");

        let operator2 = deps.api.addr_make("operator2");
        let vault2 = deps.api.addr_make("vault2");

        {
            let moved_operator1 = operator1.clone();
            let moved_vault1 = vault1.to_string();

            let moved_operator2 = operator2.clone();
            let moved_vault2 = vault2.to_string();
            deps.querier
                .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                    if let WasmQuery::Smart { contract_addr, msg } = req {
                        let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                        let contract_addr = contract_addr.to_string();
                        let operator_addr = {
                            if contract_addr == moved_vault1 {
                                moved_operator1.clone()
                            } else if contract_addr == moved_vault2 {
                                moved_operator2.clone()
                            } else {
                                panic!("Unknown contract address")
                            }
                        };
                        match msg {
                            VaultInfoQueryMsg::VaultInfo {} => {
                                let response = VaultInfoResponse {
                                    router: deps.api.addr_make("router"),
                                    operator: operator_addr,
                                };
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                        }
                    } else {
                        SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Unsupported query".to_string(),
                        })
                    }
                });

            // operator1's vault
            VAULTS
                .save(&mut deps.storage, &vault1, &Vault { whitelisted: true })
                .unwrap();

            // operator2's vault
            VAULTS
                .save(&mut deps.storage, &vault2, &Vault { whitelisted: true })
                .unwrap();
        }

        // let's run the migration
        migrate::vaults_to_index_operator(deps.as_mut()).unwrap();

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator1.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].vault, vault1);

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].vault, vault2);
    }

    #[test]
    fn test_get_withdrawal_lock_period() {
        let deps = mock_dependencies();

        let response = get_withdrawal_lock_period(deps.as_ref()).unwrap();
        assert_eq!(response, Uint64::new(604800));
    }

    #[test]
    fn test_query_slashing_request_id() {
        let mut deps = mock_dependencies();
        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let request_id = SlashingRequestId(
            HexBinary::from_hex("dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );

        // query slashing request id before its saved => None
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator.clone()).unwrap();
        assert_eq!(res, None);

        // save to state
        let key = (&service, &operator);
        SLASHING_REQUEST_IDS
            .save(&mut deps.storage, key, &request_id)
            .expect("save slashing request id failed");

        // assert slash request id exists
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator.clone()).unwrap();
        assert_eq!(res, Some(request_id));

        // assert that slash request id for a different operator is None
        let operator2 = deps.api.addr_make("operator2");
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator2).unwrap();
        assert_eq!(res, None);

        // assert that slash request id for a different service is None
        let service2 = deps.api.addr_make("service2");
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service2, operator).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn test_query_slashing_request() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let request_id = SlashingRequestId(
            HexBinary::from_hex("dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );
        let slash_request = SlashingRequest::new(
            SlashingRequestPayload {
                operator: operator.to_string(),
                bips: 100,
                timestamp: env.block.time,
                metadata: SlashingMetadata {
                    reason: "test".to_string(),
                },
            },
            env.block.time,
            env.block.time.plus_seconds(100),
        );

        // query request_id before its saved => None
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), request_id.clone()).unwrap();
        assert_eq!(res, None);

        // save request_id
        SLASHING_REQUESTS
            .save(&mut deps.storage, request_id.clone(), &slash_request)
            .expect("save slashing request failed");

        // assert request_id is saved
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), request_id.clone()).unwrap();
        assert_eq!(res, Some(slash_request));

        // assert that another request_id is not saved
        let random_request_id = SlashingRequestId(
            HexBinary::from_hex("0ff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), random_request_id).unwrap();
        assert_eq!(res, None);
    }
}
