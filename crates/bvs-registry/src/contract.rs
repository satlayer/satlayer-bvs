#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::{migration, state};
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

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
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
        ExecuteMsg::RegisterAsService { metadata } => {
            execute::register_as_service(deps, info, metadata)
        }
        ExecuteMsg::UpdateServiceMetadata(metadata) => {
            execute::service_update_metadata(deps, info, metadata)
        }
        ExecuteMsg::RegisterAsOperator { metadata } => {
            execute::register_as_operator(deps, info, metadata)
        }
        ExecuteMsg::UpdateOperatorMetadata(metadata) => {
            execute::update_operator_metadata(deps, info, metadata)
        }
        ExecuteMsg::RegisterOperatorToService { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::register_operator_to_service(deps, info, env, operator)
        }
        ExecuteMsg::DeregisterOperatorFromService { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::deregister_operator_from_service(deps, info, env, operator)
        }
        ExecuteMsg::RegisterServiceToOperator { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::register_service_to_operator(deps, info, env, service)
        }
        ExecuteMsg::DeregisterServiceFromOperator { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::deregister_service_from_operator(deps, info, env, service)
        }
        ExecuteMsg::EnableSlashing {
            slashing_parameters,
        } => execute::enable_slashing(deps, env, info, slashing_parameters),
        ExecuteMsg::DisableSlashing {} => execute::disable_slashing(deps, env, info),
        ExecuteMsg::OperatorOptInToSlashing { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::operator_opt_in_to_slashing(deps, env, info, service)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

mod execute {
    use crate::error::ContractError;
    use crate::msg::Metadata;
    use crate::state;
    use crate::state::{
        get_registration_status, is_slashing_enabled, require_active_registration_status,
        require_operator_registered, require_service_registered, set_registration_status,
        RegistrationStatus, SlashingParameters, OPERATORS, SERVICES,
    };
    use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response};

    /// Event for MetadataUpdated
    fn create_metadata_event(metadata: Metadata) -> Event {
        let mut event = Event::new("MetadataUpdated");

        if let Some(uri) = metadata.uri {
            event = event.add_attribute("metadata.uri", uri);
        }

        if let Some(name) = metadata.name {
            event = event.add_attribute("metadata.name", name);
        }

        event
    }

    /// Register `info.sender` as a service.
    pub fn register_as_service(
        deps: DepsMut,
        info: MessageInfo,
        metadata: Metadata,
    ) -> Result<Response, ContractError> {
        let registered = SERVICES
            .may_load(deps.storage, &info.sender)?
            .unwrap_or(false);

        if registered {
            return Err(ContractError::ServiceRegistered {});
        }

        SERVICES.save(deps.storage, &info.sender, &true)?;

        let metadata_event =
            create_metadata_event(metadata).add_attribute("service", info.sender.clone());

        Ok(Response::new()
            .add_event(
                Event::new("ServiceRegistered").add_attribute("service", info.sender.clone()),
            )
            .add_event(metadata_event))
    }

    /// Update service metadata (info.sender is the service)
    pub fn service_update_metadata(
        deps: DepsMut,
        info: MessageInfo,
        metadata: Metadata,
    ) -> Result<Response, ContractError> {
        require_service_registered(deps.storage, &info.sender)?;

        let metadata_event =
            create_metadata_event(metadata).add_attribute("service", info.sender.clone());

        Ok(Response::new().add_event(metadata_event))
    }

    /// Registers the `info.sender` as an operator.
    ///
    /// `Metadata` is never stored and is only emitted in the `MetadataUpdated` event.
    pub fn register_as_operator(
        deps: DepsMut,
        info: MessageInfo,
        metadata: Metadata,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();

        // error if the operator is already registered
        let operator_registered = OPERATORS
            .may_load(deps.storage, &operator)?
            .unwrap_or(false);
        if operator_registered {
            return Err(ContractError::OperatorRegistered {});
        }

        // add operator into the state
        OPERATORS.save(deps.storage, &operator, &true)?;

        let mut response = Response::new();

        let register_event =
            Event::new("OperatorRegistered").add_attribute("operator", operator.to_string());
        response = response.add_event(register_event);

        let metadata_event =
            create_metadata_event(metadata).add_attribute("operator", operator.to_string());
        response = response.add_event(metadata_event);

        Ok(response)
    }

    /// Called by an operator to emit a `MetadataUpdated` event indicating the information has updated.
    pub fn update_operator_metadata(
        deps: DepsMut,
        info: MessageInfo,
        metadata: Metadata,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();
        require_operator_registered(deps.storage, &operator)?;

        let mut response = Response::new();
        let metadata_event =
            create_metadata_event(metadata).add_attribute("operator", operator.to_string());

        response = response.add_event(metadata_event);

        Ok(response)
    }

    /// Register an operator to a service (info.sender is the service)
    /// Service must be registered via [`super::ExecuteMsg::RegisterAsService`].  
    /// If the operator has registered this service, the registration status will be set to [`RegistrationStatus::Active`] (1)  
    /// Else the registration status will be set to [`RegistrationStatus::ServiceRegistered`] (3)
    pub fn register_operator_to_service(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let service = info.sender.clone();
        require_service_registered(deps.storage, &service)?;
        require_operator_registered(deps.storage, &operator)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;
        match status {
            RegistrationStatus::Active => Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration between operator and service is already active".to_string(),
            }),
            RegistrationStatus::ServiceRegistered => {
                Err(ContractError::InvalidRegistrationStatus {
                    msg: "Service has already registered this operator".to_string(),
                })
            }
            RegistrationStatus::Inactive => {
                set_registration_status(
                    deps.storage,
                    &env,
                    key,
                    RegistrationStatus::ServiceRegistered,
                )?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_operator_to_service")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "ServiceRegistered"),
                ))
            }
            RegistrationStatus::OperatorRegistered => {
                set_registration_status(deps.storage, &env, key, RegistrationStatus::Active)?;

                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_operator_to_service")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "Active"),
                ))
            }
        }
    }

    /// Deregister an operator from a service (info.sender is the service)
    /// Set the registration status to [`RegistrationStatus::Inactive`] (0)
    pub fn deregister_operator_from_service(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let service = info.sender.clone();
        require_service_registered(deps.storage, &service)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Operator is not registered with this service".to_string(),
            })
        } else {
            set_registration_status(deps.storage, &env, key, RegistrationStatus::Inactive)?;

            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "deregister_operator_from_service")
                    .add_attribute("operator", operator)
                    .add_attribute("service", service)
                    .add_attribute("status", "Inactive"),
            ))
        }
    }

    /// Register a service to an operator (info.sender is the operator)
    /// Operator must be registered with [`ExecuteMsg::RegisterAsOperator`]
    /// If the service has registered this operator, the registration status will be set to [`RegistrationStatus::Active`] (1)
    /// Else the registration status will be set to [`RegistrationStatus::OperatorRegistered`] (2)
    pub fn register_service_to_operator(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        service: Addr,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();
        require_service_registered(deps.storage, &service)?;
        require_operator_registered(deps.storage, &operator)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;

        match status {
            RegistrationStatus::Active => Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration between operator and service is already active".to_string(),
            }),
            RegistrationStatus::OperatorRegistered => {
                Err(ContractError::InvalidRegistrationStatus {
                    msg: "Operator has already registered this service".to_string(),
                })
            }
            RegistrationStatus::Inactive => {
                set_registration_status(
                    deps.storage,
                    &env,
                    key,
                    RegistrationStatus::OperatorRegistered,
                )?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_service_to_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "OperatorRegistered"),
                ))
            }
            RegistrationStatus::ServiceRegistered => {
                set_registration_status(deps.storage, &env, key, RegistrationStatus::Active)?;

                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_service_to_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "Active"),
                ))
            }
        }
    }

    /// Deregister a service from an operator (info.sender is the Operator)
    /// Set the registration status to [`RegistrationStatus::Inactive`] (0)
    pub fn deregister_service_from_operator(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        service: Addr,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Service is not registered with this operator".to_string(),
            })
        } else {
            set_registration_status(deps.storage, &env, key, RegistrationStatus::Inactive)?;

            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "deregister_service_from_operator")
                    .add_attribute("operator", operator)
                    .add_attribute("service", service)
                    .add_attribute("status", "Inactive"),
            ))
        }
    }

    /// Enable slashing for a service by registering slashing parameters into the registry.
    ///
    /// When slashing is enabled, active operators are able to opt in the next block.
    /// New Operator <-> Service registration will automatically opt in the operator to slashing.
    /// To update the slashing parameters, the service must call this function again.
    /// When the slashing parameters are updated,
    /// all active operators
    /// that are already registered to the service will have their opt-in status reset for the new slashing parameters
    /// and have to manually opt in again.
    pub fn enable_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        slashing_parameters: SlashingParameters,
    ) -> Result<Response, ContractError> {
        // service is the sender
        let service = info.sender;

        // service must be registered
        require_service_registered(deps.storage, &service)?;

        // clear opt-in mapping
        state::reset_slashing_opt_in(deps.storage, &env, &service)?;

        // update slashing parameters
        state::enable_slashing(deps.storage, deps.api, &env, &service, &slashing_parameters)?;

        Ok(Response::new().add_event(
            Event::new("SlashingParametersEnabled")
                .add_attribute("service", service)
                .add_attribute(
                    "destination",
                    slashing_parameters
                        .destination
                        .map(|x| x.to_string())
                        .unwrap_or_default(),
                )
                .add_attribute(
                    "max_slashing_bips",
                    slashing_parameters.max_slashing_bips.to_string(),
                )
                .add_attribute(
                    "resolution_window",
                    slashing_parameters.resolution_window.to_string(),
                ),
        ))
    }

    /// Disable slashing for a service by removing slashing parameters from the registry.
    ///
    /// When slashing is disabled,
    /// all operators
    /// that are opted in to the slashing parameters will be removed from the slashing opt in.
    /// All active operators will remain actively registered to the service.
    pub fn disable_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // service is the sender
        let service = info.sender;

        // service must be registered
        require_service_registered(deps.storage, &service)?;

        // clear opt-in mapping
        state::reset_slashing_opt_in(deps.storage, &env, &service)?;

        // remove slashing parameters
        state::disable_slashing(deps.storage, &env, &service)?;

        Ok(Response::new()
            .add_event(Event::new("SlashingParametersDisabled").add_attribute("service", service)))
    }

    /// Operator opts in to the service's current slashing parameters.
    ///
    /// When slashing is enabled, active operators can opt in to slashing.  
    /// Newly registered operators, after slashing is enabled, will automatically opt in to slashing
    /// and don't need to call this function.
    pub fn operator_opt_in_to_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        service: Addr,
    ) -> Result<Response, ContractError> {
        // operator is the sender
        let operator = info.sender;

        // operator and service must have Active (1) registration status
        let key = (&operator, &service);
        require_active_registration_status(deps.storage, key)?;

        // check if the slashing is enabled for the service
        if !is_slashing_enabled(deps.storage, &service, Some(env.block.time.seconds()))? {
            return Err(ContractError::InvalidSlashingOptIn {
                msg: "Cannot opt in: slashing is not enabled for this service".to_string(),
            });
        }

        // opt-in to slashing
        state::opt_in_to_slashing(deps.storage, &env, &service, &operator)?;

        Ok(Response::new().add_event(
            Event::new("OperatorOptedInToSlashing")
                .add_attribute("operator", operator)
                .add_attribute("service", service),
        ))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status {
            service,
            operator,
            timestamp,
        } => {
            let service = deps.api.addr_validate(&service)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::status(deps, operator, service, timestamp)?)
        }
        QueryMsg::IsService(service) => {
            let service = deps.api.addr_validate(&service)?;
            to_json_binary(&query::is_service(deps, service)?)
        }
        QueryMsg::IsOperator(operator) => {
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_operator(deps, operator)?)
        }
        QueryMsg::IsOperatorActive(operator) => {
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_operator_active(deps, operator)?)
        }
        QueryMsg::SlashingParameters { service, timestamp } => {
            let service = deps.api.addr_validate(&service)?;
            to_json_binary(&query::get_slashing_parameters(deps, service, timestamp)?)
        }
        QueryMsg::IsOperatorOptedInToSlashing {
            service,
            operator,
            timestamp,
        } => {
            let service = deps.api.addr_validate(&service)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_operator_opted_in_to_slashing(
                deps, service, operator, timestamp,
            )?)
        }
        QueryMsg::ActiveOperatorsCount { service } => {
            let service = deps.api.addr_validate(&service)?;
            let count = state::SERVICE_ACTIVE_OPERATORS_COUNT
                .may_load(deps.storage, &service)?
                .unwrap_or(0);
            to_json_binary(&count)
        }
        QueryMsg::ActiveServicesCount { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            let count = state::OPERATOR_ACTIVE_REGISTRATION_COUNT
                .may_load(deps.storage, &operator)?
                .unwrap_or(0);
            to_json_binary(&count)
        }
    }
}

