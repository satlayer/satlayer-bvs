use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const IS_BVS_CONTRACT_REGISTERED: Map<&Addr, bool> = Map::new("is_bvs_contract_registered");
pub const OWNER: Item<Addr> = Item::new("owner");
