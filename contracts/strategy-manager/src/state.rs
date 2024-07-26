use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct StrategyManagerState {
    pub delegation_manager: Addr,
    pub eigen_pod_manager: Addr,
    pub slasher: Addr,
}

pub const STRATEGY_MANAGER_STATE: Item<StrategyManagerState> = Item::new("strategy_manager_state");
