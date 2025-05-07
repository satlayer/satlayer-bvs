#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, MEMBERS, TOTAL};
use bvs_library::ownership;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint64,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let cfg = Config {
        threshold: msg.threshold,
    };
    CONFIG.save(deps.storage, &cfg)?;

    let mut members = msg.members;
    cw4_group::helpers::validate_unique_members(&mut members)?;
    let members = members; // let go of mutability

    let mut total = Uint64::zero();
    for member in members.clone().into_iter() {
        let member_weight = Uint64::from(member.weight);
        total = total.checked_add(member_weight)?;
        let member_addr = deps.api.addr_validate(&member.addr)?;
        MEMBERS.save(
            deps.storage,
            &member_addr,
            &member_weight.u64(),
            env.block.height,
        )?;
    }
    TOTAL.save(deps.storage, &total.u64(), env.block.height)?;

    // TODO: need to add voters, threshold, total_weight to events ??
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("threshold", format!("{:?}", cfg.threshold))
        .add_attribute("total_weight", total.to_string())
        .add_attribute("members", members.len().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose {
            slash_request_id,
            reason,
            expiration,
        } => execute::propose(deps, env, info, slash_request_id, reason, expiration),
        ExecuteMsg::Vote {
            slash_request_id,
            vote,
        } => execute::vote(deps, env, info, slash_request_id, vote),
        ExecuteMsg::Close { slash_request_id } => execute::close(deps, env, info, slash_request_id),
        ExecuteMsg::UpdateMembers { remove, add } => {
            execute::update_members(deps, env, info, remove, add)
        }
    }
}

mod execute {
    use super::*;
    use crate::state::{
        get_voting_power, next_id, BALLOTS, PROPOSALS, SLASHING_REQUEST_TO_PROPOSAL,
    };
    use bvs_library::ownership::assert_owner;
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::Event;
    use cw3::{Ballot, Proposal, Status, Vote, Votes};
    use cw4::{Member, MemberDiff};
    use cw_utils::Expiration;

