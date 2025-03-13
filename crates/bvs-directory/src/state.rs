use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::{Item, Map};

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

pub const OWNER: Item<Addr> = Item::new("owner");
pub const BVS_OPERATOR_STATUS: Map<(Addr, Addr), OperatorBvsRegistrationStatus> =
    Map::new("bvs_operator_status");
pub const OPERATOR_SALT_SPENT: Map<(Addr, String), bool> = Map::new("operator_salt_is_spent");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const BVS_INFO: Map<String, BvsInfo> = Map::new("bvs_info");
pub const OPERATOR: Map<Addr, bool> = Map::new("operator");
pub const OPERATOR_PUBKEYS: Map<Addr, Binary> = Map::new("operator_pubkeys");
