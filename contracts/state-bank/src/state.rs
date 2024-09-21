use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const VALUES: Map<String, i64> = Map::new("values");
pub const IS_AVS_CONTRACT_REGISTERED: Map<&Addr, bool> = Map::new("is_avs_contract_registered");
