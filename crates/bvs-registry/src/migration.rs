use crate::{
    state::{increase_service_active_operator_count, REGISTRATION_STATUS},
    RegistrationStatus,
};
use cosmwasm_std::{Addr, DepsMut, Order, StdResult};

pub fn fill_service_active_operators_count(deps: DepsMut) -> StdResult<()> {
    let items = REGISTRATION_STATUS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<((Addr, Addr), u8)>>>()?;

    for ((_op, service), status_u8) in items {
        let status: RegistrationStatus =
            status_u8.try_into().unwrap_or(RegistrationStatus::Inactive);
        if status == RegistrationStatus::Active {
            increase_service_active_operator_count(deps.storage, &service)?;
        }
    }
    Ok(())
}
