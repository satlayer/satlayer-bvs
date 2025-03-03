use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::Map;

type Service = Addr;
type Operator = Addr;

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

/// Default state is Inactive,
/// neither Operator nor Service have registered
/// or one of them has Deregistered
impl Default for RegistrationStatus {
    fn default() -> Self {
        RegistrationStatus::Inactive
    }
}

impl Into<u8> for RegistrationStatus {
    fn into(self) -> u8 {
        self as u8
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

/// Mapping of service address to boolean value
/// indicating if the service is registered with the directory
pub const SERVICES: Map<&Service, bool> = Map::new("services");

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

/// Mapping of (operator_service) address.
/// See `RegistrationStatus` for more of the status
const REGISTRATION_STATUS: Map<(&Operator, &Service), u8> = Map::new("registration_status");

pub fn get_registration_status(
    store: &dyn Storage,
    key: (&Operator, &Service),
) -> StdResult<RegistrationStatus> {
    let status = REGISTRATION_STATUS
        .may_load(store, key)?
        .unwrap_or(RegistrationStatus::Inactive.into());

    Ok(status.try_into()?)
}

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
    fn require_service_registered_works() {
        let mut deps = mock_dependencies();

        let service = deps.api.addr_make("service");

        let res = require_service_registered(&deps.storage, &service);
        assert_eq!(res, Err(ContractError::ServiceNotFound {}));

        SERVICES.save(&mut deps.storage, &service, &true).unwrap();

        let res = require_service_registered(&deps.storage, &service);
        assert_eq!(res, Ok(()));
    }
}
