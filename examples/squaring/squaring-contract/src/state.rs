use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const MAX_ID: Item<u64> = Item::new("next_id");
pub const AGGREGATOR: Item<Addr> = Item::new("aggregator");
pub const STATE_BANK: Item<Addr> = Item::new("state_bank");
pub const BVS_DRIVER: Item<Addr> = Item::new("bvs_driver");
pub const CREATED_TASKS: Map<u64, i64> = Map::new("created_tasks");
pub const RESPONDED_TASKS: Map<u64, i64> = Map::new("responded_tasks");
