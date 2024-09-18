use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const OWNER: Item<Addr> = Item::new("owner");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const SLASHER: Map<Addr, bool> = Map::new("slasher");
pub const VALIDATOR: Map<Addr, bool> = Map::new("validator");
pub const MINIMAL_SLASH_SIGNATURE: Item<u64> = Item::new("minimal_slash_signature");
