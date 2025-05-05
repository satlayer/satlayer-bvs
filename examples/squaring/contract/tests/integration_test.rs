use bvs_multi_test::BvsMultiTest;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_json_binary, Addr, WasmMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use squaring_contract::msg::InstantiateMsg;

fn instantiate() -> (App, Addr) {
    let mut app = App::default();
    let env = mock_env();
    let bvs = BvsMultiTest::new(&mut app, &env);

    let code_id = app.store_code(Box::new(ContractWrapper::new(
        squaring_contract::contract::execute,
        squaring_contract::contract::instantiate,
        squaring_contract::contract::query,
    )));
    let admin = app.api().addr_make("admin");
    let msg = InstantiateMsg {
        registry: bvs.registry.addr.to_string(),
        router: bvs.vault_router.addr.to_string(),
    };

    let contract_addr = app
        .instantiate_contract(code_id, admin, &msg, &[], "Squaring", None)
        .unwrap();
    (app, contract_addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use squaring_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn request() {
        let (mut app, contract_addr) = instantiate();

        let request = ExecuteMsg::Request { input: 3 };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&request).unwrap(),
            funds: vec![],
        };
        let sender = app.api().addr_make("anyone");
        app.execute(sender, cosmos_msg.into()).unwrap();
    }

    #[test]
    fn request_respond() {
        let (mut app, contract_addr) = instantiate();

        let request = ExecuteMsg::Request { input: 2 };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&request).unwrap(),
            funds: vec![],
        };
        let sender = app.api().addr_make("anyone");
        app.execute(sender, cosmos_msg.into()).unwrap();

        let respond = ExecuteMsg::Respond {
            input: 2,
            output: 4,
        };
        let cosmos_msg = WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&respond).unwrap(),
            funds: vec![],
        };
        let operator = app.api().addr_make("operator");
        app.execute(operator.clone(), cosmos_msg.into()).unwrap();

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
}
