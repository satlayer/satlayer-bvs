use bvs_guardrail::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_guardrail::testing::GuardrailContract;
use bvs_library::slashing::SlashingRequestId;
use bvs_library::testing::TestingContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Decimal, Event};
use cw3::{ProposalResponse, Status, Vote};
use cw4::Member;
use cw_multi_test::App;
use cw_utils::Threshold;

struct TestContracts {
    guardrail: GuardrailContract,
}

fn instantiate() -> (App, TestContracts) {
    let mut app = App::default();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");

    // instantiate 2-of-4 multisig voting
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
            percentage: Decimal::percent(50),
        },
        default_expiration: 100,
    };

    let guardrail = GuardrailContract::new(&mut app, &env, instantiate_msg.into());

    (app, TestContracts { guardrail })
}

#[test]
fn propose_and_vote_successfully() {
    let (mut app, tc) = instantiate();

    // mock the block
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    let owner = app.api().addr_make("owner");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");

    let slashing_request_id = SlashingRequestId::from_hex(
        "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
    )
    .unwrap();

    // Create a proposal
    let proposal_msg = ExecuteMsg::Propose {
        slashing_request_id: slashing_request_id.clone(),
        reason: "test slashing".to_string(),
    };

    let res = tc
        .guardrail
        .execute(&mut app, &owner, &proposal_msg)
        .unwrap();

    assert_eq!(
        res.events[1],
        Event::new("wasm-propose")
            .add_attribute("_contract_address", tc.guardrail.addr.clone())
            .add_attribute("sender", owner)
            .add_attribute("proposal_id", "1")
            .add_attribute("slashing_request_id", slashing_request_id.to_string())
            .add_attribute("status", format!("{:?}", Status::Open))
    );

    // Voter1 votes "yes"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::Yes,
    };
    let res = tc.guardrail.execute(&mut app, &voter1, &vote_msg).unwrap();
    assert_eq!(
        res.events[1],
        Event::new("wasm-vote")
            .add_attribute("_contract_address", tc.guardrail.addr.clone())
            .add_attribute("sender", voter1)
            .add_attribute("proposal_id", "1")
            .add_attribute("slashing_request_id", slashing_request_id.to_string())
            .add_attribute("status", format!("{:?}", Status::Open))
            .add_attribute("vote", format!("{:?}", Vote::Yes))
    );

    // Voter2 votes "no"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter2, &vote_msg).unwrap();

    // Voter3 abstains
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::Abstain,
    };
    tc.guardrail.execute(&mut app, &voter3, &vote_msg).unwrap();

    // check the proposal status is still Open
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Open);

    // Voter4 votes "yes" to pass the proposal
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::Yes,
    };
    tc.guardrail.execute(&mut app, &voter4, &vote_msg).unwrap();

    // check the proposal status is Passed
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Passed);
}

#[test]
fn propose_and_vote_rejected() {
    let (mut app, tc) = instantiate();

    // mock the block
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    let owner = app.api().addr_make("owner");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");

    let slashing_request_id = SlashingRequestId::from_hex(
        "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
    )
    .unwrap();

    // Create a proposal
    let proposal_msg = ExecuteMsg::Propose {
        slashing_request_id: slashing_request_id.clone(),
        reason: "test slashing".to_string(),
    };
    tc.guardrail
        .execute(&mut app, &owner, &proposal_msg)
        .unwrap();

    // Voter1 votes "no"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter1, &vote_msg).unwrap();

    // Voter2 votes "no"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter2, &vote_msg).unwrap();

    // Voter3 votes "yes"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::Yes,
    };
    tc.guardrail.execute(&mut app, &voter3, &vote_msg).unwrap();

    // check the proposal status is still Open
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Open);

    // Voter4 votes "no" to reject the proposal
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter4, &vote_msg).unwrap();

    // check the proposal status is Rejected
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Rejected);
}

#[test]
fn propose_and_vote_expired() {
    let (mut app, tc) = instantiate();

    // mock the block
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    let owner = app.api().addr_make("owner");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");

    let slashing_request_id = SlashingRequestId::from_hex(
        "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb",
    )
    .unwrap();

    // Create a proposal
    let proposal_msg = ExecuteMsg::Propose {
        slashing_request_id: slashing_request_id.clone(),
        reason: "test slashing".to_string(),
    };
    tc.guardrail
        .execute(&mut app, &owner, &proposal_msg)
        .unwrap();

    // Voter1 votes "no"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter1, &vote_msg).unwrap();

    // Voter2 votes "no"
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::No,
    };
    tc.guardrail.execute(&mut app, &voter2, &vote_msg).unwrap();

    // move blockchain to past expiry
    app.update_block(|block| {
        block.height += 100;
        block.time = block.time.plus_seconds(1000);
    });

    // Voter3 votes "yes" - error due to expiry
    let vote_msg = ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: Vote::Yes,
    };
    let err = tc
        .guardrail
        .execute(&mut app, &voter3, &vote_msg)
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Proposal voting period has expired"
    );

    // check the proposal status is Rejected
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Rejected);

    // we can call the "close" execute msg to move the status in the store to Rejected.
    // the actual status in the store is still Open, but we can close it to Rejected.
    let close_msg = ExecuteMsg::Close {
        slashing_request_id: slashing_request_id.clone(),
    };
    tc.guardrail.execute(&mut app, &owner, &close_msg).unwrap();

    // check the proposal status is Rejected
    let proposal_status = QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, Status::Rejected);
}
