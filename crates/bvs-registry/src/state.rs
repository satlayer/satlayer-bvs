use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::Map;

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
        return Err(ContractError::ServiceNotFound {});
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
const REGISTRATION_STATUS: Map<(&Operator, &Service), u8> = Map::new("registration_status");

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

/// Set the registration status of the Operator to Service
pub fn set_registration_status(
    store: &mut dyn Storage,
    key: (&Operator, &Service),
    status: RegistrationStatus,
) -> StdResult<()> {
    REGISTRATION_STATUS.save(store, key, &status.into())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_require_service_registered() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");

        let res = require_service_registered(&deps.storage, &service);
        assert_eq!(res, Err(ContractError::ServiceNotFound {}));

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        let res = require_service_registered(&deps.storage, &service);
        assert!(res.is_ok());
    }

    #[test]
    fn test_registration_status() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");

        let key = (&operator, &service);

        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::Inactive);

        set_registration_status(&mut deps.storage, key, RegistrationStatus::Active).unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::Active);

        set_registration_status(
            &mut deps.storage,
            key,
            RegistrationStatus::OperatorRegistered,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::OperatorRegistered);

        set_registration_status(
            &mut deps.storage,
            key,
            RegistrationStatus::ServiceRegistered,
        )
        .unwrap();
        let status = get_registration_status(&deps.storage, key).unwrap();
        assert_eq!(status, RegistrationStatus::ServiceRegistered);
    }
}
