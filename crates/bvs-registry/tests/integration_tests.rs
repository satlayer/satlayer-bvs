use bvs_registry::msg::{ExecuteMsg, InstantiateMsg, IsPausedResponse, QueryMsg};
use bvs_registry::testing;
use bvs_registry::testing::RegistryContract;
use cosmwasm_std::Event;
use cosmwasm_std::{to_json_binary, WasmMsg};
use cw_multi_test::{App, Executor};

fn instantiate(msg: Option<InstantiateMsg>) -> (App, RegistryContract) {
    let mut app = App::default();
    let code_id = app.store_code(testing::contract());
    let contract = testing::instantiate(&mut app, code_id, msg);
    (app, contract)
}

#[test]
fn pause_unpause() {
    let (mut app, contract) = instantiate(None);

    {
        let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
        let execute_msg = WasmMsg::Execute {
            contract_addr: contract.addr.to_string(),
            msg,
            funds: vec![],
        };

        let owner = app.api().addr_make("owner");
        let res = app.execute(owner, execute_msg.into()).unwrap();

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
        let query_msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = app
            .wrap()
            .query_wasm_smart(&contract.addr, &query_msg)
            .unwrap();

        assert_eq!(res.paused, true);
    }

    {
        let msg = to_json_binary(&ExecuteMsg::Unpause {}).unwrap();
        let execute_msg = WasmMsg::Execute {
            contract_addr: contract.addr.to_string(),
            msg,
            funds: vec![],
        };

        let owner = app.api().addr_make("owner");
        let res = app.execute(owner, execute_msg.into()).unwrap();

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
        let query_msg = QueryMsg::IsPaused {
            contract: app.api().addr_make("caller").to_string(),
            method: "any".to_string(),
        };
        let res: IsPausedResponse = app
            .wrap()
            .query_wasm_smart(&contract.addr, &query_msg)
            .unwrap();

        assert_eq!(res.paused, false);
    }
}

#[test]
fn unauthorized_pause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
        initial_paused: false,
    }));

    {
        let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
        let execute_msg = WasmMsg::Execute {
            contract_addr: contract.addr.to_string(),
            msg,
            funds: vec![],
        };

        let sender = app.api().addr_make("random");
        let err = app.execute(sender, execute_msg.into()).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_registry::ContractError::Unauthorized {}.to_string()
        );
    }

    let query_msg = QueryMsg::IsPaused {
        contract: app.api().addr_make("caller").to_string(),
        method: "any".to_string(),
    };
    let res: IsPausedResponse = app
        .wrap()
        .query_wasm_smart(&contract.addr, &query_msg)
        .unwrap();

    assert_eq!(res.paused, false);
}

#[test]
fn unauthorized_unpause() {
    let (mut app, contract) = instantiate(Some(InstantiateMsg {
        owner: App::default().api().addr_make("owner").to_string(),
        initial_paused: true,
    }));

    {
        let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
        let execute_msg = WasmMsg::Execute {
            contract_addr: contract.addr.to_string(),
            msg,
            funds: vec![],
        };

        let sender = app.api().addr_make("not_authorized");
        let err = app.execute(sender, execute_msg.into()).unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            bvs_registry::ContractError::Unauthorized {}.to_string()
        );
    }

    let query_msg = QueryMsg::IsPaused {
        contract: app.api().addr_make("caller").to_string(),
        method: "any".to_string(),
    };
    let res: IsPausedResponse = app
        .wrap()
        .query_wasm_smart(&contract.addr, &query_msg)
        .unwrap();

    assert_eq!(res.paused, true);
}
