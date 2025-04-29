use crate::error::ContractError;
use bvs_library::addr::{Operator, Service};
use bvs_library::storage::EVERY_SECOND;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Env, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Map, SnapshotMap};

/// Mapping of service address to boolean value
/// indicating if the service is registered with the registry
pub const SERVICES: Map<&Service, bool> = Map::new("services");

/// Require that the service is registered in the state
pub fn require_service_registered(
    store: &dyn Storage,
    service: &Addr,
) -> Result<(), ContractError> {
    let registered = SERVICES.may_load(store, service)?.unwrap_or(false);

    if !registered {
        return Err(ContractError::Std(StdError::not_found("service")));
    }

    Ok(())
}

/// Mapping of operator address to boolean value
/// indicating if the operator is registered with the registry
pub const OPERATORS: Map<&Operator, bool> = Map::new("operators");

pub fn require_operator_registered(
    store: &dyn Storage,
    operator: &Addr,
) -> Result<(), ContractError> {
    let registered = OPERATORS.may_load(store, operator)?.unwrap_or(false);

    if !registered {
        return Err(ContractError::Std(StdError::not_found("operator")));
    }

    Ok(())
}

/// Registered status of the Operator to Service
/// Can be initiated by the Operator or the Service
/// Becomes Active when the Operator and Service both have registered
/// Becomes Inactive when the Operator or Service have unregistered (default state)
#[cw_serde]
pub enum RegistrationStatus {
    /// Default state when neither the Operator nor the Service has registered,
    /// or when either the Operator or Service has unregistered
    Inactive = 0,

    /// State when both the Operator and Service have registered with each other,
    /// indicating a fully established relationship
    Active = 1,

    /// State when only the Operator has registered but the Service hasn't yet registered,
    /// indicating a pending registration from the Service side
    /// This is Operator-initiated registration, waiting for Service to finalize
    OperatorRegistered = 2,

    /// State when only the Service has registered but the Operator hasn't yet registered,
    /// indicating a pending registration from the Operator side
    /// This is Service-initiated registration, waiting for Operator to finalize
    ServiceRegistered = 3,
}

