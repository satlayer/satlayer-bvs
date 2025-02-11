use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct StrategyState {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
    pub total_shares: Uint128,
}

pub const STRATEGY_STATE: Item<StrategyState> = Item::new("strategy_state");
pub const OWNER: Item<Addr> = Item::new("owner");
