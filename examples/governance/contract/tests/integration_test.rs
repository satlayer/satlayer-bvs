use bvs_guardrail::testing::GuardrailContract;
use bvs_multi_test::{
    Cw20TokenContract, PauserContract, RegistryContract, TestingContract, VaultBankContract,
    VaultCw20Contract, VaultRouterContract,
};
use bvs_registry::msg::Metadata;
use bvs_vault_base::msg::RecipientAmount;
use bvs_vault_router::msg::SlashingMetadata;
use cosmwasm_std::{
    coin, coins, testing::mock_env, to_json_binary, BalanceResponse, BankQuery, CosmosMsg,
    DenomMetadata, DenomUnit, QueryRequest, Uint128, WasmMsg,
};
use cw_multi_test::{App, Executor};
use governance_contract::testing::GovernanceContract;

struct TestContracts {
    vault_router: VaultRouterContract,
    bank_vault: VaultBankContract,
    cw20_vault: VaultCw20Contract,
    registry: RegistryContract,
    cw20: Cw20TokenContract,
    governance: governance_contract::testing::GovernanceContract,
    guardrail: GuardrailContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let denom_meta = DenomMetadata {
            description: "Test Token".to_string(),
            denom_units: vec![
                DenomUnit {
                    denom: "denom".to_string(),
                    exponent: 0,
                    aliases: vec![],
                },
                DenomUnit {
                    denom: "mdenom".to_string(),
                    exponent: 6,
                    aliases: vec!["microdenom".to_string()],
                },
            ],
            base: "mdenom".to_string(),
            display: "denom".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        };

        let mut app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .set_denom_metadata(storage, "denom".to_string(), denom_meta)
                .unwrap();
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });

        let env = mock_env();

        let _ = PauserContract::new(&mut app, &env, None);
        let guardrail = GuardrailContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let bank_vault = VaultBankContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);
        let cw20_vault = VaultCw20Contract::new(&mut app, &env, None);
        let governance = GovernanceContract::new(&mut app, &env, None);

        (
            app,
            Self {
                vault_router,
                bank_vault,
                cw20_vault,
                registry,
                cw20,
                governance,
                guardrail,
            },
        )
    }
}

