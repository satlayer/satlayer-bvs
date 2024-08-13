use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub enum OperatorAVSRegistrationStatus {
    Registered,
    Unregistered,
}

#[cw_serde]
pub struct AVSInfo {
    pub avs_hash: String,
    pub avs_contract: String,
    pub state_bank: String,
    pub avs_driver: String,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const AVS_OPERATOR_STATUS: Map<(Addr, Addr), OperatorAVSRegistrationStatus> = Map::new("avs_operator_status");
pub const OPERATOR_SALT_SPENT: Map<(Addr, String), bool> = Map::new("operator_salt_is_spent");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const AVS_INFO: Map<String, AVSInfo> = Map::new("avs_info");
