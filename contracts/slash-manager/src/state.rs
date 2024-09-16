use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");