use bvs_library::{ownership::OwnershipError, testing::TestingContract};
use bvs_pauser::msg::{
    CanExecuteFlag, CanExecuteResponse, ExecuteMsg, InstantiateMsg, IsPausedResponse, QueryMsg,
};
use bvs_pauser::testing::PauserContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::Event;
use cw_multi_test::App;

fn instantiate(msg: Option<InstantiateMsg>) -> (App, PauserContract) {
    let mut app = App::default();
    let env = mock_env();
    let contract = PauserContract::new(&mut app, &env, msg);
    (app, contract)
}

#[test]
fn pause_unpause() {
    let (mut app, contract) = instantiate(None);

    {
        let owner = app.api().addr_make("owner");
        let downstream_contract = app.api().addr_make("downstream_contract");
        let method = "any";
        let msg = &ExecuteMsg::Pause {
            contract: downstream_contract.to_string(),
            method: method.to_string(),
        };
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
            contract: app.api().addr_make("downstream_contract").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert!(res.is_paused());

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("downstream_contract").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.0, 1);
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::Paused);
    }

    {
        let owner = app.api().addr_make("owner");
        let msg = &ExecuteMsg::Unpause {
            contract: app.api().addr_make("downstream_contract").to_string(),
            method: "any".to_string(),
        };
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
            contract: app.api().addr_make("downstream_contract").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert!(!res.is_paused());

        let msg = QueryMsg::CanExecute {
            contract: app.api().addr_make("downstream_contract").to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: "any".to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.0, 0);
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::CanExecute);
    }
}

#[test]
fn unauthorized_pause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
    }));

    let downstream_contract = app.api().addr_make("downstream_contract");
    let method = "any";
    let sender = app.api().addr_make("random");

    {
        let msg = ExecuteMsg::Pause {
            contract: downstream_contract.to_string(),
            method: method.to_string(),
        };
        let err = contract.execute(&mut app, &sender, &msg).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_pauser::PauserError::Unauthorized {}.to_string()
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: downstream_contract.to_string(),
            method: method.to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();
        assert!(!res.is_paused());

        let msg = QueryMsg::CanExecute {
            contract: downstream_contract.to_string(),
            sender: sender.to_string(),
            method: method.to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.0, 0);
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::CanExecute);
    }
}

#[test]
fn unauthorized_unpause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
    }));

    let downstream_contract = app.api().addr_make("downstream_contract");
    let method = "any";

    {
        let sender = app.api().addr_make("not_authorized");
        let msg = ExecuteMsg::Pause {
            contract: downstream_contract.to_string(),
            method: method.to_string(),
        };
        let err = contract.execute(&mut app, &sender, &msg).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_pauser::PauserError::Unauthorized {}.to_string()
        );
    }

    {
        let msg = QueryMsg::IsPaused {
            contract: downstream_contract.to_string(),
            method: method.to_string(),
        };
        let res: IsPausedResponse = contract.query(&app, &msg).unwrap();

        assert_eq!(res.is_paused(), false);

        let msg = QueryMsg::CanExecute {
            contract: downstream_contract.to_string(),
            sender: app.api().addr_make("sender").to_string(),
            method: method.to_string(),
        };
        let res: CanExecuteResponse = contract.query(&app, &msg).unwrap();
        assert_eq!(res.0, 0);
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::CanExecute);
    }
}

#[test]
fn transfer_ownership() {
    let (mut app, contract) = instantiate(None);

    let new_owner = &app.api().addr_make("new_owner");
    let msg = ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };
    let owner = &app.api().addr_make("owner");
    let res = contract.execute(&mut app, owner, &msg).unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-TransferredOwnership")
            .add_attribute("_contract_address", contract.addr)
            .add_attribute("old_owner", owner.as_str())
            .add_attribute("new_owner", new_owner.as_str())
    );
}

#[test]
fn transfer_ownership_failed() {
    let (mut app, contract) = instantiate(None);

    let new_owner = &app.api().addr_make("new_owner");
    let msg = ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };
    let not_owner = &app.api().addr_make("not_owner");
    let err = contract.execute(&mut app, not_owner, &msg).unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        bvs_pauser::PauserError::Ownership(OwnershipError::Unauthorized).to_string()
    );
}
