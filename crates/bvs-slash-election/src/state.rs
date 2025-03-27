use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

type Operator = Addr;
type Vaults = Vec<Addr>;
type Offender = Operator;

/// This state contains the amount of slash per an offense type
pub const PUNISHMENT: Map<Offense, Uint128> = Map::new("punishment");

/// This state contains the slashing entry
/// The key is composit, made of the accused operator's address, the offense type
/// and the height of the block where the slash was requested
/// The value is the total yes votes
pub const SLASHES: Map<(Operator, Offense, u64), u64> = Map::new("slashes");

/// This state contains the minimum number of yes votes required for a slash to proceed
pub const THRESHOLD: Item<u64> = Item::new("threshold");

pub const REGISTRY: Item<Addr> = Item::new("registry");

pub const VAULTS: Map<Operator, Vaults> = Map::new("vaults");

pub enum Offense {
    DoubleSign,
    MissingBlock,
}
