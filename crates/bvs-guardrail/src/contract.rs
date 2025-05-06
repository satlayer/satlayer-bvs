#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, VOTERS};
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    if msg.voters.is_empty() {
        return Err(ContractError::NoVoters {});
    }
    let total_weight = msg.voters.iter().map(|v| v.weight).sum();

    msg.threshold.validate(total_weight)?;

    let cfg = Config {
        threshold: msg.threshold,
        total_weight,
    };
    CONFIG.save(deps.storage, &cfg)?;

    // add all voters
    for voter in msg.voters.iter() {
        let key = deps.api.addr_validate(&voter.addr)?;
        VOTERS.save(deps.storage, &key, &voter.weight)?;
    }

    // TODO: need to add voters, threshold, total_weight to events ??
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
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
        ExecuteMsg::Vote { proposal_id, vote } => execute::vote(deps, env, info, proposal_id, vote),
        ExecuteMsg::Close { proposal_id } => execute::close(deps, env, info, proposal_id),
    }
}

mod execute {
    use super::*;
    use crate::state::{BALLOTS, PROPOSALS};
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::Event;
    use cw3::{Ballot, Proposal, Status, Vote, Votes};
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
        let vote_power = VOTERS
            .may_load(deps.storage, &info.sender)?
            .ok_or(ContractError::Unauthorized {})?;

        let cfg = CONFIG.load(deps.storage)?;

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
            total_weight: cfg.total_weight,
            proposer: info.sender.clone(), // proposer is the sender
            deposit: None,
        };
        prop.update_status(&env.block);
        let proposal_id = slash_request_id;

        // save proposal only if it doesn't exist
        PROPOSALS.update(deps.storage, proposal_id.clone(), |p| match p {
            Some(_) => Err(ContractError::ProposalAlreadyExists {}),
            None => Ok(prop.clone()),
        })?;

        // Add proposer's yes vote into the ballot
        let ballot = Ballot {
            weight: vote_power,
            vote: Vote::Yes,
        };

        BALLOTS.save(deps.storage, (proposal_id.clone(), &info.sender), &ballot)?;

        // TODO: these events are different from cw3 specs to be more inlined with our other contracts
        Ok(Response::new().add_event(
            Event::new("propose")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status)),
        ))
    }

    pub fn vote(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        proposal_id: SlashingRequestId,
        vote: Vote, // TODO: check if we want to remove abstain and veto votes
    ) -> Result<Response, ContractError> {
        // only members of the multisig with weight >= 1 can vote
        let voter_power = VOTERS.may_load(deps.storage, &info.sender)?;
        let vote_power = match voter_power {
            Some(power) if power >= 1 => power,
            _ => return Err(ContractError::Unauthorized {}),
        };

        // ensure proposal exists and can be voted on
        let mut prop = PROPOSALS.load(deps.storage, proposal_id.clone())?;
        // Allow voting on Passed and Rejected proposals too,
        if ![Status::Open, Status::Passed, Status::Rejected].contains(&prop.status) {
            return Err(ContractError::NotOpen {});
        }
        // if they are not expired
        if prop.expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // cast vote if no vote previously cast
        BALLOTS.update(
            deps.storage,
            (proposal_id.clone(), &info.sender),
            |bal| match bal {
                Some(_) => Err(ContractError::AlreadyVoted {}),
                None => Ok(Ballot {
                    weight: vote_power,
                    vote,
                }),
            },
        )?;

        // update vote tally
        prop.votes.add_vote(vote, vote_power);
        prop.update_status(&env.block);
        PROPOSALS.save(deps.storage, proposal_id.clone(), &prop)?;

        Ok(Response::new().add_event(
            Event::new("vote")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status))
                .add_attribute("vote", format!("{:?}", vote)),
        ))
    }

    pub fn close(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        proposal_id: SlashingRequestId,
    ) -> Result<Response, ContractError> {
        // anyone can trigger this

        let mut prop = PROPOSALS.load(deps.storage, proposal_id.clone())?;
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
        PROPOSALS.save(deps.storage, proposal_id.clone(), &prop)?;

        Ok(Response::new().add_event(
            Event::new("close")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string()),
        ))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Threshold {} => to_json_binary(&query::threshold(deps)?),
        QueryMsg::Proposal { proposal_id } => {
            to_json_binary(&query::proposal(deps, env, proposal_id)?)
        }
        QueryMsg::Vote { proposal_id, voter } => {
            to_json_binary(&query::vote(deps, proposal_id, voter)?)
        }
        QueryMsg::ListProposals { skip, limit } => {
            to_json_binary(&query::list_proposals(deps, env, skip, limit)?)
        }
        QueryMsg::ListVotes {
            proposal_id,
            start_after,
            limit,
        } => to_json_binary(&query::list_votes(deps, proposal_id, start_after, limit)?),
        QueryMsg::Voter { address } => to_json_binary(&query::voter(deps, address)?),
        QueryMsg::ListVoters { start_after, limit } => {
            to_json_binary(&query::list_voters(deps, start_after, limit)?)
        }
    }
}

