use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::Item;

#[cw_serde]
pub struct StrategyState {
    pub strategy_manager: Addr,
    pub underlying_token: Addr,
    pub total_shares: Uint256,
}

pub const STRATEGY_STATE: Item<StrategyState> = Item::new("strategy_state");
