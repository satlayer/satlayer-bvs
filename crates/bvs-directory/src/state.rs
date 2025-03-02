use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::Map;

type Service = Addr;
type Operator = Addr;

/// Registered status of the Operator to Service
/// Can be initiated by the Operator or the Service
/// Becomes Active when the Operator and Service both have registered
/// Becomes Inactive when the Operator or Service have unregistered (default state)
#[cw_serde]
pub enum RegisteredStatus {
    Active,
    Inactive,
    OperatorRegistered,
    ServiceRegistered,
}

/// Default state is Inactive,
/// neither Operator nor Service have registered
/// or one of them has Deregistered
impl Default for RegisteredStatus {
    fn default() -> Self {
        RegisteredStatus::Inactive
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
/// See `RegisteredStatus` for more of the status
pub const REGISTRATION_STATUS: Map<(&Operator, &Service), RegisteredStatus> =
    Map::new("registration_status");

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