impl From<RegistrationStatus> for u8 {
    fn from(value: RegistrationStatus) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for RegistrationStatus {
    type Error = StdError;

    fn try_from(value: u8) -> Result<Self, StdError> {
        match value {
            0 => Ok(RegistrationStatus::Inactive),
            1 => Ok(RegistrationStatus::Active),
            2 => Ok(RegistrationStatus::OperatorRegistered),
            3 => Ok(RegistrationStatus::ServiceRegistered),
            _ => Err(StdError::generic_err("RegistrationStatus out of range")),
        }
    }
}

/// Mapping of (operator_service) address.
/// See `RegistrationStatus` for more of the status.
/// Use [get_registration_status] and [set_registration_status] to interact with this map.
pub(crate) const REGISTRATION_STATUS: SnapshotMap<(&Operator, &Service), u8> = SnapshotMap::new(
    "registration_status",
    "registration_status_checkpoint",
    "registration_status_changelog",
    EVERY_SECOND,
);

/// Get the registration status of the Operator to Service
pub fn get_registration_status(
    store: &dyn Storage,
    key: (&Operator, &Service),
) -> StdResult<RegistrationStatus> {
    let status = REGISTRATION_STATUS
        .may_load(store, key)?
        .unwrap_or(RegistrationStatus::Inactive.into());

    status.try_into()
}

/// Get the registration status of the Operator to Service at a specific timestamp
///
/// #### Warning
/// This function will return previous state.
/// If timestamp is equal to the timestamp of the save operation.
/// New state will only be available at timestamp + 1
pub fn get_registration_status_at_timestamp(
    store: &dyn Storage,
    key: (&Operator, &Service),
    timestamp: u64,
) -> StdResult<RegistrationStatus> {
    let status = REGISTRATION_STATUS
        .may_load_at_height(store, key, timestamp)?
        .unwrap_or(RegistrationStatus::Inactive.into());

    status.try_into()
}

/// Set the registration status of the Operator to Service at current timestamp
///
/// #### Warning
/// This function will only save the state at the end of the block.
/// So the new state will only be available at timestamp + 1.
/// This is so that re-ordering of txs won't cause the state to be inconsistent.
pub fn set_registration_status(
    store: &mut dyn Storage,
    env: &Env,
    key: (&Operator, &Service),
    status: RegistrationStatus,
) -> StdResult<()> {
    let (operator, service) = key;
    match status {
        RegistrationStatus::Active => {
            increase_operator_active_registration_count(store, operator)?;
            // if service has enabled slashing, opt-in operator to slashing
            if is_slashing_enabled(store, service, Some(env.block.time.seconds()))? {
                opt_in_to_slashing(store, env, service, operator)?;
            }
        }
        RegistrationStatus::Inactive => {
            decrease_operator_active_registration_count(store, operator)?;
        }
        _ => {}
    }

    REGISTRATION_STATUS.save(store, key, &status.into(), env.block.time.seconds())?;
    Ok(())
}

pub fn require_active_registration_status(
    store: &dyn Storage,
    key: (&Operator, &Service),
) -> Result<(), ContractError> {
    match get_registration_status(store, key)? {
        RegistrationStatus::Active => Ok(()),
        _ => Err(ContractError::InvalidRegistrationStatus {
            msg: "Operator and service must have active registration.".to_string(),
        }),
    }
}

/// Stores the active registration count of the operator to services.
/// This is used to check if the operator is actively registered to any service (> 0)
pub const OPERATOR_ACTIVE_REGISTRATION_COUNT: Map<&Operator, u64> =
    Map::new("operator_active_registration_count");

/// Check if the operator is actively registered to any service
pub fn is_operator_active(store: &dyn Storage, operator: &Operator) -> StdResult<bool> {
    let active_count = OPERATOR_ACTIVE_REGISTRATION_COUNT
        .may_load(store, operator)?
        .unwrap_or(0);

    Ok(active_count > 0)
}

/// Increase the operator active registration count by 1
pub fn increase_operator_active_registration_count(
    store: &mut dyn Storage,
    operator: &Operator,
) -> StdResult<u64> {
    OPERATOR_ACTIVE_REGISTRATION_COUNT.update(store, operator, |count| {
        let new_count = count.unwrap_or(0).checked_add(1);
        new_count.ok_or_else(|| {
            StdError::generic_err("Increase operator active registration count failed")
        })
    })
}

/// Decrease the operator active registration count by 1
pub fn decrease_operator_active_registration_count(
    store: &mut dyn Storage,
    operator: &Operator,
) -> StdResult<u64> {
    OPERATOR_ACTIVE_REGISTRATION_COUNT.update(store, operator, |count| {
        let new_count = count.unwrap_or(0).checked_sub(1);
        new_count.ok_or_else(|| {
            StdError::generic_err("Decrease operator active registration count failed")
        })
    })
}

#[cw_serde]
pub struct SlashingParameters {
    /// The address to which the slashed funds will be sent after the slashing is finalized.  
    /// None, indicates that the slashed funds will be burned.
    pub destination: Option<Addr>,
    /// The maximum percentage of the operator's total stake that can be slashed.  
    /// The value is represented in bips (basis points), where 100 bips = 1%.  
    /// And the value must be between 0 and 10_000 (inclusive).
    pub max_slashing_bips: u16,
    /// The minimum amount of time (in seconds)
    /// that the slashing can be delayed before it is executed and finalized.  
    /// Setting this value to a duration less than the queued withdrawal delay is recommended.
    /// To prevent restaker's early withdrawal of their assets from the vault due to the impending slash,
    /// defeating the purpose of shared security.
    pub resolution_window: u64,
}

impl SlashingParameters {
    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        if let Some(destination) = &self.destination {
            api.addr_validate(destination.as_str()).map_err(|_| {
                ContractError::InvalidSlashingParameters {
                    msg: "destination address is invalid".to_string(),
                }
            })?;
        }
        if self.max_slashing_bips > 10_000 {
            return Err(ContractError::InvalidSlashingParameters {
                msg: "max_slashing_percentage is over 10_000 bips (100%)".to_string(),
            });
        }
        Ok(())
    }
}

/// Mapping of service to the latest slashing parameters.
///
/// The presence of the Service key in the map indicates that slashing is enabled for that service.
pub(crate) const SLASHING_PARAMETERS: SnapshotMap<&Service, SlashingParameters> = SnapshotMap::new(
    "slashing_parameters",
    "slashing_parameters_checkpoint",
    "slashing_parameters_changelog",
    EVERY_SECOND,
);

/// Returns whether slashing is enabled for the given service at the given timestamp.
pub fn is_slashing_enabled(
    store: &dyn Storage,
    service: &Service,
    timestamp: Option<u64>,
) -> StdResult<bool> {
    let is_enabled = match timestamp {
        Some(t) => SLASHING_PARAMETERS
            .may_load_at_height(store, service, t)?
            .is_some(),
        None => SLASHING_PARAMETERS.may_load(store, service)?.is_some(),
    };
    Ok(is_enabled)
}