mod query {
    use crate::msg::{
        IsOperatorActiveResponse, IsOperatorOptedInToSlashingResponse, IsOperatorResponse,
        IsServiceResponse, SlashingParametersResponse, StatusResponse,
    };
    use crate::state;
    use crate::state::{
        require_operator_registered, require_service_registered, SLASHING_PARAMETERS,
    };
    use cosmwasm_std::{Addr, Deps, StdResult};

    /// Get the registration status of an operator to a service at a given timestamp.
    /// If timestamp is `None`, it will return the current registration status.  
    /// Returns: [`StdResult<StatusResponse>`]
    /// - [`RegistrationStatus::Inactive`] (0) if not registered
    /// - [`RegistrationStatus::Active`] (1) if registration is active (operator and service are registered to each other)
    /// - [`RegistrationStatus::OperatorRegistered`] (2) if operator is registered to service, pending service registration
    /// - [`RegistrationStatus::ServiceRegistered`] (3) if service is registered to operator, pending operator registration
    pub fn status(
        deps: Deps,
        operator: Addr,
        service: Addr,
        timestamp: Option<u64>,
    ) -> StdResult<StatusResponse> {
        let key = (&operator, &service);
        let status = match timestamp {
            Some(t) => state::get_registration_status_at_timestamp(deps.storage, key, t)?,
            None => state::get_registration_status(deps.storage, key)?,
        };
        Ok(status.into())
    }

