use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Map;

type Owner = Addr;
type Controller = Addr;

/// Mapping of the owner (of shares) and the controller
/// that approved the controller to act on behalf of the owner.
/// This will allow the controller to queue and redeem withdrawals on behalf of the owner.
/// This will also give the controller to redeem withdrawals to any recipient.
const APPROVED_CONTROLLER: Map<&Owner, Controller> = Map::new("approved_controller");

pub fn set_approved_controller(
    storage: &mut dyn Storage,
    owner: &Addr,
    controller: &Addr,
) -> StdResult<()> {
    APPROVED_CONTROLLER.save(storage, owner, controller)?;
    Ok(())
}

/// Return `true` if the controller is approved by the owner, otherwise `false`.
pub fn is_approved_controller(
    storage: &dyn Storage,
    owner: &Addr,
    controller: &Addr,
) -> StdResult<bool> {
    let approved_controller = APPROVED_CONTROLLER.may_load(storage, owner)?;
    Ok(match approved_controller {
        Some(approved_controller) => approved_controller == *controller,
        None => false,
    })
}
