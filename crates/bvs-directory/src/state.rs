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

pub const BVS_OPERATOR_STATUS: Map<(Addr, Addr), OperatorBvsRegistrationStatus> =
    Map::new("bvs_operator_status");
pub const OPERATOR_SALT_SPENT: Map<(Addr, String), bool> = Map::new("operator_salt_is_spent");
pub const BVS_INFO: Map<String, BvsInfo> = Map::new("bvs_info");
