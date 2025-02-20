use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPLOYED_STRATEGIES: Map<&Addr, Addr> = Map::new("strategies");
pub const IS_BLACKLISTED: Map<&Addr, bool> = Map::new("is_blacklisted");

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub strategy_code_id: u64,
    pub strategy_manager: Addr,
    pub pauser: Addr,
    pub unpauser: Addr,
}
