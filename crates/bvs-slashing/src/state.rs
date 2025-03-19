use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const SLASHER: Item<Addr> = Item::new("slasher");
pub const REGISTRY: Item<Addr> = Item::new("registry");
pub const ROUTER: Item<Addr> = Item::new("router");
pub const VAULT: Item<Addr> = Item::new("vault");
