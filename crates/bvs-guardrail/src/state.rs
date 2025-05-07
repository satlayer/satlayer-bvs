use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};

use bvs_library::storage::EVERY_SECOND;
use bvs_vault_router::state::SlashingRequestId;
use cw3::{Ballot, Proposal};
use cw4::{
    MEMBERS_CHANGELOG, MEMBERS_CHECKPOINTS, MEMBERS_KEY, TOTAL_KEY, TOTAL_KEY_CHANGELOG,
    TOTAL_KEY_CHECKPOINTS,
};
use cw4_group::ContractError;
use cw_storage_plus::{Item, Map, SnapshotItem, SnapshotMap, Strategy};
use cw_utils::Threshold;

// aliases for easier readability
pub type ProposalId = u64;
pub type VotingPower = u64;

#[cw_serde]
pub struct Config {
    pub threshold: Threshold,
}

/// Stores the configuration of the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the voting records of proposals by every member.
pub const BALLOTS: Map<(ProposalId, &Addr), Ballot> = Map::new("votes");

/// Stores the proposals
pub const PROPOSALS: Map<ProposalId, Proposal> = Map::new("proposals");

/// Stores the proposal count that is used to generate the next unique proposal id.
pub const PROPOSAL_COUNT: Item<ProposalId> = Item::new("proposal_count");

/// Stores the mapping between the slashing request id and the proposal id.
pub const SLASHING_REQUEST_TO_PROPOSAL: Map<SlashingRequestId, ProposalId> =
    Map::new("slashing_request_to_proposal");

/// Stores the total voting power of the group at every block
pub const TOTAL: SnapshotItem<VotingPower> = SnapshotItem::new(
    TOTAL_KEY,
    TOTAL_KEY_CHECKPOINTS,
    TOTAL_KEY_CHANGELOG,
    Strategy::EveryBlock,
);

/// Stores the member and their voting power at every block
pub const MEMBERS: SnapshotMap<&Addr, VotingPower> = SnapshotMap::new(
    MEMBERS_KEY,
    MEMBERS_CHECKPOINTS,
    MEMBERS_CHANGELOG,
    Strategy::EveryBlock,
);

/// Check if this address is a member, and if its weight is >= 1
/// Returns member's weight in positive case
pub fn get_voting_power(
    store: &dyn Storage,
    member: &Addr,
    height: impl Into<Option<u64>>,
) -> StdResult<Option<VotingPower>> {
    let voting_power = match height.into() {
        Some(h) => MEMBERS.may_load_at_height(store, member, h)?,
        None => MEMBERS.may_load(store, member)?,
    };

    match voting_power {
        Some(weight) if weight >= 1 => Ok(Some(weight)),
        _ => Ok(None),
    }
}

/// generates the next proposal id
pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
    PROPOSAL_COUNT.save(store, &id)?;
    Ok(id)
}
