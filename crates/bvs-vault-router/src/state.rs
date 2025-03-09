use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const VAULTS: Map<&Addr, Vault> = Map::new("vaults");

#[cw_serde]
pub struct Vault {
    pub whitelisted: bool,
}
