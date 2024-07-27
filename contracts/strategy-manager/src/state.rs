use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct StrategyManagerState {
    pub delegation_manager: Addr,
    pub slasher: Addr,
}

#[cw_serde]
pub struct StakerStrategyShares {
    pub shares: Uint128,
}

pub const STRATEGY_MANAGER_STATE: Item<StrategyManagerState> = Item::new("strategy_manager_state");
pub const STRATEGY_WHITELISTER: Item<Addr> = Item::new("strategy_whitelister");
pub const STRATEGY_WHITELIST: Map<&Addr, bool> = Map::new("strategy_whitelist");
pub const OWNER: Item<Addr> = Item::new("owner");

pub const STAKER_STRATEGY_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("staker_strategy_shares");

pub const STAKER_STRATEGY_LIST: Map<&Addr, Vec<Addr>> = Map::new("staker_strategy_list");
pub const MAX_STAKER_STRATEGY_LIST_LENGTH: usize = 10;

pub const THIRD_PARTY_TRANSFERS_FORBIDDEN: Map<&Addr, bool> = Map::new("third_party_transfers_forbidden");