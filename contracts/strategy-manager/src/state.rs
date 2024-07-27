use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct StrategyManagerState {
    pub delegation_manager: Addr,
    pub slasher: Addr,
}

pub const STRATEGY_MANAGER_STATE: Item<StrategyManagerState> = Item::new("strategy_manager_state");
pub const STRATEGY_WHITELISTER: Item<Addr> = Item::new("strategy_whitelister");
pub const STRATEGY_WHITELIST: Map<&Addr, bool> = Map::new("strategy_whitelist");
pub const OWNER: Item<Addr> = Item::new("owner");