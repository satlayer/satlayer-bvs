use bvs_multi_test::{BvsMultiTest, TestingContract};
use bvs_registry::msg::Metadata;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_json_binary, Addr, WasmMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use squaring_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn instantiate() -> (App, BvsMultiTest, Addr) {
    let mut app = App::default();
    let env = mock_env();
    let bvs = BvsMultiTest::new(&mut app, &env);

    let contract = ContractWrapper::new(
        squaring_contract::contract::execute,
        squaring_contract::contract::instantiate,
        squaring_contract::contract::query,
    )
    .with_reply(squaring_contract::contract::reply);

    let code_id = app.store_code(Box::new(contract));

    let admin = app.api().addr_make("admin");
    let owner = app.api().addr_make("owner");
    let msg = InstantiateMsg {
        registry: bvs.registry.addr.to_string(),
        router: bvs.vault_router.addr.to_string(),
        owner: owner.to_string(),
    };

    let contract_addr = app
        .instantiate_contract(code_id, admin, &msg, &[], "Squaring", None)
        .unwrap();
    (app, bvs, contract_addr)
}

#[test]
fn request() {
    let (mut app, _, contract_addr) = instantiate();

    let request = ExecuteMsg::Request { input: 3 };
    let cosmos_msg = WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_json_binary(&request).unwrap(),
        funds: vec![],
    };
    let sender = app.api().addr_make("anyone");
    app.execute(sender, cosmos_msg.into()).unwrap();
}

/// Request and respond without fault
#[test]
fn request_respond() {
    let (mut app, bvs, contract_addr) = instantiate();

    let operator = app.api().addr_make("operator");

    // Set up the operator, service to operator relationship
    {
        bvs.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: None,
                        uri: None,
                    },
                },
            )
            .unwrap();

        bvs.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
                    service: contract_addr.to_string(),
                },
            )
            .unwrap();

        let owner = app.api().addr_make("owner");
        app.execute(
            owner,
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&ExecuteMsg::RegisterOperator {
                    operator: operator.to_string(),
                })
                .unwrap(),
                funds: vec![],
            }
            .into(),
        )
        .unwrap();

        // Forward the block time as the operator and service is registration is checkpoint-ed
        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });
    }

    // Make the request
    {
        let request = ExecuteMsg::Request { input: 2 };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&request).unwrap(),
            funds: vec![],
        };
        let sender = app.api().addr_make("anyone");
        app.execute(sender, cosmos_msg.into()).unwrap();
    }

    // Respond to the request
    {
        let respond = ExecuteMsg::Respond {
            input: 2,
            output: 4,
        };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&respond).unwrap(),
            funds: vec![],
        };
        app.execute(operator.clone(), cosmos_msg.into()).unwrap();
    }

    let get_response = QueryMsg::GetResponse {
        input: 2,
        operator: operator.to_string(),
    };
    let res: i64 = app
        .wrap()
        .query_wasm_smart(contract_addr, &get_response)
        .unwrap();
    assert_eq!(res, 4);
}

/// Request and respond with fault
#[test]
fn slashing_lifecycle() {
    let (mut app, bvs, contract_addr) = instantiate();

    let operator = app.api().addr_make("operator");

    // Set up the operator, service to operator relationship
    {
        bvs.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: None,
                        uri: None,
                    },
                },
            )
            .unwrap();

        bvs.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
                    service: contract_addr.to_string(),
                },
            )
            .unwrap();

        let owner = app.api().addr_make("owner");
        app.execute(
            owner,
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&ExecuteMsg::RegisterOperator {
                    operator: operator.to_string(),
                })
                .unwrap(),
                funds: vec![],
            }
            .into(),
        )
        .unwrap();

        // Forward the block time as the operator and service is registration is checkpoint-ed
        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });
    }

    // Make the request
    {
        let request = ExecuteMsg::Request { input: 10 };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&request).unwrap(),
            funds: vec![],
        };
        let sender = app.api().addr_make("anyone");
        app.execute(sender, cosmos_msg.into()).unwrap();
    }

    // Respond to the request faultily
    {
        let respond = ExecuteMsg::Respond {
            input: 10,
            output: 20,
        };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&respond).unwrap(),
            funds: vec![],
        };
        app.execute(operator.clone(), cosmos_msg.into()).unwrap();
    }

    // Compute on chain and start the slashing process
    {
        let compute = ExecuteMsg::Compute {
            input: 10,
            operator: operator.to_string(),
        };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&compute).unwrap(),
            funds: vec![],
        };
        let sender = app.api().addr_make("anyone");
        app.execute(sender, cosmos_msg.into()).unwrap();
    }

    // TODO(fuxingloh): next lifecycle of slashing
}
