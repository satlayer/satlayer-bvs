use std::error::Error;

use cosmwasm_std::{Addr, StdError, Uint128};
use cw_storage_plus::{Item, Map};

type Operator = Addr;
type Vaults = Vec<Addr>;
type Offender = Operator;
type Voter = Operator;
type Vote = bool;

/// This state contains the amount of slash per an offense type
pub const PUNISHMENT: Map<Offense, Uint128> = Map::new("punishment");

/// This state contains the slashing entry
/// The key is composite, made of the (accused operator's address, the offense type,
/// and the height of the block where the slash was requested)
/// The value is the total yes votes
pub const SLASHES: Map<(&Offender, &str, u64), u64> = Map::new("slashes");

pub const SLASHES_VOTERS: Map<(&Offender, &str, u64), Vec<Voter>> = Map::new("slashes_voters");

/// This state contains the minimum number of yes votes required for a slash to proceed
pub const THRESHOLD: Item<u64> = Item::new("threshold");

pub const REGISTRY: Item<Addr> = Item::new("registry");

pub const VOTE_PERIOD: Item<u64> = Item::new("vote_period");

#[derive(Debug, PartialEq, Eq)]
pub enum Offense {
    DoubleSign,
    MissingBlock,
}

impl Offense {
    pub fn as_str(&self) -> &'static str {
        match self {
            Offense::DoubleSign => "DoubleSign",
            Offense::MissingBlock => "MissingBlock",
        }
    }
}

impl From<&Offense> for &str {
    fn from(value: &Offense) -> Self {
        value.as_str()
    }
}

impl TryFrom<&str> for Offense {
    type Error = StdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DoubleSign" => Ok(Offense::DoubleSign),
            "MissingBlock" => Ok(Offense::MissingBlock),
            _ => Err(StdError::generic_err("Unknown offense variant")),
        }
    }
}
