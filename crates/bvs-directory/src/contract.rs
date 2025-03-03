#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    auth,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use bvs_library::ownership;

const CONTRACT_NAME: &str = "BVS Directory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    bvs_registry::api::set_registry_addr(deps.storage, &deps.api.addr_validate(&msg.registry)?)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::_set_owner(deps.storage, &owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("registry", msg.registry))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_registry::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::ServiceRegister { metadata } => execute::service_register(deps, info, metadata),
        ExecuteMsg::ServiceUpdateMetadata(metadata) => {
            execute::service_update_metadata(deps, info, metadata)
        }
        ExecuteMsg::ServiceRegisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::service_register_operator(deps, info, operator)
        }
        ExecuteMsg::ServiceDeregisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::service_deregister_operator(deps, info, operator)
        }
        ExecuteMsg::OperatorRegisterService { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::operator_register_service(deps, info, service)
        }
        ExecuteMsg::OperatorDeregisterService { service } => {
            let service = deps.api.addr_validate(&service)?;
            execute::operator_deregister_service(deps, info, service)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps, &info, &new_owner).map_err(ContractError::Ownership)
        }
        ExecuteMsg::SetRouting { delegation_manager } => {
            let delegation_manager = deps.api.addr_validate(&delegation_manager)?;
            auth::set_routing(deps, info, delegation_manager)
        }
    }
}

mod execute {
    use crate::msg::ServiceMetadata;
    use crate::state::{RegistrationStatus, SERVICES};
    use crate::{auth, state, ContractError};
    use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

    /// Register a service (info.server = service)
    pub fn service_register(
        deps: DepsMut,
        info: MessageInfo,
        metadata: ServiceMetadata,
    ) -> Result<Response, ContractError> {
        let registered = SERVICES
            .may_load(deps.storage, &info.sender)?
            .unwrap_or(false);

        if registered {
            return Err(ContractError::ServiceRegistered {});
        }

        SERVICES.save(deps.storage, &info.sender, &true)?;

        Ok(Response::new()
            .add_event(
                Event::new("ServiceRegistered").add_attribute("service", info.sender.to_string()),
            )
            .add_event(new_event_metadata(metadata, &info.sender)))
    }

    /// Update service metadata (info.server = service)
    pub fn service_update_metadata(
        deps: DepsMut,
        info: MessageInfo,
        metadata: ServiceMetadata,
    ) -> Result<Response, ContractError> {
        state::require_service_registered(deps.storage, &info.sender)?;

        Ok(Response::new().add_event(new_event_metadata(metadata, &info.sender)))
    }

    /// Event for ServiceMetadataUpdated
    /// Service hash `SHA256(service)` will be calculated offchain
    fn new_event_metadata(metadata: ServiceMetadata, service: &Addr) -> Event {
        let mut event = Event::new("ServiceMetadataUpdated").add_attribute("service", service);

        if let Some(uri) = metadata.uri {
            event = event.add_attribute("metadata.uri", uri);
        }

        if let Some(name) = metadata.name {
            event = event.add_attribute("metadata.name", name);
        }

        event
    }

    /// Register an operator to a service (info.server = service)
    /// Service must be registered via ExecuteMsg::ServiceRegister
    /// If the operator is already registered, the registration status will be set to ACTIVE (1)
    /// Else the registration status will be set to SERVICE_REGISTERED (3)
    pub fn service_register_operator(
        deps: DepsMut,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        state::require_service_registered(deps.storage, &info.sender)?;

        let key = (&operator, &info.sender);
        let status = state::get_registration_status(deps.storage, key)?;
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
                state::set_registration_status(
                    deps.storage,
                    key,
                    RegistrationStatus::ServiceRegistered,
                )?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "service_register_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", info.sender)
                        .add_attribute("status", "ServiceRegistered"),
                ))
            }
            RegistrationStatus::OperatorRegistered => {
                state::set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "service_register_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", info.sender)
                        .add_attribute("status", "Active"),
                ))
            }
        }
    }

    /// Deregister an operator to a service (info.server = service)
    /// Set the registration status to INACTIVE (0)
    pub fn service_deregister_operator(
        deps: DepsMut,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        state::require_service_registered(deps.storage, &info.sender)?;

        let key = (&operator, &info.sender);
        let status = state::get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            })
        } else {
            state::set_registration_status(deps.storage, key, RegistrationStatus::Inactive)?;
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "service_deregister_operator")
                    .add_attribute("operator", operator)
                    .add_attribute("service", info.sender)
                    .add_attribute("status", "Inactive"),
            ))
        }
    }

    /// Register a service to an operator (info.server = operator)
    /// Operator must be registered on the delegation manager
    /// If the service is already registered, the registration status will be set to ACTIVE (1)
    /// Else the registration status will be set to OPERATOR_REGISTERED (2)
    pub fn operator_register_service(
        deps: DepsMut,
        info: MessageInfo,
        service: Addr,
    ) -> Result<Response, ContractError> {
        state::require_service_registered(deps.storage, &service)?;

        let delegation_manager = auth::get_delegation_manager(deps.storage)?;
        let is_operator_response: bvs_delegation_manager::query::OperatorResponse =
            deps.querier.query_wasm_smart(
                delegation_manager.clone(),
                &bvs_delegation_manager::msg::QueryMsg::IsOperator {
                    operator: info.sender.to_string(),
                },
            )?;

        if !is_operator_response.is_operator {
            return Err(ContractError::OperatorNotFound {
                msg: "Operator is not registered on delegation manager.".to_string(),
            });
        }

        let key = (&info.sender, &service);
        let status = state::get_registration_status(deps.storage, key)?;
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
                state::set_registration_status(
                    deps.storage,
                    key,
                    RegistrationStatus::OperatorRegistered,
                )?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "operator_register_service")
                        .add_attribute("operator", info.sender)
                        .add_attribute("service", service)
                        .add_attribute("status", "OperatorRegistered"),
                ))
            }
            RegistrationStatus::ServiceRegistered => {
                state::set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
                Ok(Response::new().add_event(
                    Event::new("RegistrationStatusUpdated")
                        .add_attribute("method", "operator_register_service")
                        .add_attribute("operator", info.sender)
                        .add_attribute("service", service)
                        .add_attribute("status", "Active"),
                ))
            }
        }
    }

    /// Deregister a service to an operator (info.server = operator)
    /// Set the registration status to INACTIVE (0)
    pub fn operator_deregister_service(
        deps: DepsMut,
        info: MessageInfo,
        service: Addr,
    ) -> Result<Response, ContractError> {
        state::require_service_registered(deps.storage, &service)?;

        let key = (&info.sender, &service);
        let status = state::get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            })
        } else {
            state::set_registration_status(deps.storage, key, RegistrationStatus::Inactive)?;
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "operator_deregister_service")
                    .add_attribute("operator", info.sender)
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
    }
}

