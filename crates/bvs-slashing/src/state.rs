use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::{Item, Map};

/// Downstream custom strategized slashing contracts
/// Map<slasher_addr, operator>
pub const SLASHERS: Map<&Addr, Addr> = Map::new("slashers");

/// SLASH_ENTRIES hold the hash of the slashing event
/// Map<hash, slasher_strategy that this slash entry belongs to>
pub const SLASH_ENTRIES: Map<Binary, Addr> = Map::new("slash_entries");

pub const REGISTRY: Item<Addr> = Item::new("registry");

pub const ROUTER: Item<Addr> = Item::new("router");

pub const VAULT: Item<Addr> = Item::new("vault");