    pub fn propose(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        slash_request_id: SlashingRequestId,
        reason: String,
        expiration: Expiration,
    ) -> Result<Response, ContractError> {
        // only members of the multisig can create a proposal
        let vote_power = MEMBERS
            .may_load(deps.storage, &info.sender)?
            .ok_or(ContractError::Unauthorized {})?;

        let cfg = CONFIG.load(deps.storage)?;

        let total_weight = TOTAL.may_load(deps.storage)?.unwrap_or_default();
        // TODO: do we need to throw err when total_weight is 0 (means no voter) ?

        // create a proposal
        let mut prop = Proposal {
            title: format!("Proposal To Finalize Slash {}", slash_request_id),
            description: reason,
            start_height: env.block.height,
            expires: expiration,
            msgs: vec![], // no msg to execute
            status: Status::Open,
            votes: Votes::yes(vote_power), // proposer will vote yes automatically
            threshold: cfg.threshold,
            total_weight,
            proposer: info.sender.clone(), // proposer is the sender
            deposit: None,
        };
        prop.update_status(&env.block);
        let proposal_id = next_id(deps.storage)?;
        PROPOSALS.save(deps.storage, proposal_id, &prop)?;

        // Add proposer's yes vote into the ballot
        let ballot = Ballot {
            weight: vote_power,
            vote: Vote::Yes,
        };

        BALLOTS.save(deps.storage, (proposal_id, &info.sender), &ballot)?;

        // save mapping of slashing request id to proposal id only if it doesn't exist
        SLASHING_REQUEST_TO_PROPOSAL.update(
            deps.storage,
            slash_request_id.clone(),
            |id| match id {
                Some(_) => Err(ContractError::ProposalAlreadyExists {}),
                None => Ok(proposal_id),
            },
        )?;

        // TODO: these events are different from cw3 specs to be more inlined with our other contracts
        Ok(Response::new().add_event(
            Event::new("propose")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("slash_request_id", slash_request_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status)),
        ))
    }

    pub fn vote(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        slash_request_id: SlashingRequestId,
        vote: Vote, // TODO: check if we want to remove abstain and veto votes
    ) -> Result<Response, ContractError> {
        // get proposal id
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .may_load(deps.storage, slash_request_id.clone())?
            .ok_or(ContractError::ProposalNotFound {})?;

        // ensure proposal exists and can be voted on
        let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
        // Allow voting on Passed and Rejected proposals too,
        if ![Status::Open, Status::Passed, Status::Rejected].contains(&prop.status) {
            return Err(ContractError::NotOpen {});
        }
        // if they are not expired
        if prop.expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // get voter's weight only if weight is >= 1 at start of proposal
        let vote_power = get_voting_power(deps.storage, &info.sender, prop.start_height)?
            .ok_or(ContractError::Unauthorized {})?;

        // cast vote if no vote previously cast
        BALLOTS.update(deps.storage, (proposal_id, &info.sender), |bal| match bal {
            Some(_) => Err(ContractError::AlreadyVoted {}),
            None => Ok(Ballot {
                weight: vote_power,
                vote,
            }),
        })?;

        // update vote tally
        prop.votes.add_vote(vote, vote_power);
        prop.update_status(&env.block);
        PROPOSALS.save(deps.storage, proposal_id, &prop)?;

        Ok(Response::new().add_event(
            Event::new("vote")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("slash_request_id", slash_request_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status))
                .add_attribute("vote", format!("{:?}", vote)),
        ))
    }

    pub fn close(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        slash_request_id: SlashingRequestId,
    ) -> Result<Response, ContractError> {
        // anyone can trigger this

        // get proposal id
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .may_load(deps.storage, slash_request_id.clone())?
            .ok_or(ContractError::ProposalNotFound {})?;

        let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
        if [Status::Executed, Status::Rejected, Status::Passed].contains(&prop.status) {
            return Err(ContractError::WrongCloseStatus {});
        }
        // Avoid closing of Passed due to expiration proposals
        if prop.current_status(&env.block) == Status::Passed {
            return Err(ContractError::WrongCloseStatus {});
        }
        if !prop.expires.is_expired(&env.block) {
            return Err(ContractError::NotExpired {});
        }

        // set it to rejected
        prop.status = Status::Rejected;
        PROPOSALS.save(deps.storage, proposal_id, &prop)?;

        Ok(Response::new().add_event(
            Event::new("close")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("slash_request_id", slash_request_id.to_string()),
        ))
    }

    pub fn update_members(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to_remove: Vec<String>,
        mut to_add: Vec<Member>,
    ) -> Result<Response, ContractError> {
        // only owner can update member
        assert_owner(deps.storage, &info)?;

        cw4_group::helpers::validate_unique_members(&mut to_add)?;
        let to_add = to_add; // let go of mutability

        let mut total = Uint64::from(TOTAL.load(deps.storage)?);
        let mut diffs: Vec<MemberDiff> = vec![];

        // add all new members and update total
        for add in to_add.clone().into_iter() {
            let add_addr = deps.api.addr_validate(&add.addr)?;
            MEMBERS.update(
                deps.storage,
                &add_addr,
                env.block.height,
                |old| -> StdResult<_> {
                    total = total.checked_sub(Uint64::from(old.unwrap_or_default()))?;
                    total = total.checked_add(Uint64::from(add.weight))?;
                    diffs.push(MemberDiff::new(add.addr, old, Some(add.weight)));
                    Ok(add.weight)
                },
            )?;
        }

        // remove members and update total
        for remove in to_remove.clone().into_iter() {
            let remove_addr = deps.api.addr_validate(&remove)?;
            let old = MEMBERS.may_load(deps.storage, &remove_addr)?;
            // Only process this if they were actually in the list before
            if let Some(weight) = old {
                diffs.push(MemberDiff::new(remove, Some(weight), None));
                total = total.checked_sub(Uint64::from(weight))?;
                MEMBERS.remove(deps.storage, &remove_addr, env.block.height)?;
            }
        }

        TOTAL.save(deps.storage, &total.u64(), env.block.height)?;
        Ok(Response::new().add_event(
            Event::new("update_members")
                .add_attribute("added", to_add.len().to_string())
                .add_attribute("removed", to_remove.len().to_string())
                .add_attribute("total_weight", total.to_string()),
        ))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Threshold { height } => to_json_binary(&query::threshold(deps, height)?),
        QueryMsg::Proposal { proposal_id } => {
            to_json_binary(&query::proposal(deps, env, proposal_id)?)
        }
        QueryMsg::ProposalBySlashingRequestId {
            slashing_request_id,
        } => to_json_binary(&query::proposal_by_slashing_request_id(
            deps,
            env,
            slashing_request_id,
        )?),

        QueryMsg::ListProposals { start_after, limit } => {
            to_json_binary(&query::list_proposals(deps, env, start_after, limit)?)
        }
        QueryMsg::ReverseProposals {
            start_before,
            limit,
        } => to_json_binary(&query::reverse_proposals(deps, env, start_before, limit)?),
        QueryMsg::Vote { proposal_id, voter } => {
            to_json_binary(&query::vote(deps, proposal_id, voter)?)
        }
        QueryMsg::VoteBySlashingRequestId {
            slashing_request_id,
            voter,
        } => to_json_binary(&query::vote_by_slashing_request_id(
            deps,
            slashing_request_id,
            voter,
        )?),
        QueryMsg::ListVotes {
            proposal_id,
            start_after,
            limit,
        } => to_json_binary(&query::list_votes(deps, proposal_id, start_after, limit)?),
        QueryMsg::Voter { address, height } => {
            to_json_binary(&query::voter(deps, address, height)?)
        }
        QueryMsg::ListVoters { start_after, limit } => {
            to_json_binary(&query::list_voters(deps, start_after, limit)?)
        }
    }
}

