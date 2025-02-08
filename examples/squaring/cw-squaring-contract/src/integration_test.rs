use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use crate::msg::ExecuteMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
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
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const ADMIN: &str = "ADMIN";
    const AGGREGATOR: &str = "AGGREGATOR";
    const STATE_BANK: &str = "STATE_BANK";
    const BVS_DRIVER: &str = "BVS_DRIVER";

    const NATIVE_DENOM: &str = "tBABY";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(ADMIN),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100),
                    }],
                )
                .unwrap();
        })
    }

    fn instantiate() -> (App, SquaringContract) {
        let mut app = mock_app();
        let contract_id = app.store_code(contract());

        let admin = app.api().addr_make(ADMIN);
        assert_eq!(
            app.wrap().query_balance(&admin, NATIVE_DENOM).unwrap().amount,
            Uint128::new(100)
        );

        let msg = InstantiateMsg {
            aggregator: app.api().addr_make(AGGREGATOR),
            state_bank: app.api().addr_make(STATE_BANK),
            bvs_driver: app.api().addr_make(BVS_DRIVER),
        };
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                admin,
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let contract = SquaringContract(contract_addr);
        (app, contract)
    }

    mod tasks {
        use super::*;
        use crate::msg::ExecuteMsg;

        #[test]
        fn create_new_task() {
            let (mut app, contract) = instantiate();

            let msg = ExecuteMsg::CreateNewTask {
                input: 3,
            };
            // TODO(fuxingloh): this test will fail because we don't have aggregator, state_bank, bvs_driver contracts
            //  to call
            let cosmos_msg = contract.call(msg).unwrap();
            app.execute(Addr::unchecked("anyone"), cosmos_msg).unwrap();
        }
    }
}
