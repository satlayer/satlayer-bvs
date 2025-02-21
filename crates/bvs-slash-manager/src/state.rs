use crate::utils::SlashDetails;
use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::{Item, Map};

pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
pub const SLASHER: Map<Addr, bool> = Map::new("slasher");
pub const VALIDATOR: Map<Addr, bool> = Map::new("validator");
pub const VALIDATOR_PUBKEYS: Map<Addr, Binary> = Map::new("validator_pubkeys");
pub const MINIMAL_SLASH_SIGNATURE: Item<u64> = Item::new("minimal_slash_signature");
pub const SLASH_DETAILS: Map<String, SlashDetails> = Map::new("slash_details");
