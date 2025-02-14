use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPLOYED_STRATEGIES: Map<&Addr, Addr> = Map::new("strategies");
pub const IS_BLACKLISTED: Map<&Addr, bool> = Map::new("is_blacklisted");
pub const PENDING_OWNER: Item<Option<Addr>> = Item::new("pending_owner");
pub const PENDING_TOKENS: Map<u64, Addr> = Map::new("pending_tokens");
pub const NEXT_DEPLOY_ID: Item<u64> = Item::new("next_deploy_id");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub strategy_code_id: u64,
    pub strategy_manager: Addr,
    pub pauser: Addr,
    pub unpauser: Addr,
}
