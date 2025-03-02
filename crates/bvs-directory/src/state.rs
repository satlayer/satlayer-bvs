use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

#[cw_serde]
pub enum OperatorBvsRegistrationStatus {
    Registered,
    Unregistered,
}

#[cw_serde]
pub struct BvsInfo {
    pub bvs_hash: String,
    pub bvs_contract: String,
}

pub const BVS_OPERATOR_STATUS: Map<(&Addr, &Addr), OperatorBvsRegistrationStatus> =
    Map::new("bvs_operator_status");

// TODO: should be stored as Map<(&Addr, &[u8]), bool>
pub const OPERATOR_SALT_SPENT: Map<(&Addr, &String), bool> = Map::new("operator_salt_is_spent");

// TODO: should be stored as Map<(&Binary, BvsInfo), bool>
pub const BVS_INFO: Map<&String, BvsInfo> = Map::new("bvs_info");