#[test]
fn test_simplified_slashing_lifecycle() {
    let (mut app, contracts) = TestContracts::init();
    let committee = [
        app.api().addr_make("voter1"),
        app.api().addr_make("voter2"),
        app.api().addr_make("voter3"),
        app.api().addr_make("voter4"),
    ];

    {
        let owner = app.api().addr_make("owner");

        let msg = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: contracts.bank_vault.addr().to_string(),
            whitelisted: true,
        };

        contracts
            .vault_router
            .execute(&mut app, &owner, msg)
            .unwrap();

        let msg = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: contracts.cw20_vault.addr().to_string(),
            whitelisted: true,
        };

        contracts
            .vault_router
            .execute(&mut app, &owner, msg)
            .unwrap();

        let owner = app.api().addr_make("owner");
        let denom = "denom";

        let staker = app.api().addr_make("staker");
        let msg = bvs_vault_cw20::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(300_u128),
        });
        contracts
            .cw20
            .increase_allowance(&mut app, &staker, contracts.cw20_vault.addr(), 300_u128);
        contracts.cw20.fund(&mut app, &staker, 300_u128);
        contracts
            .cw20_vault
            .execute(&mut app, &staker, &msg)
            .unwrap();

        // Fund the staker with some initial tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to Vault
        let msg = bvs_vault_bank::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(300),
        });
        contracts
            .bank_vault
            .execute_with_funds(&mut app, &staker, &msg, coins(300, denom))
            .unwrap();

        let bank_vault_info = contracts
            .bank_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        let cw20_vault_info = contracts
            .cw20_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();

        assert_eq!(bank_vault_info.total_assets, Uint128::new(300));
        assert_eq!(cw20_vault_info.total_assets, Uint128::new(300));
    }

    let slashing_parameters = bvs_registry::SlashingParameters {
        destination: Some(contracts.governance.addr.clone()),
        max_slashing_bips: 600,
        resolution_window: 60 * 60, // in seconds, 1hr
    };

    let proposal_action = bvs_registry::msg::ExecuteMsg::EnableSlashing {
        slashing_parameters: slashing_parameters.clone(),
    };

    let proposal_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: contracts.registry.addr.to_string(),
        msg: to_json_binary(&proposal_action).unwrap(),
        funds: vec![],
    }
    .into();

    let msg =
        governance_contract::msg::ExecuteMsg::Base(cw3_fixed_multisig::msg::ExecuteMsg::Propose {
            title: "Enable Slashing".to_string(),
            description: "Proposal to enable slashing with specific parameters".to_string(),
            msgs: vec![proposal_msg],
            latest: None,
        });

    let res = contracts
        .governance
        .execute(&mut app, &committee[0], &msg)
        .expect("Failed to execute proposal");

    let proposal_id: u64 = res
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .and_then(|e| e.attributes.iter().find(|a| a.key == "proposal_id"))
        .map(|a| a.value.parse().unwrap())
        .expect("Proposal ID not found in events");

    // vote
    {
        let msg =
            governance_contract::msg::ExecuteMsg::Base(cw3_fixed_multisig::msg::ExecuteMsg::Vote {
                proposal_id,
                vote: cw3::Vote::Yes,
            });

        // except the proposer, all other committee members vote
        for voter in committee.iter().skip(1) {
            contracts
                .governance
                .execute(&mut app, voter, &msg)
                .expect("Failed to vote");
        }
    }

    // execute proposal to enable slashing
    {
        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Execute { proposal_id },
        );

        contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");

        let query_msg = bvs_registry::msg::QueryMsg::SlashingParameters {
            service: contracts.governance.addr.to_string(),
            timestamp: None,
        };

        let res: bvs_registry::msg::SlashingParametersResponse = contracts
            .registry
            .query(&app, &query_msg)
            .expect("Failed to query slashing parameters");

        assert_eq!(
            res.0,
            Some(slashing_parameters),
            "Slashing parameters do not match expected values"
        );
    }

    // setup vaults and operator
    let operator = app.api().addr_make("operator");

    contracts
        .registry
        .execute(
            &mut app,
            &operator,
            &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                metadata: Metadata {
                    name: Some("operator".to_string()),
                    uri: None,
                },
            },
        )
        .expect("failed to register operator");

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: contracts.governance.addr().to_string(),
        };
        contracts
            .registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        contracts
            .registry
            .execute(&mut app, &contracts.governance.addr, msg)
            .expect("failed to register operator to service");
    }

    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // operator opt-in to slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::OperatorOptInToSlashing {
            service: contracts.governance.addr().to_string(),
        };
        contracts
            .registry
            .execute(&mut app, &operator, msg)
            .expect("failed to opt-in to slashing");
    }

    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // propose request slashing by the committee
    let proposed_action = bvs_vault_router::msg::ExecuteMsg::RequestSlashing(
        bvs_vault_router::msg::RequestSlashingPayload {
            bips: 500,
            operator: operator.to_string(),
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "Test slashing".to_string(),
            },
        },
    );

    let proposal_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: contracts.vault_router.addr.to_string(),
        msg: to_json_binary(&proposed_action).unwrap(),
        funds: vec![],
    }
    .into();

    let msg =
        governance_contract::msg::ExecuteMsg::Base(cw3_fixed_multisig::msg::ExecuteMsg::Propose {
            title: "Request Slashing".to_string(),
            description: "Proposal to request slashing for an operator".to_string(),
            msgs: vec![proposal_msg],
            latest: None,
        });

    let res = contracts
        .governance
        .execute(&mut app, &committee[0], &msg)
        .expect("Failed to execute proposal");

    let proposal_id: u64 = res
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .and_then(|e| e.attributes.iter().find(|a| a.key == "proposal_id"))
        .map(|a| a.value.parse().unwrap())
        .expect("Proposal ID not found in events");

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // vote on the proposal
    {
        let msg =
            governance_contract::msg::ExecuteMsg::Base(cw3_fixed_multisig::msg::ExecuteMsg::Vote {
                proposal_id,
                vote: cw3::Vote::Yes,
            });

        // except the proposer, all other committee members vote
        for voter in committee.iter().skip(2) {
            contracts
                .governance
                .execute(&mut app, voter, &msg)
                .expect("Failed to vote");
        }
    }

    // execute the proposal to request slashing
    let slash_id;
    {
        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Execute { proposal_id },
        );

        contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");

        // check if the slashing request was processed
        let query_msg = bvs_vault_router::msg::QueryMsg::SlashingRequestId {
            service: contracts.governance.addr.to_string(),
            operator: operator.to_string(),
        };

        let res: bvs_vault_router::msg::SlashingRequestIdResponse = contracts
            .vault_router
            .query(&app, &query_msg)
            .expect("Failed to query slashing requests");

        slash_id = res.0.unwrap();
    }

    // propose to lock the slash id vote and execute the slash locking
    {
        let proposed_action = bvs_vault_router::msg::ExecuteMsg::LockSlashing(slash_id.clone());

        let proposal_msg: CosmosMsg = WasmMsg::Execute {
            contract_addr: contracts.vault_router.addr.to_string(),
            msg: to_json_binary(&proposed_action).unwrap(),
            funds: vec![],
        }
        .into();

        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Propose {
                title: "Lock Slashing".to_string(),
                description: "Proposal to lock slashing for an operator".to_string(),
                msgs: vec![proposal_msg],
                latest: None,
            },
        );

        let res = contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");

        let proposal_id: u64 = res
            .events
            .iter()
            .find(|e| e.ty == "wasm")
            .and_then(|e| e.attributes.iter().find(|a| a.key == "proposal_id"))
            .map(|a| a.value.parse().unwrap())
            .expect("Proposal ID not found in events");

        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });

        // vote on the proposal
        let msg =
            governance_contract::msg::ExecuteMsg::Base(cw3_fixed_multisig::msg::ExecuteMsg::Vote {
                proposal_id,
                vote: cw3::Vote::Yes,
            });

        // except the proposer, all other committee members vote
        for voter in committee.iter().skip(2) {
            contracts
                .governance
                .execute(&mut app, voter, &msg)
                .expect("Failed to vote");
        }

        app.update_block(|block| {
            block.height += 360;
            block.time = block.time.plus_seconds(3600);
        });

        // execute the proposal to lock slashing
        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Execute { proposal_id },
        );
        contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");

        let router_cw20_balance = contracts.cw20.balance(&app, &contracts.vault_router.addr);

        let query = BankQuery::Balance {
            address: contracts.vault_router.addr().to_string(),
            denom: "denom".to_string(),
        };

        let router_bank_balance: BalanceResponse =
            app.wrap().query(&QueryRequest::Bank(query)).unwrap();

        assert_eq!(
            router_cw20_balance, 15,
            "Router CW20 balance should be 15 after slash locking"
        );
        assert_eq!(
            router_bank_balance.amount,
            coin(15, "denom"),
            "Router bank balance should be 15 after slash locking"
        );
    }

    // slash finalize propose by committee
    {
        let action = bvs_vault_router::msg::ExecuteMsg::FinalizeSlashing(slash_id.clone());

        let proposal_msg: CosmosMsg = WasmMsg::Execute {
            contract_addr: contracts.vault_router.addr.to_string(),
            msg: to_json_binary(&action).unwrap(),
            funds: vec![],
        }
        .into();

        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Propose {
                title: "Finalize Slashing".to_string(),
                description: "Proposal to finalize slashing for an operator".to_string(),
                msgs: vec![proposal_msg],
                latest: None,
            },
        );

        let res = contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");

        let finalize_slash_proposal_id: u64 = res
            .events
            .iter()
            .find(|e| e.ty == "wasm")
            .and_then(|e| e.attributes.iter().find(|a| a.key == "proposal_id"))
            .map(|a| a.value.parse().unwrap())
            .expect("Proposal ID not found in events");

        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });

        // vote on the proposal by the governance committee
        for voter in committee.iter().skip(2) {
            let msg = governance_contract::msg::ExecuteMsg::Base(
                cw3_fixed_multisig::msg::ExecuteMsg::Vote {
                    proposal_id: finalize_slash_proposal_id,
                    vote: cw3::Vote::Yes,
                },
            );

            contracts
                .governance
                .execute(&mut app, voter, &msg)
                .expect("Failed to vote");
        }

        // satlayer guardrail will need to approve this too
        let guardrail_proposal = bvs_guardrail::msg::ExecuteMsg::Propose {
            slashing_request_id: slash_id.clone(),
            reason: "Finalize slashing".to_string(),
        };

        // guardrail member
        let guardrail_member = app.api().addr_make("owner");

        contracts
            .guardrail
            .execute(&mut app, &guardrail_member, &guardrail_proposal)
            .expect("Failed to propose guardrail approval");

        // now the bvs governance can go ahead and finalize the slashing
        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Execute {
                proposal_id: finalize_slash_proposal_id,
            },
        );

        contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");
    }

    // check slashed collateral arrived at destination
    {
        let bank_balance = app
            .wrap()
            .query_balance(contracts.governance.addr(), "denom")
            .expect("Failed to query bank balance");
        let cw20_balance = contracts.cw20.balance(&app, &contracts.governance.addr);
        assert_eq!(
            bank_balance,
            coin(15, "denom"),
            "Bank balance should be 15 after slashing"
        );
        assert_eq!(cw20_balance, 15, "CW20 balance should be 15 after slashing");
    }
}
