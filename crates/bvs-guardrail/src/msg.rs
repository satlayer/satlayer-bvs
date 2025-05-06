use bvs_vault_router::state::SlashingRequestId;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw3::{DepositInfo, Status, Vote};
use cw_utils::{Expiration, Threshold, ThresholdResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub voters: Vec<Voter>,
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
        proposal_id: SlashingRequestId,
        vote: Vote,
    },
    Close {
        proposal_id: SlashingRequestId,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw_utils::ThresholdResponse)]
    Threshold {},
    #[returns(ProposalResponse)]
    Proposal { proposal_id: SlashingRequestId },
    #[returns(ProposalListResponse)]
    ListProposals {
        skip: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::VoteResponse)]
    Vote {
        proposal_id: SlashingRequestId,
        voter: String,
    },
    #[returns(VoteListResponse)]
    ListVotes {
        proposal_id: SlashingRequestId,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(cw3::VoterResponse)]
    Voter { address: String },
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

/// Adapted from `cw3::ProposalResponse`
#[cw_serde]
pub struct ProposalResponse<T = Empty> {
    pub id: SlashingRequestId,
    pub title: String,
    pub description: String,
    pub msgs: Vec<CosmosMsg<T>>,
    pub status: Status,
    pub expires: Expiration,
    /// This is the threshold that is applied to this proposal. Both
    /// the rules of the voting contract, as well as the total_weight
    /// of the voting group may have changed since this time. That
    /// means that the generic `Threshold{}` query does not provide
    /// valid information for existing proposals.
    pub threshold: ThresholdResponse,
    pub proposer: Addr,
    pub deposit: Option<DepositInfo>,
}

/// Adapted from `cw3::ProposalListResponse`
#[cw_serde]
pub struct ProposalListResponse<T = Empty> {
    pub proposals: Vec<ProposalResponse<T>>,
}

/// Adapted from `cw3::VoteInfo`
#[cw_serde]
pub struct VoteInfo {
    pub proposal_id: SlashingRequestId,
    pub voter: String,
    pub vote: Vote,
    pub weight: u64,
}

/// Adapted from `cw3::VoteListResponse`
#[cw_serde]
pub struct VoteListResponse {
    pub votes: Vec<VoteInfo>,
}

/// Adapted from `cw3::VoteResponse`
#[cw_serde]
pub struct VoteResponse {
    pub vote: Option<VoteInfo>,
}
