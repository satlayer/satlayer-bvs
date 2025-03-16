use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage, Uint64};
use cw_storage_plus::{Item, Map};

/// Mapping of vault's Addr to Vault
pub const VAULTS: Map<&Addr, Vault> = Map::new("vaults");

/// Storage for the router address
pub const REGISTRY: Item<Addr> = Item::new("registry");

#[cw_serde]
pub struct Vault {
    pub whitelisted: bool,
}

/// Get the `registry` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub fn get_registry(storage: &dyn Storage) -> StdResult<Addr> {
    REGISTRY
        .may_load(storage)?
        .ok_or(StdError::not_found("registry"))
}

/// Set the `registry` address, called once during `initialization`.
/// The `registry` is the address where the vault calls
pub fn set_registry(storage: &mut dyn Storage, registry: &Addr) -> StdResult<()> {
    REGISTRY.save(storage, registry)?;
    Ok(())
}
/// Store the withdrawal lock period in seconds.
pub const WITHDRAWAL_LOCK_PERIOD: Item<Uint64> = Item::new("withdrawal_lock_period");

/// This is used when the withdrawal lock period is not set.
/// The default value is 7 days.
pub const DEFAULT_WITHDRAWAL_LCOK_PERIOD: Uint64 = Uint64::new(604800);