    /// Query if the service is registered or not.
    pub fn is_service(deps: Deps, service: Addr) -> StdResult<IsServiceResponse> {
        let is_service_registered = require_service_registered(deps.storage, &service).is_ok();

        Ok(IsServiceResponse(is_service_registered))
    }

    /// Query if the operator is registered or not.
    pub fn is_operator(deps: Deps, operator: Addr) -> StdResult<IsOperatorResponse> {
        let is_operator_registered = require_operator_registered(deps.storage, &operator).is_ok();

        Ok(IsOperatorResponse(is_operator_registered))
    }

    /// Query if the operator is actively registered to any service
    pub fn is_operator_active(deps: Deps, operator: Addr) -> StdResult<IsOperatorActiveResponse> {
        let is_operator_active = state::is_operator_active(deps.storage, &operator)?;

        Ok(IsOperatorActiveResponse(is_operator_active))
    }

    /// Query the slashing registry for a service
    pub fn get_slashing_parameters(
        deps: Deps,
        service: Addr,
        timestamp: Option<u64>,
    ) -> StdResult<SlashingParametersResponse> {
        let slashing_parameters = match timestamp {
            Some(t) => SLASHING_PARAMETERS.may_load_at_height(deps.storage, &service, t)?,
            None => SLASHING_PARAMETERS.may_load(deps.storage, &service)?,
        };
        Ok(SlashingParametersResponse(slashing_parameters))
    }

