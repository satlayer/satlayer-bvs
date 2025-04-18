#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
            execute::register_operator_to_service(deps, info, operator)
        }
        ExecuteMsg::DeregisterOperatorFromService { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::deregister_operator_from_service(deps, info, operator)
        }
        ExecuteMsg::RegisterServiceToOperator { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::register_service_to_operator(deps, info, service)
        }
        ExecuteMsg::DeregisterServiceFromOperator { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::deregister_service_from_operator(deps, info, service)
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
        get_registration_status, require_operator_registered, require_service_registered,
        set_registration_status, RegistrationStatus, OPERATORS, SERVICES,
    };
    use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

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
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let service = info.sender.clone();
        require_service_registered(deps.storage, &service)?;
        require_operator_registered(deps.storage, &operator)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;
        match status {
            RegistrationStatus::Active => Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration is already active.".to_string(),
            }),
            RegistrationStatus::ServiceRegistered => {
                Err(ContractError::InvalidRegistrationStatus {
                    msg: "Service has already registered.".to_string(),
                })
            }
            RegistrationStatus::Inactive => {
                set_registration_status(deps.storage, key, RegistrationStatus::ServiceRegistered)?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_operator_to_service")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "ServiceRegistered"),
                ))
            }
            RegistrationStatus::OperatorRegistered => {
                set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
                // increase operator status count
                state::increase_operator_active_registration_count(deps.storage, &operator)?;

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
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let service = info.sender.clone();
        require_service_registered(deps.storage, &service)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            })
        } else {
            set_registration_status(deps.storage, key, RegistrationStatus::Inactive)?;
            // decrease operator status count
            state::decrease_operator_active_registration_count(deps.storage, &operator)?;

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
        service: Addr,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();
        require_service_registered(deps.storage, &service)?;
        require_operator_registered(deps.storage, &operator)?;

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;
        match status {
            RegistrationStatus::Active => Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration is already active.".to_string(),
            }),
            RegistrationStatus::OperatorRegistered => {
                Err(ContractError::InvalidRegistrationStatus {
                    msg: "Operator has already registered.".to_string(),
                })
            }
            RegistrationStatus::Inactive => {
                set_registration_status(deps.storage, key, RegistrationStatus::OperatorRegistered)?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "register_service_to_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "OperatorRegistered"),
                ))
            }
            RegistrationStatus::ServiceRegistered => {
                set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
                // increase operator status count
                state::increase_operator_active_registration_count(deps.storage, &operator)?;

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
        service: Addr,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();

        let key = (&operator, &service);
        let status = get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            })
        } else {
            set_registration_status(deps.storage, key, RegistrationStatus::Inactive)?;
            // decrease operator status count
            state::decrease_operator_active_registration_count(deps.storage, &operator)?;

            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "deregister_service_from_operator")
                    .add_attribute("operator", operator)
                    .add_attribute("service", service)
                    .add_attribute("status", "Inactive"),
            ))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status { service, operator } => {
            let service = deps.api.addr_validate(&service)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::status(deps, operator, service)?)
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
    }
}

mod query {
    use crate::msg::{
        IsOperatorActiveResponse, IsOperatorResponse, IsServiceResponse, StatusResponse,
    };
    use crate::state;
    use crate::state::{require_operator_registered, require_service_registered};
    use cosmwasm_std::{Addr, Deps, StdResult};

    /// Get the registration status of an operator to a service
    /// Returns: [`StdResult<StatusResponse>`]
    /// - [`RegistrationStatus::Inactive`] (0) if not registered
    /// - [`RegistrationStatus::Active`] (1) if registration is active (operator and service are registered to each other)
    /// - [`RegistrationStatus::OperatorRegistered`] (2) if operator is registered to service, pending service registration
    /// - [`RegistrationStatus::ServiceRegistered`] (3) if service is registered to operator, pending operator registration
    pub fn status(deps: Deps, operator: Addr, service: Addr) -> StdResult<StatusResponse> {
        let key = (&operator, &service);
        let status = state::get_registration_status(deps.storage, key)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute::{register_operator_to_service, register_service_to_operator};
    use crate::contract::query::status;
    use crate::msg::{
        InstantiateMsg, IsOperatorActiveResponse, IsOperatorResponse, IsServiceResponse, Metadata,
        StatusResponse,
    };
    use crate::state;
    use crate::state::{RegistrationStatus, OPERATORS, SERVICES};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Event, OwnedDeps, Response};

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

        let operator = deps.api.addr_make("operator/2");
        let service = deps.api.addr_make("service/2");
        let operator_info = message_info(&operator, &[]);

        // register service + operator
        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Operator has already registered.".to_string(),
            }),
        );
    }

    #[test]
    fn test_register_service_already_registered() {
        let mut deps = mock_contract();

        let operator = deps.api.addr_make("operator/3");
        let service = deps.api.addr_make("service/3");
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Service has already registered.".to_string(),
            }),
        );
    }

    #[test]
    fn test_register_already_active() {
        let mut deps = mock_contract();

        let operator = deps.api.addr_make("operator/4");
        let service = deps.api.addr_make("service/4");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        // register service + operator
        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        let res = execute::register_service_to_operator(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration is already active.".to_string(),
            }),
        );

        let res = execute::register_operator_to_service(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Registration is already active.".to_string(),
            }),
        );
    }

    #[test]
    fn test_service_deregister_operator() {
        let mut deps = mock_contract();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();
        state::increase_operator_active_registration_count(&mut deps.storage, &operator)
            .expect("failed to increase operator active registration count");

        let res = execute::deregister_operator_from_service(
            deps.as_mut(),
            service_info.clone(),
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

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();
        state::increase_operator_active_registration_count(&mut deps.storage, &operator)
            .expect("failed to increase operator active registration count");

        let res = execute::deregister_service_from_operator(
            deps.as_mut(),
            operator_info.clone(),
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

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let operator_info = message_info(&operator, &[]);
        let service_info = message_info(&service, &[]);

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Inactive,
        )
        .unwrap();

        let res = execute::deregister_service_from_operator(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            }),
        );

        let res = execute::deregister_operator_from_service(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            }),
        );
    }

    #[test]
    fn query_status() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone()),
            Ok(StatusResponse(0))
        );

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Inactive,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone()),
            Ok(StatusResponse(0))
        );

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone()),
            Ok(StatusResponse(1))
        );

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone()),
            Ok(StatusResponse(2))
        );

        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();

        assert_eq!(
            status(deps.as_ref(), operator.clone(), service.clone()),
            Ok(StatusResponse(3))
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
        register_operator_to_service(deps.as_mut(), message_info(&service, &[]), operator.clone())
            .expect("register operator to service failed");

        // assert is_operator_active false - status is only ServiceRegistered
        let is_operator_active =
            query::is_operator_active(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator_active, IsOperatorActiveResponse(false));

        // register service to operator => Active
        register_service_to_operator(deps.as_mut(), message_info(&operator, &[]), service.clone())
            .expect("register service to operator failed");

        // assert is_operator_active true - status is now Active
        let is_operator_active =
            query::is_operator_active(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator_active, IsOperatorActiveResponse(true));
    }
}