mod query {
    use super::*;
    use crate::msg::{
        ProposalListResponse, ProposalResponse, VoteInfo, VoteListResponse, VoteResponse,
    };
    use crate::state::{BALLOTS, PROPOSALS};
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::{BlockInfo, Deps, Order};
    use cw3::{Proposal, VoterDetail, VoterListResponse, VoterResponse};
    use cw_storage_plus::Bound;
    use cw_utils::ThresholdResponse;

    pub fn threshold(deps: Deps) -> StdResult<ThresholdResponse> {
        let cfg = CONFIG.load(deps.storage)?;
        Ok(cfg.threshold.to_response(cfg.total_weight))
    }

    pub fn proposal(deps: Deps, env: Env, id: SlashingRequestId) -> StdResult<ProposalResponse> {
        let prop = PROPOSALS.load(deps.storage, id.clone())?;
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

    // settings for pagination
    const MAX_LIMIT: u32 = 30;
    const DEFAULT_LIMIT: u32 = 10;

    pub fn list_proposals(
        deps: Deps,
        env: Env,
        skip: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<ProposalListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let skip = skip.unwrap_or(0) as usize;
        let proposals = PROPOSALS
            .range(deps.storage, None, None, Order::Ascending)
            .skip(skip)
            .take(limit)
            .map(|p| map_proposal(&env.block, p))
            .collect::<StdResult<_>>()?;

        Ok(ProposalListResponse { proposals })
    }

    fn map_proposal(
        block: &BlockInfo,
        item: StdResult<(SlashingRequestId, Proposal)>,
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

    pub fn vote(
        deps: Deps,
        proposal_id: SlashingRequestId,
        voter: String,
    ) -> StdResult<VoteResponse> {
        let voter = deps.api.addr_validate(&voter)?;
        let ballot = BALLOTS.may_load(deps.storage, (proposal_id.clone(), &voter))?;
        let vote = ballot.map(|b| VoteInfo {
            proposal_id,
            voter: voter.into(),
            vote: b.vote,
            weight: b.weight,
        });
        Ok(VoteResponse { vote })
    }

    pub fn list_votes(
        deps: Deps,
        proposal_id: SlashingRequestId,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<VoteListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let votes = BALLOTS
            .prefix(proposal_id.clone())
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, ballot)| VoteInfo {
                    proposal_id: proposal_id.clone(),
                    voter: addr.into(),
                    vote: ballot.vote,
                    weight: ballot.weight,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(VoteListResponse { votes })
    }

    pub fn voter(deps: Deps, voter: String) -> StdResult<VoterResponse> {
        let voter = deps.api.addr_validate(&voter)?;
        let weight = VOTERS.may_load(deps.storage, &voter)?;
        Ok(VoterResponse { weight })
    }

    pub fn list_voters(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<VoterListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let voters = VOTERS
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
    use crate::msg::Voter;
    use crate::state::PROPOSALS;
    use bvs_vault_router::state::SlashingRequestId;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{Decimal, Event, Order, StdError};
    use cw2::{get_contract_version, ContractVersion};
    use cw3::{Proposal, Status, Vote, Votes};
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
            voters: vec![
                Voter {
                    addr: voter1.to_string(),
                    weight: 1,
                },
                Voter {
                    addr: voter2.to_string(),
                    weight: 1,
                },
                Voter {
                    addr: voter3.to_string(),
                    weight: 1,
                },
                Voter {
                    addr: voter4.to_string(),
                    weight: 1,
                },
                Voter {
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

        // Negative - no voters passed
        {
            let instantiate_msg = InstantiateMsg {
                owner: owner.to_string(),
                voters: vec![],
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
            assert_eq!(err, ContractError::NoVoters {});
        }

        // Setup
        setup(deps.as_mut(), env, info).unwrap();

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
                total_weight: 4,
            }
        );

        // assert voters are set
        let mut voters = VOTERS
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();

        assert_eq!(
            voters.sort(),
            vec![
                (voter1, 1),
                (voter2, 1),
                (voter3, 1),
                (voter4, 1),
                (owner, 0)
            ]
            .sort()
        )
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
                    .add_attribute("proposal_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Open))
            ))
        );

        // assert the proposal is saved
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
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
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

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
                    proposal_id: slash_request_id.clone(),
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
                    proposal_id: SlashingRequestId::from_hex(
                        "0ff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09",
                    )
                    .unwrap(),
                    vote: Vote::Yes,
                },
            )
            .unwrap_err();
            assert_eq!(
                err,
                ContractError::Std(StdError::not_found("type: cw3::proposal::Proposal; key: [00, 09, 70, 72, 6F, 70, 6F, 73, 61, 6C, 73, 0F, F7, A6, F4, 03, EF, F6, 32, 63, 65, 33, 66, 0A, B5, 3A, B3, 5E, 7A, E0, FE, 2E, 5D, AC, B1, 60, AA, 7D, 87, 6A, 41, 2F, 09]"))
            );
        }

        // Negative - owner (proposer) cannot vote due to 0 weight
        {
            let err = execute(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                ExecuteMsg::Vote {
                    proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        );

        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter1.to_string())
                    .add_attribute("proposal_id", slash_request_id.to_string())
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
                    proposal_id: slash_request_id.clone(),
                    vote: Vote::No,
                },
            )
            .unwrap();
        }

        // check that the proposal status is still open
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
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
                    proposal_id: slash_request_id.clone(),
                    vote: Vote::Abstain,
                },
            );

            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter3.to_string())
                        .add_attribute("proposal_id", slash_request_id.to_string())
                        .add_attribute("status", format!("{:?}", Status::Open))
                        .add_attribute("vote", format!("{:?}", Vote::Abstain))
                ))
            );
        }

        // check that the proposal status is still open
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
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
                    proposal_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            );

            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter4.to_string())
                        .add_attribute("proposal_id", slash_request_id.to_string())
                        .add_attribute("status", format!("{:?}", Status::Passed))
                        .add_attribute("vote", format!("{:?}", Vote::Yes))
                ))
            );
        }

        // check that the proposal status is passed
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Passed);
    }

    #[test]
    fn test_execute_vote_rejected() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

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

        // execute vote No for voter1
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
                vote: Vote::No,
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter3.to_string())
                    .add_attribute("proposal_id", slash_request_id.to_string())
                    .add_attribute("status", format!("{:?}", Status::Rejected))
                    .add_attribute("vote", format!("{:?}", Vote::No))
            ))
        );

        // assert that the proposal status is rejected
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
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
                    proposal_id: slash_request_id.clone(),
                    vote: Vote::Yes,
                },
            );
            assert_eq!(
                res,
                Ok(Response::new().add_event(
                    Event::new("vote")
                        .add_attribute("sender", voter4.to_string())
                        .add_attribute("proposal_id", slash_request_id.to_string())
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

        // execute vote Yes successfully for voter1
        let voter1 = deps.api.addr_make("voter1");
        let voter1_info = message_info(&voter1, &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            voter1_info.clone(),
            ExecuteMsg::Vote {
                proposal_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("vote")
                    .add_attribute("sender", voter1.to_string())
                    .add_attribute("proposal_id", slash_request_id.to_string())
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
                    proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
            },
        );
        assert_eq!(
            res,
            Ok(Response::new().add_event(
                Event::new("close")
                    .add_attribute("sender", info.sender)
                    .add_attribute("proposal_id", slash_request_id.to_string())
            ))
        );

        // check that the proposal status is rejected
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        assert_eq!(prop.status, Status::Rejected);
    }

    #[test]
    fn test_execute_close_on_passed_proposal() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        setup(deps.as_mut(), env.clone(), info.clone()).unwrap();

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
                proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
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
                proposal_id: slash_request_id.clone(),
                vote: Vote::Yes,
            },
        )
        .unwrap();

        // check that the proposal status is passed
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        assert_eq!(prop.current_status(&env.block), Status::Passed);

        // move the env time to expire the proposal
        env.block.time = env.block.time.plus_seconds(100);

        // execute close failed due to proposal already passed despite expiration
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Close {
                proposal_id: slash_request_id.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::WrongCloseStatus {});

        // check that the proposal status is still passed
        let prop = PROPOSALS
            .load(deps.as_ref().storage, slash_request_id.clone())
            .unwrap();
        assert_eq!(prop.status, Status::Passed);
    }
}
