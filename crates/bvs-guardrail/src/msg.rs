use crate::state::ProposalId;
use bvs_library::slashing::SlashingRequestId;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw3::Vote;
use cw4::Member;
use cw_utils::Threshold;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub members: Vec<Member>,
    pub threshold: Threshold,
    pub default_expiration: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Propose {
        slashing_request_id: SlashingRequestId,
        reason: String,
    },
    Vote {
        slashing_request_id: SlashingRequestId,
        vote: Vote,
    },
    Close {
        slashing_request_id: SlashingRequestId,
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

#[cw_serde]
pub struct MigrateMsg {}

#[cfg(test)]
mod tests {
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{to_json_string, Decimal};
    use cw_utils::Threshold;

    #[test]
    fn to_json_string_expected() {
        let api_deps = mock_dependencies();
        let owner = api_deps.api.addr_make("owner");
        let member = api_deps.api.addr_make("member");

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            members: vec![
                cw4::Member {
                    addr: owner.to_string(),
                    weight: 1,
                },
                cw4::Member {
                    addr: member.to_string(),
                    weight: 1,
                },
            ],
            threshold: Threshold::AbsolutePercentage {
                percentage: Decimal::percent(50),
            },
            default_expiration: 60 * 60 * 24 * 100, // 100 days
        };
        let a = to_json_string(&msg).unwrap();
        let b = r#"{"owner":"cosmwasm1fsgzj6t7udv8zhf6zj32mkqhcjcpv52yph5qsdcl0qt94jgdckqs2g053y","members":[{"addr":"cosmwasm1fsgzj6t7udv8zhf6zj32mkqhcjcpv52yph5qsdcl0qt94jgdckqs2g053y","weight":1},{"addr":"cosmwasm1uvdtvs7yfaaqajpykkw3r9xkpkkrxssqmpz7v8fd9zw65rcg06jqfd9qlh","weight":1}],"threshold":{"absolute_percentage":{"percentage":"0.5"}},"default_expiration":8640000}"#;
        assert_eq!(a, b);
    }
}