    /// Query if the operator is opted in to slashing
    pub fn is_operator_opted_in_to_slashing(
        deps: Deps,
        service: Addr,
        operator: Addr,
        timestamp: Option<u64>,
    ) -> StdResult<IsOperatorOptedInToSlashingResponse> {
        let is_opted_in =
            state::is_operator_opted_in_to_slashing(deps.storage, &service, &operator, timestamp)?;
        Ok(IsOperatorOptedInToSlashingResponse(is_opted_in))
    }
}

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 2.0.0
/// - Internal migrate of [REGISTRATION_STATUS] state from `Map` to `SnapshotMap`.
///   Storage mapping is not needed because SnapshotMap uses a map with the same namespace.
///   This migration will also mean
///   that the current state of REGISTRATION_STATUS will be the default state
///   when querying for state from earlier timestamp.
///   For instance, migration happens at block 200 and Operator1 and Service1 is in Active status.
///   When queried at genesis, the status of Operator1 and Service1 will be Active.
/// - No storage migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: Option<MigrateMsg>,
) -> Result<Response, ContractError> {
    let old_version =
        cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match old_version.minor {
        3 => {
            migration::fill_service_active_operators_count(deps)?;
            Ok(Response::default())
        }
        _ => Ok(Response::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute::{
        enable_slashing, register_operator_to_service, register_service_to_operator,
    };
    use crate::contract::query::status;
    use crate::msg::{
        InstantiateMsg, IsOperatorActiveResponse, IsOperatorOptedInToSlashingResponse,
        IsOperatorResponse, IsServiceResponse, Metadata, SlashingParametersResponse,
        StatusResponse,
    };
    use crate::state;
    use crate::state::{
        increase_operator_active_registration_count, set_registration_status, RegistrationStatus,
        SlashingParameters, OPERATORS, REGISTRATION_STATUS, SERVICES, SLASHING_OPT_IN,
        SLASHING_PARAMETERS,
    };
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Event, OwnedDeps, Response, StdError};

    fn mock_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let pauser = deps.api.addr_make("pauser");
        let owner_info = message_info(&owner, &[]);

        instantiate(
            deps.as_mut(),
            env,
            owner_info.clone(),
            InstantiateMsg {
                owner: owner.to_string(),
                pauser: pauser.to_string(),
            },
        )
        .unwrap();

        deps
    }

    #[test]
    fn test_register_as_service() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        let res = execute::register_as_service(
            deps.as_mut(),
            service_info,
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("name".to_string()),
            },
        );

        assert_eq!(
            res,
            Ok(Response::new()
                .add_event(
                    Event::new("ServiceRegistered").add_attribute("service", service.as_ref())
                )
                .add_event(
                    Event::new("MetadataUpdated")
                        .add_attribute("metadata.uri", "uri")
                        .add_attribute("metadata.name", "name")
                        .add_attribute("service", service.as_ref())
                ))
        );

        let registered = SERVICES.load(&deps.storage, &service).unwrap();
        assert!(registered);
    }

    #[test]
    fn test_register_service_optional_metadata() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        let res = execute::register_as_service(
            deps.as_mut(),
            service_info,
            Metadata {
                uri: None,
                name: Some("Meta Bridging".to_string()),
            },
        );

        assert_eq!(
            res,
            Ok(Response::new()
                .add_event(
                    Event::new("ServiceRegistered").add_attribute("service", service.as_ref())
                )
                .add_event(
                    Event::new("MetadataUpdated")
                        .add_attribute("metadata.name", "Meta Bridging")
                        .add_attribute("service", service.as_ref())
                ))
        );

        let registered = SERVICES.load(&deps.storage, &service).unwrap();
        assert!(registered);
    }

    #[test]
    fn test_service_update_metadata() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        let res = execute::service_update_metadata(
            deps.as_mut(),
            service_info,
            Metadata {
                uri: Some("new_uri".to_string()),
                name: Some("new_name".to_string()),
            },
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("MetadataUpdated")
                    .add_attribute("metadata.uri", "new_uri")
                    .add_attribute("metadata.name", "new_name")
                    .add_attribute("service", service.clone())
            ))
        );
    }

    #[test]
    fn test_register_as_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        let res = execute::register_as_operator(
            deps.as_mut(),
            operator_info,
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("operator1".to_string()),
            },
        );

        assert_eq!(
            res,
            Ok(Response::new()
                .add_event(
                    Event::new("OperatorRegistered").add_attribute("operator", operator.clone())
                )
                .add_event(
                    Event::new("MetadataUpdated")
                        .add_attribute("metadata.uri", "uri")
                        .add_attribute("metadata.name", "operator1")
                        .add_attribute("operator", operator.clone())
                ))
        );

        let operator_registered = OPERATORS.load(&deps.storage, &operator).is_ok();
        assert!(operator_registered);
    }

    #[test]
    fn test_register_as_operator_more_than_once() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        {
            // register operator the first time - success
            execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            )
            .expect("register operator failed");

            let operator_registered = OPERATORS.load(&deps.storage, &operator).is_ok();
            assert!(operator_registered);
        }

        // register operator the second time - error
        let err = execute::register_as_operator(
            deps.as_mut(),
            operator_info,
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("operator1".to_string()),
            },
        );
        assert_eq!(err, Err(ContractError::OperatorRegistered {}),);
    }

    #[test]
    fn test_update_operator_metadata_uri() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        {
            // register operator the first time
            let res = execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            );

            // assert OperatorMetadataURIUpdated event
            assert_eq!(
                res,
                Ok(Response::new()
                    .add_event(
                        Event::new("OperatorRegistered")
                            .add_attribute("operator", operator.as_ref())
                    )
                    .add_event(
                        Event::new("MetadataUpdated")
                            .add_attribute("metadata.uri", "uri")
                            .add_attribute("metadata.name", "operator1")
                            .add_attribute("operator", operator.clone())
                    ))
            );
        }

        // update operator details
        let res = execute::update_operator_metadata(
            deps.as_mut(),
            operator_info,
            Metadata {
                uri: Some("uri2".to_string()),
                name: None,
            },
        );

        // assert event
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("MetadataUpdated")
                    .add_attribute("metadata.uri", "uri2") // updated uri
                    .add_attribute("operator", operator.clone())
            ))
        );
    }

    #[test]
    fn test_register_lifecycle_operator_then_service() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        // register service + operator
        execute::register_as_service(
            deps.as_mut(),
            service_info.clone(),
            Metadata {
                uri: None,
                name: None,
            },
        )
        .expect("register service failed");

        execute::register_as_operator(
            deps.as_mut(),
            operator_info.clone(),
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("operator1".to_string()),
            },
        )
        .expect("register operator failed");

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "register_service_to_operator")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "OperatorRegistered")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::OperatorRegistered);

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "register_operator_to_service")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "Active")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::Active);
    }

    #[test]
    fn test_register_lifecycle_service_then_operator() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        // register service + operator
        execute::register_as_service(
            deps.as_mut(),
            service_info.clone(),
            Metadata {
                uri: None,
                name: None,
            },
        )
        .expect("register service failed");

        execute::register_as_operator(
            deps.as_mut(),
            operator_info.clone(),
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("operator1".to_string()),
            },
        )
        .expect("register operator failed");

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "register_operator_to_service")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "ServiceRegistered")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::ServiceRegistered);

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "register_service_to_operator")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "Active")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::Active);
    }

    #[test]
    fn test_register_operator_already_registered() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator/2");
        let service = deps.api.addr_make("service/2");
        let operator_info = message_info(&operator, &[]);

        // register service + operator
        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Operator has already registered this service".to_string(),
            }),
        );
    }

    #[test]
    fn test_register_service_already_registered() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator/3");
        let service = deps.api.addr_make("service/3");
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Service has already registered this operator".to_string(),
            }),
        );
    }

    #[test]
    fn test_register_already_active() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator/4");
        let service = deps.api.addr_make("service/4");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        // register service + operator
        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration between operator and service is already active".to_string(),
            }),
        );

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration between operator and service is already active".to_string(),
            }),
        );
    }

    #[test]
    fn test_service_deregister_operator() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();
        state::increase_operator_active_registration_count(&mut deps.storage, &operator)
            .expect("failed to increase operator active registration count");

        let res = execute::deregister_operator_from_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "deregister_operator_from_service")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "Inactive")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::Inactive);
    }

    #[test]
    fn test_operator_deregister_service() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();
        state::increase_operator_active_registration_count(&mut deps.storage, &operator)
            .expect("failed to increase operator active registration count");

        let res = execute::deregister_service_from_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "deregister_service_from_operator")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "Inactive")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::Inactive);
    }

    #[test]
    fn test_already_deregistered() {
        let mut deps = mock_contract();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        // deregister for the first time - success
        execute::deregister_service_from_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        )
        .expect("failed to deregister service from operator");

        let res = execute::deregister_service_from_operator(
            deps.as_mut(),
            operator_info.clone(),
            env.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Service is not registered with this operator".to_string(),
            }),
        );

        let res = execute::deregister_operator_from_service(
            deps.as_mut(),
            service_info.clone(),
            env.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Operator is not registered with this service".to_string(),
            }),
        );
    }

    #[test]
    fn test_enable_slashing() {
        let mut deps = mock_contract();
        let env = mock_env();

        let service = deps.api.addr_make("service");
        let destination = deps.api.addr_make("destination");
        let service_info = message_info(&service, &[]);

        // register service
        execute::register_as_service(
            deps.as_mut(),
            service_info.clone(),
            Metadata {
                uri: None,
                name: None,
            },
        )
        .expect("register service failed");

        // enable slashing
        let res = execute::enable_slashing(
            deps.as_mut(),
            env,
            service_info.clone(),
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 10,
                resolution_window: 1000,
            },
        );

        // assert SLASHING_PARAMETERS state is updated
        let slashing_parameters = SLASHING_PARAMETERS
            .load(&deps.storage, &service)
            .expect("failed to load slashing parameters");
        assert_eq!(
            slashing_parameters,
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 10,
                resolution_window: 1000,
            }
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("SlashingParametersEnabled")
                    .add_attribute("service", service.as_ref())
                    .add_attribute("destination", destination.to_string())
                    .add_attribute("max_slashing_bips", "10")
                    .add_attribute("resolution_window", "1000")
            ))
        );
    }

    #[test]
    fn test_re_enable_slashing() {
        let mut deps = mock_contract();
        let env = mock_env();

        let service = deps.api.addr_make("service");
        let destination = deps.api.addr_make("destination");
        let service_info = message_info(&service, &[]);

        // register service
        execute::register_as_service(
            deps.as_mut(),
            service_info.clone(),
            Metadata {
                uri: None,
                name: None,
            },
        )
        .expect("register service failed");

        // enable slashing for the first time
        execute::enable_slashing(
            deps.as_mut(),
            env.clone(),
            service_info.clone(),
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 1000,
                resolution_window: 1000,
            },
        )
        .expect("enable slashing failed");

        // operators opt-in to slashing
        for i in 0..3 {
            let operator = deps.api.addr_make(format!("operator{i}").as_str());
            SLASHING_OPT_IN
                .save(
                    &mut deps.storage,
                    (&service, &operator),
                    &true,
                    env.block.time.seconds(),
                )
                .expect("failed to save slashing opt-in");
        }

        // re-enable slashing
        execute::enable_slashing(
            deps.as_mut(),
            env.clone(),
            service_info.clone(),
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 9999,
                resolution_window: 2000,
            },
        )
        .unwrap();

        // assert that SLASHING_PARAMETERS state is updated
        let slashing_parameters = SLASHING_PARAMETERS
            .load(&deps.storage, &service)
            .expect("failed to load slashing parameters");
        assert_eq!(
            slashing_parameters,
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 9999,
                resolution_window: 2000,
            }
        );

        // assert that the opt-in mapping is cleared
        for i in 0..3 {
            let operator = deps.api.addr_make(format!("operator{i}").as_str());
            let opt_in = SLASHING_OPT_IN
                .may_load(&deps.storage, (&service, &operator))
                .unwrap();
            assert!(opt_in.is_none());
        }
    }

    #[test]
    fn test_enable_slashing_error_not_registered() {
        let mut deps = mock_contract();
        let env = mock_env();

        let service = deps.api.addr_make("service");
        let destination = deps.api.addr_make("destination");
        let service_info = message_info(&service, &[]);

        // enable slashing
        let res = execute::enable_slashing(
            deps.as_mut(),
            env,
            service_info.clone(),
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 10,
                resolution_window: 1000,
            },
        );

        assert_eq!(res, Err(ContractError::Std(StdError::not_found("service"))));
    }

    #[test]
    fn test_operator_opt_in_to_slashing() {
        let mut deps = mock_contract();
        let mut env = mock_env();

        let operator = deps.api.addr_make("operator");
        let operator2 = deps.api.addr_make("operator2");
        let operator3 = deps.api.addr_make("operator3");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);

        // NEGATIVE test - operator <-> service does not have active registration
        {
            let err = execute::operator_opt_in_to_slashing(
                deps.as_mut(),
                env.clone(),
                operator_info.clone(),
                service.clone(),
            )
            .unwrap_err();
            assert_eq!(
                err,
                ContractError::InvalidRegistrationStatus {
                    msg: "Operator and service must have active registration".to_string()
                }
            );
        }

        // update state and register operator & operator2 + service
        {
            OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();
            OPERATORS
                .save(&mut deps.storage, &operator2, &true)
                .unwrap();
            SERVICES.save(&mut deps.storage, &service, &true).unwrap();
            set_registration_status(
                &mut deps.storage,
                &env,
                (&operator, &service),
                RegistrationStatus::Active,
            )
            .expect("failed to set registration status");
            set_registration_status(
                &mut deps.storage,
                &env,
                (&operator2, &service),
                RegistrationStatus::Active,
            )
            .expect("failed to set registration status");
            increase_operator_active_registration_count(&mut deps.storage, &operator)
                .expect("failed to increase operator active registration count");
            increase_operator_active_registration_count(&mut deps.storage, &operator)
                .expect("failed to increase operator active registration count");
        }

        // NEGATIVE test - slashing not enabled
        {
            let err = execute::operator_opt_in_to_slashing(
                deps.as_mut(),
                env.clone(),
                operator_info.clone(),
                service.clone(),
            )
            .unwrap_err();
            assert_eq!(
                err,
                ContractError::InvalidSlashingOptIn {
                    msg: "Cannot opt in: slashing is not enabled for this service".to_string()
                }
            );
        }

        // service enable slashing
        enable_slashing(
            deps.as_mut(),
            env.clone(),
            message_info(&service, &[]),
            SlashingParameters {
                destination: Some(operator.clone()),
                max_slashing_bips: 1000,
                resolution_window: 1000,
            },
        )
        .expect("enable slashing failed");

        // move blockchain
        env.block.time = env.block.time.plus_seconds(1);

        // operator opt-in to slashing
        let res = execute::operator_opt_in_to_slashing(
            deps.as_mut(),
            env.clone(),
            operator_info.clone(),
            service.clone(),
        );

        // assert events
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("OperatorOptedInToSlashing")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
            ))
        );

        // assert state is updated for operator opting in to service slashing
        let opted_in = SLASHING_OPT_IN
            .may_load(&deps.storage, (&service, &operator))
            .unwrap();
        assert_eq!(opted_in, Some(true));

        // assert state is not updated for operator2
        let opted_in = SLASHING_OPT_IN
            .may_load(&deps.storage, (&service, &operator2))
            .unwrap();
        assert_eq!(opted_in, None);

        // NEGATIVE test -
        // operator3 opt-in to service slashing fail as registration status isn't active
        {
            let err = execute::operator_opt_in_to_slashing(
                deps.as_mut(),
                env.clone(),
                message_info(&operator3, &[]),
                service.clone(),
            )
            .unwrap_err();
            assert_eq!(
                err,
                ContractError::InvalidRegistrationStatus {
                    msg: "Operator and service must have active registration".to_string()
                }
            );
        }

        // service re-enable slashing
        enable_slashing(
            deps.as_mut(),
            env.clone(),
            message_info(&service, &[]),
            SlashingParameters {
                destination: Some(operator.clone()),
                max_slashing_bips: 5000,
                resolution_window: 1000,
            },
        )
        .expect("enable slashing failed");

        // assert that the opt-in mapping is cleared
        let opted_in = SLASHING_OPT_IN
            .may_load(&deps.storage, (&service, &operator))
            .unwrap();
        assert_eq!(opted_in, None);

        // assert that operator2 can opt-in to slashing
        execute::operator_opt_in_to_slashing(
            deps.as_mut(),
            env.clone(),
            message_info(&operator2, &[]),
            service.clone(),
        )
        .expect("operator2 opt-in to slashing failed");
        let opted_in = SLASHING_OPT_IN
            .may_load(&deps.storage, (&service, &operator2))
            .unwrap();
        assert_eq!(opted_in, Some(true));
    }

    #[test]
    fn query_status() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone(), None),
            Ok(StatusResponse(0))
        );

        REGISTRATION_STATUS
            .save(
                &mut deps.storage,
                (&operator, &service),
                &RegistrationStatus::Inactive.into(),
                env.block.time.seconds(),
            )
            .expect("failed to save registration status");

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone(), None),
            Ok(StatusResponse(0))
        );

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone(), None),
            Ok(StatusResponse(1))
        );

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone(), None),
            Ok(StatusResponse(2))
        );

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone(), None),
            Ok(StatusResponse(3))
        );
    }

    #[test]
    fn query_status_at_timestamp() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        assert_eq!(
            status(
                deps.as_ref(),
                operator.clone(),
                service.clone(),
                Some(env.block.time.seconds())
            ),
            Ok(RegistrationStatus::Inactive.into())
        );

        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        // Assert that the status is inactive at the current timestamp
        assert_eq!(
            status(
                deps.as_ref(),
                operator.clone(),
                service.clone(),
                Some(env.block.time.seconds())
            ),
            Ok(RegistrationStatus::Inactive.into())
        );

        // Assert that the status is active at the next timestamp
        assert_eq!(
            status(
                deps.as_ref(),
                operator.clone(),
                service.clone(),
                Some(env.block.time.plus_seconds(1).seconds())
            ),
            Ok(RegistrationStatus::Active.into())
        );

        // advance block by 10 seconds
        env.block.time = env.block.time.plus_seconds(10);

        // save status at current timestamp
        state::set_registration_status(
            &mut deps.storage,
            &env,
            (&operator, &service),
            RegistrationStatus::Inactive,
        )
        .unwrap();

        // Assert that the status is active at current timestamp
        assert_eq!(
            status(
                deps.as_ref(),
                operator.clone(),
                service.clone(),
                Some(env.block.time.seconds())
            ),
            Ok(RegistrationStatus::Active.into())
        );
        // Assert that the status is inactive at timestamp + 1
        assert_eq!(
            status(
                deps.as_ref(),
                operator.clone(),
                service.clone(),
                Some(env.block.time.plus_seconds(1).seconds())
            ),
            Ok(RegistrationStatus::Inactive.into())
        );
    }

    #[test]
    fn query_is_service() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");

        // assert is_service false before service registration
        let is_service = query::is_service(deps.as_ref(), service.clone()).unwrap();
        assert_eq!(is_service, IsServiceResponse(false));

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        // assert is_service true after service registration
        let is_service = query::is_service(deps.as_ref(), service.clone()).unwrap();
        assert_eq!(is_service, IsServiceResponse(true));
    }

    #[test]
    fn query_is_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");

        // assert is_operator false before operator registration
        let is_operator = query::is_operator(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator, IsOperatorResponse(false));

        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        // assert is_operator true after operator registration
        let is_operator = query::is_operator(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator, IsOperatorResponse(true));
    }

    #[test]
    fn query_is_operator_active() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        // assert is_operator_active false before operator registration active
        let is_operator_active =
            query::is_operator_active(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator_active, IsOperatorActiveResponse(false));

        // register operator + service
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();
        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        // register operator to service => ServiceRegistered
        register_operator_to_service(
            deps.as_mut(),
            message_info(&service, &[]),
            env.clone(),
            operator.clone(),
        )
        .expect("register operator to service failed");

        // assert is_operator_active false - status is only ServiceRegistered
        let is_operator_active =
            query::is_operator_active(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator_active, IsOperatorActiveResponse(false));

        // register service to operator => Active
        register_service_to_operator(
            deps.as_mut(),
            message_info(&operator, &[]),
            env.clone(),
            service.clone(),
        )
        .expect("register service to operator failed");

        // assert is_operator_active true - status is now Active
        let is_operator_active =
            query::is_operator_active(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator_active, IsOperatorActiveResponse(true));
    }

    #[test]
    fn query_slashing_parameters() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let service = deps.api.addr_make("service");
        let destination = deps.api.addr_make("destination");
        let service_info = message_info(&service, &[]);

        // register service
        execute::register_as_service(
            deps.as_mut(),
            service_info.clone(),
            Metadata {
                uri: None,
                name: None,
            },
        )
        .expect("register service failed");

        // enable slashing
        execute::enable_slashing(
            deps.as_mut(),
            env.clone(),
            service_info.clone(),
            SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 1000,
                resolution_window: 1000,
            },
        )
        .expect("enable slashing failed");

        // query slashing parameters
        let SlashingParametersResponse(slashing_parameters) =
            query::get_slashing_parameters(deps.as_ref(), service.clone(), None).unwrap();
        assert_eq!(
            slashing_parameters,
            Some(SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 1000,
                resolution_window: 1000,
            })
        );

        // move blockchain
        env.block.time = env.block.time.plus_seconds(1);

        // update slashing parameters
        execute::enable_slashing(
            deps.as_mut(),
            env.clone(),
            service_info.clone(),
            SlashingParameters {
                destination: None,
                max_slashing_bips: 5000,
                resolution_window: 999,
            },
        )
        .expect("update slashing failed");

        // move blockchain
        env.block.time = env.block.time.plus_seconds(1);

        // query updated slashing parameters
        let SlashingParametersResponse(slashing_parameters) =
            query::get_slashing_parameters(deps.as_ref(), service.clone(), None).unwrap();
        assert_eq!(
            slashing_parameters,
            Some(SlashingParameters {
                destination: None,
                max_slashing_bips: 5000,
                resolution_window: 999,
            })
        );

        // query previous slashing parameters
        let SlashingParametersResponse(slashing_parameters) = query::get_slashing_parameters(
            deps.as_ref(),
            service.clone(),
            Some(env.block.time.minus_seconds(1).seconds()),
        )
        .unwrap();
        assert_eq!(
            slashing_parameters,
            Some(SlashingParameters {
                destination: Some(destination.clone()),
                max_slashing_bips: 1000,
                resolution_window: 1000,
            })
        );

        // move blockchain
        env.block.time = env.block.time.plus_seconds(1);

        // disable slashing
        execute::disable_slashing(deps.as_mut(), env.clone(), service_info.clone())
            .expect("disable slashing failed");

        // query slashing parameters
        let SlashingParametersResponse(slashing_parameters) =
            query::get_slashing_parameters(deps.as_ref(), service.clone(), None).unwrap();
        assert_eq!(slashing_parameters, None);

        // query previous slashing parameters
        let SlashingParametersResponse(slashing_parameters) = query::get_slashing_parameters(
            deps.as_ref(),
            service.clone(),
            Some(env.block.time.minus_seconds(1).seconds()),
        )
        .unwrap();
        assert_eq!(
            slashing_parameters,
            Some(SlashingParameters {
                destination: None,
                max_slashing_bips: 5000,
                resolution_window: 999,
            })
        );
    }

    #[test]
    fn query_is_operator_opted_in_to_slashing() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let operator = deps.api.addr_make("operator");
        let operator2 = deps.api.addr_make("operator2");
        let service = deps.api.addr_make("service");

        // assert is_operator_opted_in_to_slashing false before operator registration
        let is_operator_opted_in = query::is_operator_opted_in_to_slashing(
            deps.as_ref(),
            service.clone(),
            operator.clone(),
            None,
        )
        .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(false)
        );

        // move blockchain
        env.block.time = env.block.time.plus_seconds(1);

        // operator opt-in to slashing
        state::opt_in_to_slashing(&mut deps.storage, &env, &service, &operator)
            .expect("operator opt-in to slashing failed");

        // assert is_operator_opted_in_to_slashing true - status is now Active
        let is_operator_opted_in = query::is_operator_opted_in_to_slashing(
            deps.as_ref(),
            service.clone(),
            operator.clone(),
            None,
        )
        .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(true)
        );

        // assert that operator2 is not opted in
        let is_operator2_opted_in = query::is_operator_opted_in_to_slashing(
            deps.as_ref(),
            service.clone(),
            operator2.clone(),
            None,
        )
        .unwrap();
        assert_eq!(
            is_operator2_opted_in,
            IsOperatorOptedInToSlashingResponse(false)
        );

        // assert is_operator_opted_in_to_slashing false at timestamp - 1
        let is_operator_opted_in = query::is_operator_opted_in_to_slashing(
            deps.as_ref(),
            service.clone(),
            operator.clone(),
            Some(env.block.time.minus_seconds(1).seconds()),
        )
        .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(false)
        );
    }
}
