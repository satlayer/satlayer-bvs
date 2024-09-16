use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPLOYEDSTRATEGIES: Map<&Addr, Addr> = Map::new("strategies");
pub const IS_BLACKLISTED: Map<&Addr, bool> = Map::new("is_blacklisted");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub strategy_code_id: u64,
    pub strategy_manager: Addr,
    pub pauser: Addr,
    pub unpauser: Addr,
}
