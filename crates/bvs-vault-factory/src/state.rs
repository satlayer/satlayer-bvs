use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ROUTER: Item<Addr> = Item::new("router");
pub const REGISTRY: Item<Addr> = Item::new("registry");
