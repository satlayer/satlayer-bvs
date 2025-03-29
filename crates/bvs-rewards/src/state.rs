use cosmwasm_std::{Addr, HexBinary, Uint128};
use cw_storage_plus::Map;

type Service = Addr;
type Earner = Addr;
type Token = String;
type Root = HexBinary;

/// Stores the latest distribution roots for each (service, token) pair
pub const DISTRIBUTION_ROOTS: Map<(&Service, &Token), HexBinary> = Map::new("distribution_roots");

/// Stores the live balances of each (service, token) pair
pub const BALANCES: Map<(&Service, &Token), Uint128> = Map::new("balances");

/// Stores the total claimed rewards of each (service, token, earner) pair
pub const CLAIMED_REWARDS: Map<(&Service, &Token, &Earner), Uint128> = Map::new("claimed_rewards");
