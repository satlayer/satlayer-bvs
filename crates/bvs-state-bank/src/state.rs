use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const VALUES: Map<String, String> = Map::new("values");
pub const IS_BVS_CONTRACT_REGISTERED: Map<&Addr, bool> = Map::new("is_bvs_contract_registered");
// TODO: there is no use of OWNER in the codebase (remove?)
pub const OWNER: Item<Addr> = Item::new("owner");