/// Enable slashing for the given service at current timestamp
pub fn enable_slashing(
    store: &mut dyn Storage,
    api: &dyn Api,
    env: &Env,
    service: &Service,
    slashing_parameters: &SlashingParameters,
) -> Result<(), ContractError> {
    // Validate the slashing parameters
    slashing_parameters.validate(api)?;

    // Save the slashing parameters to the store
    SLASHING_PARAMETERS.save(
        store,
        service,
        slashing_parameters,
        env.block.time.seconds(),
    )?;
    Ok(())
}

/// Disable slashing for the given service at current timestamp
pub fn disable_slashing(store: &mut dyn Storage, env: &Env, service: &Service) -> StdResult<()> {
    SLASHING_PARAMETERS.remove(store, service, env.block.time.seconds())?;
    Ok(())
}

/// Stores the slashing parameters opt-in status for (service, operator) pair.
///
/// If value is `true`,
/// operator has opted in to slashing parameters for that service at the given timestamp.
/// If key isn't found, it means the operator hasn't opted in to slashing parameters for that service.
/// The `false` value is not used.
pub(crate) const SLASHING_OPT_IN: SnapshotMap<(&Service, &Operator), bool> = SnapshotMap::new(
    "slashing_opt_in",
    "slashing_opt_in_checkpoint",
    "slashing_opt_in_changelog",
    EVERY_SECOND,
);

/// Opt-in operator to the current service slashing parameters at current timestamp
pub fn opt_in_to_slashing(
    store: &mut dyn Storage,
    env: &Env,
    service: &Service,
    operator: &Operator,
) -> StdResult<()> {
    SLASHING_OPT_IN.save(store, (service, operator), &true, env.block.time.seconds())?;
    Ok(())
}

/// Check if the operator has opted in to slashing for the given service at the given timestamp.
pub fn is_operator_opted_in_to_slashing(
    store: &dyn Storage,
    service: &Service,
    operator: &Operator,
    timestamp: Option<u64>,
) -> StdResult<bool> {
    let is_opted_in = match timestamp {
        Some(t) => SLASHING_OPT_IN
            .may_load_at_height(store, (service, operator), t)?
            .is_some(),
        None => SLASHING_OPT_IN
            .may_load(store, (service, operator))?
            .is_some(),
    };
    Ok(is_opted_in)
}