mod query {
    use super::*;
    use crate::state::{ProposalId, BALLOTS, PROPOSALS, SLASHING_REQUEST_TO_PROPOSAL};
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::{BlockInfo, Deps, Order, StdError};
    use cw3::{
        Proposal, ProposalListResponse, ProposalResponse, VoteInfo, VoteListResponse, VoteResponse,
        VoterDetail, VoterListResponse, VoterResponse,
    };
    use cw_storage_plus::Bound;
    use cw_utils::ThresholdResponse;

    pub fn threshold(deps: Deps, height: Option<u64>) -> StdResult<ThresholdResponse> {
        let cfg = CONFIG.load(deps.storage)?;
        let total_weight = match height {
            Some(h) => TOTAL.may_load_at_height(deps.storage, h)?,
            None => TOTAL.may_load(deps.storage)?,
        };
        Ok(cfg.threshold.to_response(total_weight.unwrap_or_default()))
    }

    pub fn proposal(deps: Deps, env: Env, id: ProposalId) -> StdResult<ProposalResponse> {
        let prop = PROPOSALS.load(deps.storage, id)?;
        let status = prop.current_status(&env.block);
        let threshold = prop.threshold.to_response(prop.total_weight);
        Ok(ProposalResponse {
            id,
            title: prop.title,
            description: prop.description,
            msgs: prop.msgs,
            status,
            expires: prop.expires,
            deposit: prop.deposit,
            proposer: prop.proposer,
            threshold,
        })
    }

    pub fn proposal_by_slashing_request_id(
        deps: Deps,
        env: Env,
        slashing_request_id: SlashingRequestId,
    ) -> StdResult<ProposalResponse> {
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .may_load(deps.storage, slashing_request_id.clone())?
            .ok_or(StdError::not_found(format!(
                "No proposal found for slashing request id: {:?}",
                slashing_request_id
            )))?;
        proposal(deps, env, proposal_id)
    }

    // settings for pagination
    const MAX_LIMIT: u32 = 30;
    const DEFAULT_LIMIT: u32 = 10;

