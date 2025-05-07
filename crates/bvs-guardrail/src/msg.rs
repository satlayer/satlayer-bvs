use crate::state::ProposalId;
use bvs_vault_router::state::SlashingRequestId;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw3::{DepositInfo, Status, Vote};
use cw4::Member;
use cw_utils::{Expiration, Threshold, ThresholdResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub members: Vec<Member>,
    pub threshold: Threshold,
}

#[cw_serde]
pub enum ExecuteMsg {
    Propose {
        slash_request_id: SlashingRequestId,
        reason: String,
        expiration: Expiration,
    },
    Vote {
        slash_request_id: SlashingRequestId,
        vote: Vote,
    },
    Close {
        slash_request_id: SlashingRequestId,
    },
    /// apply a diff to the existing members.
    /// remove is applied after add, so if an address is in both, it is removed
    UpdateMembers {
        remove: Vec<String>,
        add: Vec<Member>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw_utils::ThresholdResponse)]
    Threshold { height: Option<u64> },
    #[returns(cw3::ProposalResponse)]
    Proposal { proposal_id: ProposalId },
    #[returns(cw3::ProposalResponse)]
    ProposalBySlashingRequestId {
        slashing_request_id: SlashingRequestId,
    },
    #[returns(cw3::ProposalListResponse)]
    ListProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::ProposalListResponse)]
    ReverseProposals {
        start_before: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::VoteResponse)]
    Vote {
        proposal_id: ProposalId,
        voter: String,
    },
    #[returns(cw3::VoteResponse)]
    VoteBySlashingRequestId {
        slashing_request_id: SlashingRequestId,
        voter: String,
    },
    #[returns(cw3::VoteListResponse)]
    ListVotes {
        proposal_id: ProposalId,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(cw3::VoterResponse)]
    Voter {
        address: String,
        height: Option<u64>,
    },
    #[returns(cw3::VoterListResponse)]
    ListVoters {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}
