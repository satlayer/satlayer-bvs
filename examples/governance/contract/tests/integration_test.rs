use bvs_multi_test::{
    Cw20TokenContract, PauserContract, RegistryContract, TestingContract, VaultBankContract,
    VaultCw20Contract, VaultRouterContract,
};
use cosmwasm_std::{coins, testing::mock_env, to_json_binary, CosmosMsg, Uint128, WasmMsg};
use cw_multi_test::App;
use governance_contract::testing::GovernanceContract;

struct TestContracts {
    vault_router: VaultRouterContract,
    bank_vault: VaultBankContract,
    cw20_vault: VaultCw20Contract,
    registry: RegistryContract,
    cw20: Cw20TokenContract,
    governance: governance_contract::testing::GovernanceContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });
        let env = mock_env();

        let _ = PauserContract::new(&mut app, &env, None);
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
            },
        )
    }
}

#[test]
fn test_enable_slashig() {
    let (mut app, contracts) = TestContracts::init();
    let committee = vec![
        app.api().addr_make("voter1"),
        app.api().addr_make("voter2"),
        app.api().addr_make("voter3"),
        app.api().addr_make("voter4"),
    ];

    let slashing_parameters = bvs_registry::SlashingParameters {
        destination: Some(contracts.governance.addr.clone()),
        max_slashing_bips: 100,
        resolution_window: 60 * 60, // in seconds, 1hr
    };

    let proposal_action = governance_contract::msg::ExecuteMsg::Extended(
        governance_contract::msg::ExtendedExecuteMsg::EnableSlashing(slashing_parameters),
    );

    let proposal_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: contracts.governance.addr.to_string(),
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

    // execute proposal
    {
        let msg = governance_contract::msg::ExecuteMsg::Base(
            cw3_fixed_multisig::msg::ExecuteMsg::Execute { proposal_id },
        );

        contracts
            .governance
            .execute(&mut app, &committee[0], &msg)
            .expect("Failed to execute proposal");
    }
}
