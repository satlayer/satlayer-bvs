use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use bvs_vault_router::state::SlashingRequestId;
use cw3::{Ballot, Proposal};
use cw_storage_plus::{Item, Map};
use cw_utils::Threshold;

#[cw_serde]
pub struct Config {
    pub threshold: Threshold,
    pub total_weight: u64,
}

/// Stores the configuration of the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the voters' vote for a proposal.
pub const BALLOTS: Map<(SlashingRequestId, &Addr), Ballot> = Map::new("votes");

/// Stores proposals data.
pub const PROPOSALS: Map<SlashingRequestId, Proposal> = Map::new("proposals");

/// Stores the valid voters address and their vote weight.
pub const VOTERS: Map<&Addr, u64> = Map::new("voters");
