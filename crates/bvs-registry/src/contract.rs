#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crate:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_STAKER_OPT_OUT_WINDOW_BLOCKS: u64 = 180 * 24 * 60 * 60 / 12; // 15 days

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
        ExecuteMsg::RegisterAsOperator {
            operator_details,
            metadata,
        } => execute::register_as_operator(deps, info, operator_details, metadata),
        ExecuteMsg::UpdateOperatorDetails(new_operator_details) => {
            execute::update_operator_details(deps, info, new_operator_details)
        }
        ExecuteMsg::UpdateOperatorMetadata(metadata) => {
            execute::update_operator_metadata_uri(deps, info, metadata)
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
    use super::*;
    use crate::error::ContractError;
    use crate::msg::{Metadata, OperatorDetails};
    use crate::state;
    use crate::state::{require_operator_registered, RegistrationStatus, OPERATORS, SERVICES};
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
        state::require_service_registered(deps.storage, &info.sender)?;

        let metadata_event =
            create_metadata_event(metadata).add_attribute("service", info.sender.clone());

        Ok(Response::new().add_event(metadata_event))
    }

    fn set_operator_details(
        deps: DepsMut,
        operator: Addr,
        new_operator_details: OperatorDetails,
    ) -> Result<Event, ContractError> {
        let current = OPERATORS
            .may_load(deps.storage, &operator)?
            .unwrap_or(OperatorDetails {
                staker_opt_out_window_blocks: 0,
            });

        if new_operator_details.staker_opt_out_window_blocks > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS {
            return Err(ContractError::OperatorUpdate {
                msg: "staker_opt_out_window_blocks cannot be more than MAX_STAKER_OPT_OUT_WINDOW_BLOCKS"
                    .to_string(),
            });
        }

        if new_operator_details.staker_opt_out_window_blocks < current.staker_opt_out_window_blocks
        {
            return Err(ContractError::OperatorUpdate {
                msg: "staker_opt_out_window_blocks cannot be reduced to shorter than current value"
                    .to_string(),
            });
        }

        OPERATORS.save(deps.storage, &operator, &new_operator_details)?;

        let event = Event::new("SetOperatorDetails")
            .add_attribute("operator", operator.to_string())
            .add_attribute(
                "staker_opt_out_window_blocks",
                new_operator_details
                    .staker_opt_out_window_blocks
                    .to_string(),
            );

        Ok(event)
    }

    /// Registers the `info.sender` as an operator.
    ///
    /// `metadata_uri` is never stored and is only emitted in the `OperatorMetadataURIUpdated` event.
    pub fn register_as_operator(
        mut deps: DepsMut,
        info: MessageInfo,
        operator_details: OperatorDetails,
        metadata: Metadata,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();

        // error if the operator is already registered
        let operator_registered = OPERATORS.may_load(deps.storage, &operator)?.is_some();
        if operator_registered {
            return Err(ContractError::OperatorRegistered {});
        }

        // add operator into the state
        let set_operator_event =
            set_operator_details(deps.branch(), operator.clone(), operator_details)?;

        let mut response = Response::new();

        let register_event =
            Event::new("OperatorRegistered").add_attribute("operator", operator.to_string());
        response = response.add_event(register_event);

        response = response.add_event(set_operator_event);

        let metadata_event =
            create_metadata_event(metadata).add_attribute("operator", operator.to_string());
        response = response.add_event(metadata_event);

        Ok(response)
    }

    /// Called by an operator to set new [`OperatorDetails`].
    ///
    /// New `staker_opt_out_window_blocks` cannot be reduced and exceed [`MAX_STAKER_OPT_OUT_WINDOW_BLOCKS`].
    pub fn update_operator_details(
        deps: DepsMut,
        info: MessageInfo,
        new_operator_details: OperatorDetails,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();
        require_operator_registered(deps.storage, &operator)?;

        let set_operator_details_event =
            set_operator_details(deps, operator, new_operator_details)?;

        Ok(Response::new().add_event(set_operator_details_event))
    }

    /// Called by an operator to emit an `OperatorMetadataURIUpdated` event indicating the information has updated.
    pub fn update_operator_metadata_uri(
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
    /// Service must be registered via [`super::ExecuteMsg::ServiceRegister`].  
    /// If the operator has registered this service, the registration status will be set to [`RegistrationStatus::Active`] (1)  
    /// Else the registration status will be set to [`RegistrationStatus::ServiceRegistered`] (3)
    pub fn register_operator_to_service(
        deps: DepsMut,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let service = info.sender.clone();
        state::require_service_registered(deps.storage, &service)?;

        let key = (&operator, &service);
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
                        .add_attribute("method", "register_operator_to_service")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "ServiceRegistered"),
                ))
            }
            RegistrationStatus::OperatorRegistered => {
                state::set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
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
        state::require_service_registered(deps.storage, &service)?;

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
                    .add_attribute("method", "deregister_operator_from_service")
                    .add_attribute("operator", operator)
                    .add_attribute("service", service)
                    .add_attribute("status", "Inactive"),
            ))
        }
    }

    /// Register a service to an operator (info.sender is the operator)
    /// Operator must be registered in the delegation manager
    /// If the service has registered this operator, the registration status will be set to [`RegistrationStatus::Active`] (1)
    /// Else the registration status will be set to [`RegistrationStatus::OperatorRegistered`] (2)
    pub fn register_service_to_operator(
        deps: DepsMut,
        info: MessageInfo,
        service: Addr,
    ) -> Result<Response, ContractError> {
        let operator = info.sender.clone();
        state::require_service_registered(deps.storage, &service)?;
        state::require_operator_registered(deps.storage, &operator)?;

        let key = (&operator, &service);
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
                        .add_attribute("method", "register_service_to_operator")
                        .add_attribute("operator", operator)
                        .add_attribute("service", service)
                        .add_attribute("status", "OperatorRegistered"),
                ))
            }
            RegistrationStatus::ServiceRegistered => {
                state::set_registration_status(deps.storage, key, RegistrationStatus::Active)?;
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
        state::require_service_registered(deps.storage, &service)?;

        let key = (&operator, &service);
        let status = state::get_registration_status(deps.storage, key)?;

        if status == RegistrationStatus::Inactive {
            Err(ContractError::InvalidRegistrationStatus {
                msg: "Already deregistered.".to_string(),
            })
        } else {
            state::set_registration_status(deps.storage, key, RegistrationStatus::Inactive)?;
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
            let service_addr = deps.api.addr_validate(&service)?;
            to_json_binary(&query::is_service(deps, service_addr)?)
        }
        QueryMsg::IsOperator(operator) => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_operator(deps, operator_addr)?)
        }
        QueryMsg::OperatorDetails(operator) => {
            let operator_addr = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::operator_details(deps, operator_addr)?)
        }
    }
}