/// Clears the slashing parameters opt-in status for the given service at current timestamp.
/// This happens only when a new slashing condition is set/updated.
pub fn reset_slashing_opt_in(
    store: &mut dyn Storage,
    env: &Env,
    service: &Service,
) -> Result<(), ContractError> {
    let operator_keys = SLASHING_OPT_IN
        .prefix(service)
        .range(store, None, None, Order::Ascending)
        .map(|item| {
            let (operator, _) = item?;
            Ok(operator)
        })
        .collect::<Vec<StdResult<Operator>>>();

    for operator in operator_keys {
        let key = (service, &operator?);
        SLASHING_OPT_IN.remove(store, key, env.block.time.seconds())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bvs_library::time::{DAYS, MINUTES};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test_is_operator_active() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let operator2 = deps.api.addr_make("operator2");

        // assert that the operator is not active
        let res = is_operator_active(&deps.storage, &operator).unwrap();
        assert!(!res);

        // set the operator active count to 1
        OPERATOR_ACTIVE_REGISTRATION_COUNT
            .save(&mut deps.storage, &operator, &1)
            .expect("OPERATOR_ACTIVE_REGISTRATION_COUNT save failed");

        // assert that the operator is active
        let res = is_operator_active(&deps.storage, &operator).unwrap();
        assert!(res);

        // assert that the operator2 is not active
        let res = is_operator_active(&deps.storage, &operator2).unwrap();
        assert!(!res);
    }

    #[test]
    fn test_require_service_registered() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");

        let res = require_service_registered(&deps.storage, &service);
        assert_eq!(res, Err(ContractError::Std(StdError::not_found("service"))));

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        let res = require_service_registered(&deps.storage, &service);
        assert!(res.is_ok());
    }

    #[test]
    fn test_require_operator_registered() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");

        // assert that the operator is not registered
        let res = require_operator_registered(&deps.storage, &operator);
        assert_eq!(
            res,
            Err(ContractError::Std(StdError::not_found("operator")))
        );

        OPERATORS.save(&mut deps.storage, &operator, &true).unwrap();

        // assert that the operator is registered
        let res = require_operator_registered(&deps.storage, &operator);
        assert!(res.is_ok());
    }

    #[test]
    fn test_registration_status() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        let key = (&operator, &service);

        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::Inactive);

        set_registration_status(&mut deps.storage, &env, key, RegistrationStatus::Active).unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::Active);

        set_registration_status(
            &mut deps.storage,
            &env,
            key,
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::OperatorRegistered);

        set_registration_status(
            &mut deps.storage,
            &env,
            key,
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::ServiceRegistered);
    }

    #[test]
    fn test_slashing_parameters_validate() {
        let deps = mock_dependencies();

        // NEGATIVE tests
        {
            // Invalid destination address
            let valid_slashing_parameters = SlashingParameters {
                destination: Some(Addr::unchecked("invalid_address")),
                max_slashing_bips: 100,
                resolution_window: 60 * MINUTES,
            };

            assert_eq!(
                valid_slashing_parameters.validate(&deps.api).unwrap_err(),
                ContractError::InvalidSlashingParameters {
                    msg: "destination address is invalid".to_string()
                }
            );
        }
        {
            // max_slashing_percentage too high
            let valid_slashing_parameters = SlashingParameters {
                destination: Some(deps.api.addr_make("destination")),
                max_slashing_bips: 10_001,
                resolution_window: 60 * MINUTES,
            };

            assert_eq!(
                valid_slashing_parameters.validate(&deps.api).unwrap_err(),
                ContractError::InvalidSlashingParameters {
                    msg: "max_slashing_percentage is over 10_000 bips (100%)".to_string()
                }
            );
        }

        // POSITIVE tests
        {
            // Valid slashing parameters
            let valid_slashing_parameters = SlashingParameters {
                destination: Some(deps.api.addr_make("destination")),
                max_slashing_bips: 10_000,
                resolution_window: 7 * DAYS,
            };

            assert!(valid_slashing_parameters.validate(&deps.api).is_ok());
        }
        {
            // Valid slashing parameters with None destination
            let valid_slashing_parameters = SlashingParameters {
                destination: None,
                max_slashing_bips: 0,
                resolution_window: 0,
            };

            assert!(valid_slashing_parameters.validate(&deps.api).is_ok());
        }
    }

    #[test]
    fn test_is_operator_opted_in_to_slashing() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let service = deps.api.addr_make("service");
        let operator = deps.api.addr_make("operator");

        // assert that the operator is not opted in
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator, None).unwrap();
        assert!(!res);

        SLASHING_OPT_IN
            .save(
                &mut deps.storage,
                (&service, &operator),
                &true,
                env.block.time.seconds(),
            )
            .unwrap();

        // assert that the operator is opted in
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator, None).unwrap();
        assert!(res);
    }

    #[test]
    fn test_reset_slashing_opt_in() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let service = deps.api.addr_make("service");
        let service2 = deps.api.addr_make("service2");
        let operator = deps.api.addr_make("operator");
        let operator2 = deps.api.addr_make("operator2");

        // set the slashing parameters opt-in status
        {
            SLASHING_OPT_IN
                .save(
                    &mut deps.storage,
                    (&service, &operator),
                    &true,
                    env.block.time.seconds(),
                )
                .unwrap();
            SLASHING_OPT_IN
                .save(
                    &mut deps.storage,
                    (&service, &operator2),
                    &true,
                    env.block.time.seconds(),
                )
                .unwrap();
            SLASHING_OPT_IN
                .save(
                    &mut deps.storage,
                    (&service2, &operator),
                    &true,
                    env.block.time.seconds(),
                )
                .unwrap();
            SLASHING_OPT_IN
                .save(
                    &mut deps.storage,
                    (&service2, &operator2),
                    &true,
                    env.block.time.seconds(),
                )
                .unwrap();
        }

        // move the block time forward
        env.block.time = env.block.time.plus_seconds(10);

        // assert that the operator and operator2 are opted in to service
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator, None).unwrap();
        assert!(res);
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator2, None).unwrap();
        assert!(res);

        // reset the slashing parameters opt-in status for service
        reset_slashing_opt_in(&mut deps.storage, &env, &service).unwrap();

        // move the block time forward
        env.block.time = env.block.time.plus_seconds(10);

        // assert that the operator and operator2 are not opted in to service
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator, None).unwrap();
        assert!(!res);
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service, &operator2, None).unwrap();
        assert!(!res);

        // assert that operator and operator2 are still opted in to service2
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service2, &operator, None).unwrap();
        assert!(res);
        let res =
            is_operator_opted_in_to_slashing(&deps.storage, &service2, &operator2, None).unwrap();
        assert!(res);

        // assert that the operator and operator2 are opted in to service at the previous timestamp
        let res = is_operator_opted_in_to_slashing(
            &deps.storage,
            &service,
            &operator,
            Some(env.block.time.minus_seconds(10).seconds()),
        )
        .unwrap();
        assert!(res);

        let res = is_operator_opted_in_to_slashing(
            &deps.storage,
            &service,
            &operator2,
            Some(env.block.time.minus_seconds(10).seconds()),
        )
        .unwrap();
        assert!(res);
    }
}
