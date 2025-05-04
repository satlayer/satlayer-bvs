use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use squaring_contract::msg::ExecuteMsg;

pub struct SquaringContract(pub Addr);

impl SquaringContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::SquaringContract;
    use cosmwasm_std::Empty;
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use squaring_contract::msg::InstantiateMsg;

    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            squaring_contract::contract::execute,
            squaring_contract::contract::instantiate,
            squaring_contract::contract::query,
        );
        Box::new(contract)
    }

    fn instantiate() -> (App, SquaringContract) {
        let mut app = App::default();

        let code_id = app.store_code(contract());
        let admin = app.api().addr_make("admin");
        let msg = InstantiateMsg {};
        let contract_addr = app
            .instantiate_contract(code_id, admin, &msg, &[], "Squaring Example", None)
            .unwrap();
        let contract = SquaringContract(contract_addr);
        (app, contract)
    }

    mod tasks {
        use super::*;
        use squaring_contract::msg::{ExecuteMsg, QueryMsg};

        #[test]
        fn request() {
            let (mut app, contract) = instantiate();

            let msg = ExecuteMsg::Request { input: 3 };
            let cosmos_msg = contract.call(msg).unwrap();
            let sender = app.api().addr_make("anyone");
            app.execute(sender, cosmos_msg).unwrap();
        }

        #[test]
        fn request_respond() {
            let (mut app, contract) = instantiate();

            let msg = ExecuteMsg::Request { input: 2 };
            let cosmos_msg = contract.call(msg).unwrap();
            let sender = app.api().addr_make("anyone");
            app.execute(sender, cosmos_msg).unwrap();

            let msg = ExecuteMsg::Respond {
                input: 2,
                output: 4,
            };
            let cosmos_msg = contract.call(msg).unwrap();
            let operator = app.api().addr_make("operator");
            app.execute(operator.clone(), cosmos_msg).unwrap();

            let query_msg = QueryMsg::GetResponse {
                input: 2,
                operator: operator.to_string(),
            };
            let res: i64 = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert_eq!(res, 4);
        }
    }
}
