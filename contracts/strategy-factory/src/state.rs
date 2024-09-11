use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STRATEGIES: Map<&Addr, Addr> = Map::new("strategies");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub strategy_code_id: u64,
    pub only_owner_can_create: bool,
    pub strategy_manager: Addr,
    pub pauser: Addr,
    pub unpauser: Addr,
}
