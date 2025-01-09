use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const MINTER: Item<Addr> = Item::new("minter");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
