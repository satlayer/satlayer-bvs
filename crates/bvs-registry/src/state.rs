use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::{Map, SnapshotMap, Strategy};

type Service = Addr;
type Operator = Addr;

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
    Inactive = 0,
    Active = 1,
    OperatorRegistered = 2,
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
    Strategy::EveryBlock,
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

/// Get the registration status of the Operator to Service at a specific block height
pub fn get_registration_status_at_height(
    store: &dyn Storage,
    key: (&Operator, &Service),
    block_height: u64,
) -> StdResult<RegistrationStatus> {
    let status = REGISTRATION_STATUS
        .may_load_at_height(store, key, block_height)?
        .unwrap_or(RegistrationStatus::Inactive.into());

    status.try_into()
}

/// Set the registration status of the Operator to Service at a specific block height
pub fn set_registration_status(
    store: &mut dyn Storage,
    key: (&Operator, &Service),
    status: RegistrationStatus,
    block_height: u64,
) -> StdResult<()> {
    REGISTRATION_STATUS.save(store, key, &status.into(), block_height)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
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

        set_registration_status(
            &mut deps.storage,
            key,
            RegistrationStatus::Active,
            env.block.height,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::Active);

        set_registration_status(
            &mut deps.storage,
            key,
            RegistrationStatus::OperatorRegistered,
            env.block.height,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::OperatorRegistered);

        set_registration_status(
            &mut deps.storage,
            key,
            RegistrationStatus::ServiceRegistered,
            env.block.height,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::ServiceRegistered);
    }
}