mod query {
    use crate::msg::{
        IsOperatorResponse, IsServiceResponse, OperatorDetailsResponse, StatusResponse,
    };
    use crate::state;
    use crate::state::{require_operator_registered, require_service_registered, OPERATORS};
    use cosmwasm_std::{Addr, Deps, StdError, StdResult};

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
        let is_service_registered =
            require_service_registered(deps.storage, &service).map_or(false, |_| true);

        Ok(IsServiceResponse(is_service_registered))
    }

    /// Query if the operator is registered or not.
    pub fn is_operator(deps: Deps, operator: Addr) -> StdResult<IsOperatorResponse> {
        let is_operator_registered = require_operator_registered(deps.storage, &operator).is_ok();

        Ok(IsOperatorResponse(is_operator_registered))
    }

    /// Query the operator details.
    pub fn operator_details(deps: Deps, operator: Addr) -> StdResult<OperatorDetailsResponse> {
        require_operator_registered(deps.storage, &operator)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        let details = OPERATORS.load(deps.storage, &operator)?;
        Ok(OperatorDetailsResponse { details })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query::status;
    use crate::msg::{
        InstantiateMsg, IsOperatorResponse, IsServiceResponse, Metadata, OperatorDetails,
        StatusResponse,
    };
    use crate::state;
    use crate::state::{RegistrationStatus, OPERATORS, SERVICES};
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
            Metadata {
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
    fn test_register_as_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        let res = execute::register_as_operator(
            deps.as_mut(),
            operator_info,
            OperatorDetails {
                staker_opt_out_window_blocks: 100,
            },
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
                    Event::new("SetOperatorDetails")
                        .add_attribute("operator", operator.clone())
                        .add_attribute("staker_opt_out_window_blocks", 100.to_string())
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
                OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
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
            OperatorDetails {
                staker_opt_out_window_blocks: 100,
            },
            Metadata {
                uri: Some("uri".to_string()),
                name: Some("operator1".to_string()),
            },
        );
        assert_eq!(err, Err(ContractError::OperatorRegistered {}),);
    }

    #[test]
    fn test_update_operator_details() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        {
            // register operator the first time
            execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            )
            .expect("register operator failed");

            let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
            assert_eq!(operator.staker_opt_out_window_blocks, 100);
        }

        // update operator details
        let res = execute::update_operator_details(
            deps.as_mut(),
            operator_info,
            OperatorDetails {
                staker_opt_out_window_blocks: 101,
            },
        );

        // assert event
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("SetOperatorDetails")
                    .add_attribute("operator", operator.as_ref())
                    .add_attribute("staker_opt_out_window_blocks", 101.to_string())
            ))
        );

        // assert operator details is updated
        let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
        assert_eq!(operator.staker_opt_out_window_blocks, 101);
    }

    #[test]
    fn test_update_operator_details_reduced_error() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        {
            // register operator the first time
            execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            )
            .expect("register operator failed");

            let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
            assert_eq!(operator.staker_opt_out_window_blocks, 100);
        }

        // update operator details
        let err = execute::update_operator_details(
            deps.as_mut(),
            operator_info,
            OperatorDetails {
                staker_opt_out_window_blocks: 100 - 1, // reduce 1
            },
        );

        // assert event
        assert_eq!(
            err,
            Err(ContractError::OperatorUpdate {
                msg: "staker_opt_out_window_blocks cannot be reduced to shorter than current value"
                    .to_string()
            })
        );

        // assert operator details are not updated
        let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
        assert_eq!(operator.staker_opt_out_window_blocks, 100);
    }

    #[test]
    fn test_register_and_update_operator_details_more_than_max() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator_info = message_info(&operator, &[]);

        {
            // register operator fails due to max + 1
            let err = execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                OperatorDetails {
                    staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
                },
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            );

            assert_eq!(
                err,
                Err(ContractError::OperatorUpdate {
                    msg: "staker_opt_out_window_blocks cannot be more than MAX_STAKER_OPT_OUT_WINDOW_BLOCKS"
                        .to_string()
                })
            );
        }
        {
            // register operator success
            execute::register_as_operator(
                deps.as_mut(),
                operator_info.clone(),
                OperatorDetails {
                    staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS,
                },
                Metadata {
                    uri: Some("uri".to_string()),
                    name: Some("operator1".to_string()),
                },
            )
            .expect("register operator failed");

            let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
            assert_eq!(
                operator.staker_opt_out_window_blocks,
                MAX_STAKER_OPT_OUT_WINDOW_BLOCKS
            );
        }

        // update operator details failed due to max + 1
        let err = execute::update_operator_details(
            deps.as_mut(),
            operator_info,
            OperatorDetails {
                staker_opt_out_window_blocks: MAX_STAKER_OPT_OUT_WINDOW_BLOCKS + 1,
            },
        );

        // assert error
        assert_eq!(
            err,
            Err(ContractError::OperatorUpdate {
                msg: "staker_opt_out_window_blocks cannot be more than MAX_STAKER_OPT_OUT_WINDOW_BLOCKS"
                    .to_string()
            })
        );

        // assert operator details are not updated
        let operator = OPERATORS.load(&deps.storage, &operator).unwrap();
        assert_eq!(
            operator.staker_opt_out_window_blocks,
            MAX_STAKER_OPT_OUT_WINDOW_BLOCKS
        );
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
                OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
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
                        Event::new("SetOperatorDetails")
                            .add_attribute("operator", operator.as_ref())
                            .add_attribute("staker_opt_out_window_blocks", 100.to_string())
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
        let res = execute::update_operator_metadata_uri(
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
            OperatorDetails {
                staker_opt_out_window_blocks: 100,
            },
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
            OperatorDetails {
                staker_opt_out_window_blocks: 100,
            },
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
        OPERATORS
            .save(
                &mut deps.storage,
                &operator,
                &OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
            )
            .unwrap();

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
        OPERATORS
            .save(
                &mut deps.storage,
                &operator,
                &OperatorDetails {
                    staker_opt_out_window_blocks: 100,
                },
            )
            .unwrap();

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

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATORS
            .save(&mut deps.storage, &operator, &operator_details)
            .unwrap();

        // assert is_operator true after operator registration
        let is_operator = query::is_operator(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(is_operator, IsOperatorResponse(true));
    }

    #[test]
    fn query_operator_details() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");

        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        OPERATORS
            .save(&mut deps.storage, &operator, &operator_details)
            .unwrap();

        // assert operator details
        let query_res = query::operator_details(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(query_res.details, operator_details);

        // update OperatorDetails
        let new_operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 200,
        };
        OPERATORS
            .save(&mut deps.storage, &operator, &new_operator_details)
            .unwrap();

        // assert new operator details
        let query_res = query::operator_details(deps.as_ref(), operator.clone()).unwrap();
        assert_eq!(query_res.details, new_operator_details);
    }

    #[test]
    fn query_operator_details_before_registration() {
        let deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");

        // assert operator details
        let query_err = query::operator_details(deps.as_ref(), operator.clone()).unwrap_err();
        assert_eq!(
            query_err,
            StdError::generic_err("Operator is not registered")
        );
    }
}
