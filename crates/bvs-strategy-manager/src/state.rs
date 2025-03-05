use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const STRATEGY_IS_WHITELISTED_FOR_DEPOSIT: Map<&Addr, bool> = Map::new("strategy_whitelist");

// DEPLOYED_STRATEGIES and IS_BLACKLISTED are previously handled by factory
// Putting just Addr in type defs is not very useful
// local alias so we know how these state are indexed by and know what store what
type TokenContractAddr = Addr;
type StrategyContractAddr = Addr;
pub const DEPLOYED_STRATEGIES: Map<&TokenContractAddr, StrategyContractAddr> =
    Map::new("strategies");
pub const IS_BLACKLISTED: Map<&TokenContractAddr, bool> = Map::new("is_blacklisted");

pub const STAKER_STRATEGY_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("staker_strategy_shares");

pub const STAKER_STRATEGY_LIST: Map<&Addr, Vec<Addr>> = Map::new("staker_strategy_list");
pub const MAX_STAKER_STRATEGY_LIST_LENGTH: usize = 10;