    pub fn list_proposals(
        deps: Deps,
        env: Env,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<ProposalListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);
        let proposals = PROPOSALS
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|p| map_proposal(&env.block, p))
            .collect::<StdResult<_>>()?;

        Ok(ProposalListResponse { proposals })
    }

    fn map_proposal(
        block: &BlockInfo,
        item: StdResult<(ProposalId, Proposal)>,
    ) -> StdResult<ProposalResponse> {
        item.map(|(id, prop)| {
            let status = prop.current_status(block);
            let threshold = prop.threshold.to_response(prop.total_weight);
            ProposalResponse {
                id,
                title: prop.title,
                description: prop.description,
                msgs: prop.msgs,
                status,
                deposit: prop.deposit,
                proposer: prop.proposer,
                expires: prop.expires,
                threshold,
            }
        })
    }

    pub fn reverse_proposals(
        deps: Deps,
        env: Env,
        start_before: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<ProposalListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let end = start_before.map(Bound::exclusive);
        let props: StdResult<Vec<_>> = PROPOSALS
            .range(deps.storage, None, end, Order::Descending)
            .take(limit)
            .map(|p| map_proposal(&env.block, p))
            .collect();

        Ok(ProposalListResponse { proposals: props? })
    }

    pub fn vote(deps: Deps, proposal_id: ProposalId, voter: String) -> StdResult<VoteResponse> {
        let voter = deps.api.addr_validate(&voter)?;
        let ballot = BALLOTS.may_load(deps.storage, (proposal_id, &voter))?;
        let vote = ballot.map(|b| VoteInfo {
            proposal_id,
            voter: voter.into(),
            vote: b.vote,
            weight: b.weight,
        });
        Ok(VoteResponse { vote })
    }

    pub fn vote_by_slashing_request_id(
        deps: Deps,
        slashing_request_id: SlashingRequestId,
        voter: String,
    ) -> StdResult<VoteResponse> {
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .may_load(deps.storage, slashing_request_id.clone())?
            .ok_or(StdError::not_found(format!(
                "No proposal found for slashing request id: {:?}",
                slashing_request_id
            )))?;
        vote(deps, proposal_id, voter)
    }

    pub fn list_votes(
        deps: Deps,
        proposal_id: ProposalId,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<VoteListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let votes = BALLOTS
            .prefix(proposal_id)
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, ballot)| VoteInfo {
                    proposal_id,
                    voter: addr.into(),
                    vote: ballot.vote,
                    weight: ballot.weight,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(VoteListResponse { votes })
    }

    pub fn voter(deps: Deps, voter: String, height: Option<u64>) -> StdResult<VoterResponse> {
        let voter = deps.api.addr_validate(&voter)?;
        let weight = match height {
            Some(h) => MEMBERS.may_load_at_height(deps.storage, &voter, h)?,
            None => MEMBERS.may_load(deps.storage, &voter)?,
        };
        Ok(VoterResponse { weight })
    }

    pub fn list_voters(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<VoterListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let voters = MEMBERS
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, weight)| VoterDetail {
                    addr: addr.into(),
                    weight,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(VoterListResponse { voters })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{PROPOSALS, SLASHING_REQUEST_TO_PROPOSAL};
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{Decimal, Event, Order};
    use cw2::{get_contract_version, ContractVersion};
    use cw3::{Proposal, Status, Vote, Votes};
    use cw4::Member;
    use cw_utils::Threshold::AbsolutePercentage;
    use cw_utils::{Expiration, Threshold};

    /// create a 2-of-4 multisig voter
    fn setup(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let api_deps = mock_dependencies();

        let owner = api_deps.api.addr_make("owner");

        let voter1 = api_deps.api.addr_make("voter1");
        let voter2 = api_deps.api.addr_make("voter2");
        let voter3 = api_deps.api.addr_make("voter3");
        let voter4 = api_deps.api.addr_make("voter4");

        let instantiate_msg = InstantiateMsg {
            owner: owner.to_string(),
            members: vec![
                Member {
                    addr: voter1.to_string(),
                    weight: 1,
                },
                Member {
                    addr: voter2.to_string(),
                    weight: 1,
                },
                Member {
                    addr: voter3.to_string(),
                    weight: 1,
                },
                Member {
                    addr: voter4.to_string(),
                    weight: 1,
                },
                Member {
                    addr: owner.to_string(),
                    weight: 0,
                },
            ],
            threshold: Threshold::AbsolutePercentage {
                percentage: Decimal::percent(50), // 50% to pass
            },
        };
        instantiate(deps, env, info, instantiate_msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);

        let voter1 = deps.api.addr_make("voter1");
        let voter2 = deps.api.addr_make("voter2");
        let voter3 = deps.api.addr_make("voter3");
        let voter4 = deps.api.addr_make("voter4");

        // Negative - duplicate voters
        {
            let instantiate_msg = InstantiateMsg {
                owner: owner.to_string(),
                members: vec![
                    Member {
                        addr: voter1.to_string(),
                        weight: 1,
                    },
                    Member {
                        addr: voter1.to_string(),
                        weight: 1,
                    },
                ],
                threshold: Threshold::AbsolutePercentage {
                    percentage: Decimal::percent(50),
                },
            };
            let err = instantiate(
                deps.as_mut(),
                mock_env(),
                info.clone(),
                instantiate_msg.clone(),
            )
            .unwrap_err();
            assert_eq!(
                err,
                ContractError::Cw4Group(cw4_group::ContractError::DuplicateMember {
                    member: voter1.to_string(),
                })
            );
        }

        // Setup
        let res = setup(deps.as_mut(), env, info).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("owner", owner.clone())
                .add_attribute(
                    "threshold",
                    format!(
                        "{:?}",
                        AbsolutePercentage {
                            percentage: Decimal::percent(50)
                        }
                    )
                )
                .add_attribute("total_weight", "4")
                .add_attribute("members", "5")
        );

        // Verify
        assert_eq!(
            ContractVersion {
                contract: CONTRACT_NAME.to_string(),
                version: CONTRACT_VERSION.to_string(),
            },
            get_contract_version(&deps.storage).unwrap()
        );

        // assert config is set
        let cfg = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            cfg,
            Config {
                threshold: Threshold::AbsolutePercentage {
                    percentage: Decimal::percent(50)
                },
            }
        );

        // assert voters are set
        let mut voters = MEMBERS
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        voters.sort();

        let mut expected_voters = vec![
            (voter1, 1),
            (voter2, 1),
            (voter3, 1),
            (voter4, 1),
            (owner, 0),
        ];
        expected_voters.sort();

        assert_eq!(voters, expected_voters)
    }

    #[test]
    fn test_execute_propose() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        let slash_request_id = SlashingRequestId::from_hex(
            "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
        )
        .unwrap();
        let expiration = Expiration::AtTime(env.block.time.plus_seconds(100));
        let execute_msg = ExecuteMsg::Propose {
            slash_request_id: slash_request_id.clone(),
            reason: "test".to_string(),
            expiration,
        };

        // Negative - not a member
        {
            let not_voter = deps.api.addr_make("not_voter");
            let info = message_info(&not_voter, &[]);
            let err = execute(deps.as_mut(), env.clone(), info, execute_msg.clone()).unwrap_err();
            assert_eq!(err, ContractError::Unauthorized {});
        }

        // execute propose successfully
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            execute_msg.clone(),
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("propose")
                    .add_attribute("sender", info.clone().sender)
                    .add_attribute("proposal_id", 1.to_string())
                    .add_attribute("slash_request_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Open))
            ))
        );

        // assert the proposal is saved
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        assert_eq!(proposal_id, 1);
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(
            prop,
            Proposal {
                title:"Proposal To Finalize Slash cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb".to_string(),
                description: "test".to_string(),
                start_height: env.block.height,
                expires: expiration,
                msgs: vec![],
                status: Status::Open,
                votes: Votes::yes(0),
                threshold: Threshold::AbsolutePercentage {
                    percentage: Decimal::percent(50)
                },
                total_weight: 4,
                proposer: owner.clone(),
                deposit: None,
            }
        );

        // Negative - proposal already exists
        {
            let err = execute(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                execute_msg.clone(),
            )
            .unwrap_err();
            assert_eq!(err, ContractError::ProposalAlreadyExists {});
        }
    }

    #[test]
    fn test_execute_vote_passed() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // move to next block
        env.block.height += 1;
        env.block.time = env.block.time.plus_seconds(10);

        // execute proposal
        let slash_request_id = SlashingRequestId::from_hex(
            "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
        )
        .unwrap();
        let expiration = Expiration::AtTime(env.block.time.plus_seconds(100));
        let execute_msg = ExecuteMsg::Propose {
            slash_request_id: slash_request_id.clone(),
            reason: "test".to_string(),
            expiration,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // Negative - not a member
        {
            let not_voter = deps.api.addr_make("not_voter");
            let info = message_info(&not_voter, &[]);
            let err = execute(
                deps.as_mut(),
                env.clone(),
                info,
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            )
            .unwrap_err();
            assert_eq!(err, ContractError::Unauthorized {});
        }

        // Negative - proposal does not exist
        {
            let err = execute(
                deps.as_mut(),
                env.clone(),
                voter1_info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: SlashingRequestId::from_hex(
                        "0ff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09",
                    )
                    .unwrap(),
                    vote: Vote::Yes,
                },
            )
            .unwrap_err();
            assert_eq!(err, ContractError::ProposalNotFound {});
        }

        // Negative - owner (proposer) cannot vote due to 0 weight
        {
            let err = execute(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            )
            .unwrap_err();
            assert_eq!(err, ContractError::Unauthorized {});
        }

        // execute vote YES successfully for voter1
        let res = execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter1.to_string())
                    .add_attribute("proposal_id", 1.to_string())
                    .add_attribute("slash_request_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Open))
                    .add_attribute("vote", format!("{:?}", Vote::Yes))
            ))
        );

        // execute vote No for voter2
        {
            let voter2 = deps.api.addr_make("voter2");
            let voter2_info = message_info(&voter2, &[]);
            execute(
                deps.as_mut(),
                env.clone(),
                voter2_info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::No,
                },
            )
            .unwrap();
        }

        // check that the proposal status is still open
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Open);

        // execute vote Abstain for voter3
        {
            let voter3 = deps.api.addr_make("voter3");
            let voter3_info = message_info(&voter3, &[]);
            let res = execute(
                deps.as_mut(),
                env.clone(),
                voter3_info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Abstain,
                },
            );

            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter3.to_string())
                        .add_attribute("proposal_id", proposal_id.to_string())
                        .add_attribute("slash_request_id", slash_request_id.to_string())
                        .add_attribute("status", format!("{:?}", Status::Open))
                        .add_attribute("vote", format!("{:?}", Vote::Abstain))
                ))
            );
        }

        // check that the proposal status is still open
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Open);

        // execute vote Yes for voter4 to pass the proposal (>50%)
        {
            let voter4 = deps.api.addr_make("voter4");
            let voter4_info = message_info(&voter4, &[]);
            let res = execute(
                deps.as_mut(),
                env.clone(),
                voter4_info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            );

            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter4.to_string())
                        .add_attribute("proposal_id", proposal_id.to_string())
                        .add_attribute("slash_request_id", slash_request_id.to_string())
                        .add_attribute("status", format!("{:?}", Status::Passed))
                        .add_attribute("vote", format!("{:?}", Vote::Yes))
                ))
            );
        }

        // check that the proposal status is passed
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Passed);
    }

    #[test]
    fn test_execute_vote_rejected() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // move to next block
        env.block.height += 1;
        env.block.time = env.block.time.plus_seconds(10);

        // execute proposal
        let slash_request_id = SlashingRequestId::from_hex(
            "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
        )
        .unwrap();
        let expiration = Expiration::AtTime(env.block.time.plus_seconds(100));
        let execute_msg = ExecuteMsg::Propose {
            slash_request_id: slash_request_id.clone(),
            reason: "test".to_string(),
            expiration,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();

        // execute vote No for voter1
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::No,
            },
        )
        .unwrap();

        // execute vote No for voter2
        let voter2 = deps.api.addr_make("voter2");
        let voter2_info = message_info(&voter2, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter2_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::No,
            },
        )
        .unwrap();

        // execute vote No for voter3 to reject the proposal
        let voter3 = deps.api.addr_make("voter3");
        let voter3_info = message_info(&voter3, &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            voter3_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::No,
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter3.to_string())
                    .add_attribute("proposal_id", proposal_id.to_string())
                    .add_attribute("slash_request_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Rejected))
                    .add_attribute("vote", format!("{:?}", Vote::No))
            ))
        );

        // assert that the proposal status is rejected
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Rejected);

        // voter can still vote on a rejected proposal as long as it is not expired
        // execute vote Yes for voter4
        {
            let voter4 = deps.api.addr_make("voter4");
            let voter4_info = message_info(&voter4, &[]);
            let res = execute(
                deps.as_mut(),
                env.clone(),
                voter4_info.clone(),
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            );
            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter4.to_string())
                        .add_attribute("proposal_id", proposal_id.to_string())
                        .add_attribute("slash_request_id", slash_request_id.to_string())
                        .add_attribute("status", format!("{:?}", Status::Rejected))
                        .add_attribute("vote", format!("{:?}", Vote::Yes))
                ))
            );
        }
    }

    #[test]
    fn test_execute_close() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // move to next block
        env.block.height += 1;
        env.block.time = env.block.time.plus_seconds(10);

        // execute proposal
        let slash_request_id = SlashingRequestId::from_hex(
            "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
        )
        .unwrap();
        let expiration = Expiration::AtTime(env.block.time.plus_seconds(100));
        let execute_msg = ExecuteMsg::Propose {
            slash_request_id: slash_request_id.clone(),
            reason: "test".to_string(),
            expiration,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();

        // execute vote Yes successfully for voter1
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter1.to_string())
                    .add_attribute("proposal_id", proposal_id.to_string())
                    .add_attribute("slash_request_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Open))
                    .add_attribute("vote", format!("{:?}", Vote::Yes))
            ))
        );

        // move the env time to expire the proposal
        env.block.time = env.block.time.plus_seconds(1000);

        // Negative - voters will not be able to vote after expiration
        {
            let voter2 = deps.api.addr_make("voter2");
            let voter2_info = message_info(&voter2, &[]);
            let err = execute(
                deps.as_mut(),
                env.clone(),
                voter2_info,
                ExecuteMsg::Vote {
                    slash_request_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            )
            .unwrap_err();
            assert_eq!(err, ContractError::Expired {});
        }

        // execute close successfully
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Close {
                slash_request_id: slash_request_id.clone(),
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("close")
                    .add_attribute("sender", info.sender)
                    .add_attribute("proposal_id", proposal_id.to_string())
                    .add_attribute("slash_request_id", slash_request_id.to_string())
            ))
        );

        // check that the proposal status is rejected
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.status, Status::Rejected);
    }

    #[test]
    fn test_execute_close_on_passed_proposal() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // move to next block
        env.block.height += 1;
        env.block.time = env.block.time.plus_seconds(10);

        // execute proposal
        let slash_request_id = SlashingRequestId::from_hex(
            "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
        )
        .unwrap();
        let expiration = Expiration::AtTime(env.block.time.plus_seconds(100));
        let execute_msg = ExecuteMsg::Propose {
            slash_request_id: slash_request_id.clone(),
            reason: "test".to_string(),
            expiration,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // execute vote Yes successfully for voter1, voter2 and voter3 (to pass the proposal)
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        )
        .unwrap();

        let voter2 = deps.api.addr_make("voter2");
        let voter2_info = message_info(&voter2, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter2_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        )
        .unwrap();

        let voter3 = deps.api.addr_make("voter3");
        let voter3_info = message_info(&voter3, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter3_info.clone(),
            ExecuteMsg::Vote {
                slash_request_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        )
        .unwrap();

        // check that the proposal status is passed
        let proposal_id = SLASHING_REQUEST_TO_PROPOSAL
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Passed);

        // move the env time to expire the proposal
        env.block.time = env.block.time.plus_seconds(100);

        // execute close failed due to proposal already passed despite expiration
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Close {
                slash_request_id: slash_request_id.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::WrongCloseStatus {});

        // check that the proposal status is still passed
        let prop = PROPOSALS.load(deps.as_ref().storage, proposal_id).unwrap();
        assert_eq!(prop.status, Status::Passed);
    }

    #[test]
    fn test_update_members() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);

        // Setup
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // added to member as init setup
        let voter1 = deps.api.addr_make("voter1");
        let voter2 = deps.api.addr_make("voter2");
        let voter3 = deps.api.addr_make("voter3");
        let voter4 = deps.api.addr_make("voter4");

        // not added to member as init setup
        let voter5 = deps.api.addr_make("voter5");
        let voter6 = deps.api.addr_make("voter6");

        // move to next block
        env.block.height += 1;
        env.block.time = env.block.time.plus_seconds(10);

        // Negative - update with duplicate addr in to_add
        {
            let to_add = vec![
                Member {
                    addr: voter5.to_string(),
                    weight: 5,
                },
                Member {
                    addr: voter5.to_string(),
                    weight: 6,
                },
            ];
            let update_msg = ExecuteMsg::UpdateMembers {
                remove: vec![],
                add: to_add,
            };
            let err =
                execute(deps.as_mut(), env.clone(), info.clone(), update_msg.clone()).unwrap_err();
            assert_eq!(
                err,
                ContractError::Cw4Group(cw4_group::ContractError::DuplicateMember {
                    member: voter5.to_string()
                })
            );
        }

        // Add voter5 and remove voter1
        let update_msg = ExecuteMsg::UpdateMembers {
            remove: vec![voter1.to_string()],
            add: vec![Member {
                addr: voter5.to_string(),
                weight: 1,
            }],
        };

        // execute update successfully
        let res = execute(deps.as_mut(), env.clone(), info.clone(), update_msg.clone());

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("update_members")
                    .add_attribute("added", "1")
                    .add_attribute("removed", "1")
                    .add_attribute("total_weight", "4")
            ))
        );

        // assert total weight is updated
        let total = TOTAL.load(deps.as_ref().storage).unwrap();
        assert_eq!(total, 4);

        // assert the updated members are saved
        let mut members = MEMBERS
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        members.sort();

        let mut expected_members = vec![
            (owner.clone(), 0),
            (voter2.clone(), 1),
            (voter3.clone(), 1),
            (voter4.clone(), 1),
            (voter5.clone(), 1),
        ];
        expected_members.sort();

        assert_eq!(members, expected_members);

        // add voter5 weight to 2 and add new voter6
        let update_msg = ExecuteMsg::UpdateMembers {
            remove: vec![],
            add: vec![
                Member {
                    addr: voter5.to_string(),
                    weight: 2, // update weight
                },
                Member {
                    addr: voter6.to_string(),
                    weight: 1,
                },
            ],
        };
        // execute update successfully
        let res = execute(deps.as_mut(), env.clone(), info.clone(), update_msg.clone());
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("update_members")
                    .add_attribute("added", "2")
                    .add_attribute("removed", "0")
                    .add_attribute("total_weight", "6")
            ))
        );

        // assert total weight is updated
        let total = TOTAL.load(deps.as_ref().storage).unwrap();
        assert_eq!(total, 6);

        // assert the updated members are saved
        let mut members = MEMBERS
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        members.sort();
        let mut expected_members = vec![
            (owner, 0),
            (voter2, 1),
            (voter3, 1),
            (voter4, 1),
            (voter5.clone(), 2),
            (voter6.clone(), 1),
        ];
        expected_members.sort();
        assert_eq!(members, expected_members);
    }
}
