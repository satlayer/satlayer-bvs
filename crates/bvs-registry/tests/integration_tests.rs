use bvs_library::testing::TestingContract;
use bvs_registry::msg::{
    CanExecuteResponse, ExecuteMsg, InstantiateMsg, IsPausedResponse, QueryMsg,
};
use bvs_registry::testing::RegistryContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::Event;
use cw_multi_test::App;

fn instantiate(msg: Option<InstantiateMsg>) -> (App, RegistryContract) {
    let mut app = App::default();
    let env = mock_env();
    let contract = RegistryContract::new(&mut app, &env, msg);
    (app, contract)
}

#[test]
fn pause_unpause() {
    let (mut app, contract) = instantiate(None);

    {
        let owner = app.api().addr_make("owner");
        let msg = &ExecuteMsg::Pause {};
        let res = contract.execute(&mut app, &owner, &msg).unwrap();

        assert_eq!(res.events.len(), 2);
        assert_eq!(
            res.events[1],
            Event::new("wasm")
                .add_attribute("_contract_address", contract.addr.to_string())
                .add_attribute("method", "pause")
                .add_attribute("sender", app.api().addr_make("owner").to_string())
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.is_paused(), true);

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("caller").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.can_execute(), false);
    }

    {
        let owner = app.api().addr_make("owner");
        let msg = &ExecuteMsg::Unpause {};
        let res = contract.execute(&mut app, &owner, &msg).unwrap();

        assert_eq!(res.events.len(), 2);

        assert_eq!(
            res.events[1],
            Event::new("wasm")
                .add_attribute("_contract_address", contract.addr.to_string())
                .add_attribute("method", "unpause")
                .add_attribute("sender", app.api().addr_make("owner").to_string())
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.is_paused(), false);

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("caller").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.can_execute(), true);
    }
}

#[test]
fn unauthorized_pause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
        initial_paused: false,
    }));

    {
        let sender = app.api().addr_make("random");
        let msg = ExecuteMsg::Pause {};
        let err = contract.execute(&mut app, &sender, &msg).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_registry::ContractError::Unauthorized {}.to_string()
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.is_paused(), false);

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("caller").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.can_execute(), true);
    }
}

#[test]
fn unauthorized_unpause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
        initial_paused: true,
    }));

    {
        let sender = app.api().addr_make("not_authorized");
        let msg = ExecuteMsg::Pause {};
        let err = contract.execute(&mut app, &sender, &msg).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_registry::ContractError::Unauthorized {}.to_string()
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();

        assert_eq!(res.is_paused(), true);

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("caller").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.can_execute(), false);
    }
}
