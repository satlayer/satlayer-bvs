use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const VALUES: Map<String, String> = Map::new("values");
pub const IS_BVS_CONTRACT_REGISTERED: Map<&Addr, bool> = Map::new("is_bvs_contract_registered");