mod query {
    use crate::msg::StatusResponse;
    use crate::state;
    use cosmwasm_std::{Addr, Deps, StdResult};

    /// Get the registration status of an operator to a service
    /// Returns:
    /// - Inactive (0) if not registered
    /// - Active (1) if registration is active (operator and service are registered to each other)
    /// - OperatorRegistered (2) if operator is registered to service, pending service registration
    /// - ServiceRegistered (3) if service is registered to operator, pending operator registration
    pub fn status(deps: Deps, operator: Addr, service: Addr) -> StdResult<StatusResponse> {
        let key = (&operator, &service);
        let status = state::get_registration_status(deps.storage, key)?;
        Ok(status.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query::status;
    use crate::msg::{InstantiateMsg, ServiceMetadata, StatusResponse};
    use crate::state;
    use crate::state::{RegistrationStatus, SERVICES};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        ContractResult, Event, OwnedDeps, Response, SystemError, SystemResult, WasmQuery,
    };

    fn mock_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let owner_info = message_info(&owner, &[]);

        instantiate(
            deps.as_mut(),
            env,
            owner_info.clone(),
            InstantiateMsg {
                owner: owner.to_string(),
                registry: registry.to_string(),
            },
        )
        .unwrap();

        let delegation_manager = deps.api.addr_make("delegation_manager");
        auth::set_routing(
            deps.as_mut(),
            owner_info.clone(),
            delegation_manager.clone(),
        )
        .unwrap();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { .. } => SystemResult::Ok(ContractResult::Ok(
                to_json_binary(&bvs_delegation_manager::query::OperatorResponse {
                    is_operator: true,
                })
                .unwrap(),
            )),
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        deps
    }

    #[test]
    fn test_service_registered() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        let res = execute::service_register(
            deps.as_mut(),
            service_info,
            ServiceMetadata {
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
                    Event::new("ServiceMetadataUpdated")
                        .add_attribute("service", service.as_ref())
                        .add_attribute("metadata.uri", "uri")
                        .add_attribute("metadata.name", "name")
                ))
        );

        let registered = SERVICES.load(&deps.storage, &service).unwrap();
        assert!(registered);
    }

    #[test]
    fn test_service_registered_optional_metadata() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");
        let service_info = message_info(&service, &[]);

        let res = execute::service_register(
            deps.as_mut(),
            service_info,
            ServiceMetadata {
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
                    Event::new("ServiceMetadataUpdated")
                        .add_attribute("service", service.as_ref())
                        .add_attribute("metadata.name", "Meta Bridging")
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
            ServiceMetadata {
                uri: Some("new_uri".to_string()),
                name: Some("new_name".to_string()),
            },
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("ServiceMetadataUpdated")
                    .add_attribute("service", service.as_ref())
                    .add_attribute("metadata.uri", "new_uri")
                    .add_attribute("metadata.name", "new_name")
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

        execute::service_register(
            deps.as_mut(),
            service_info.clone(),
            ServiceMetadata {
                uri: None,
                name: None,
            },
        )
        .unwrap();

        let res = execute::operator_register_service(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "operator_register_service")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "OperatorRegistered")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::OperatorRegistered);

        let res = execute::service_register_operator(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "service_register_operator")
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

        execute::service_register(
            deps.as_mut(),
            service_info.clone(),
            ServiceMetadata {
                uri: None,
                name: None,
            },
        )
        .unwrap();

        let res = execute::service_register_operator(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "service_register_operator")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("service", service.as_ref())
                    .add_attribute("status", "ServiceRegistered")
            )),
        );

        let status = state::get_registration_status(&deps.storage, (&operator, &service)).unwrap();
        assert_eq!(status, RegistrationStatus::ServiceRegistered);

        let res = execute::operator_register_service(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "operator_register_service")
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

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();

        let res = execute::operator_register_service(
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
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();

        let res = execute::service_register_operator(
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

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();
        state::set_registration_status(
            &mut deps.storage,
            (&operator, &service),
            RegistrationStatus::Active,
        )
        .unwrap();

        let res = execute::operator_register_service(
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

        let res = execute::service_register_operator(
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

        let res = execute::service_deregister_operator(
            deps.as_mut(),
            service_info.clone(),
            operator.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "service_deregister_operator")
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

        let res = execute::operator_deregister_service(
            deps.as_mut(),
            operator_info.clone(),
            service.clone(),
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("RegistrationStatusUpdated")
                    .add_attribute("method", "operator_deregister_service")
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

        let res = execute::operator_deregister_service(
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

        let res = execute::service_deregister_operator(
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
}
