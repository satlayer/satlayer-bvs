use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Map;

type Owner = Addr;
type Proxy = Addr;

/// Mapping of the owner (of shares) and the proxy.  
/// This will allow the proxy to queue and redeem withdrawals on behalf of the owner.
const APPROVED_PROXY: Map<(&Owner, &Proxy), bool> = Map::new("approved_proxy");

pub fn set_approved_proxy(
    storage: &mut dyn Storage,
    owner: &Addr,
    proxy: &Addr,
    is_approved: &bool,
) -> StdResult<()> {
    APPROVED_PROXY.save(storage, (owner, proxy), is_approved)?;
    Ok(())
}

/// Return `true` if the proxy is approved by the owner, otherwise `false`.
pub fn is_approved_proxy(storage: &dyn Storage, owner: &Addr, proxy: &Addr) -> StdResult<bool> {
    let is_approved = APPROVED_PROXY.may_load(storage, (owner, proxy))?;
    Ok(is_approved.unwrap_or(false))
}
