use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg};

struct StateBankContract(pub Addr);

impl StateBankContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<bvs_state_bank::msg::ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
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
    use super::StateBankContract;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_state_bank::contract::execute,
            bvs_state_bank::contract::instantiate,
            bvs_state_bank::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make("admin"),
                    vec![Coin {
                        denom: "SAT".to_string(),
                        amount: Uint128::new(100),
                    }],
                )
                .unwrap();
        })
    }

    fn instantiate() -> (App, StateBankContract) {
        let mut app = mock_app();

        let contract_id = app.store_code(contract());
        let admin = app.api().addr_make("admin");
        let owner = app.api().addr_make("owner");

        let msg = bvs_state_bank::msg::InstantiateMsg {
            initial_owner: owner.to_string(),
        };
        let contract_addr = app
            .instantiate_contract(contract_id, admin, &msg, &[], "State Bank", None)
            .unwrap();

        let contract = StateBankContract(contract_addr);
        (app, contract)
    }

    mod tasks {
        use super::*;
        use bvs_state_bank::msg::ExecuteMsg;
        use bvs_state_bank::query::ValueResponse;
        use cosmwasm_std::{Addr, Event};

        #[test]
        fn execute_set() {
            let (mut app, contract) = instantiate();

            let state_bank_address = app.api().addr_make("state_bank");

            {
                // Register BVS contract into State Bank
                let msg = ExecuteMsg::AddRegisteredBvsContract {
                    address: state_bank_address.to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(Addr::unchecked("anyone"), cosmos_msg).unwrap();
            }
            {
                // Execute set to update state
                let msg = ExecuteMsg::Set {
                    key: "weather".to_string(),
                    value: "winter".to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                let res = app.execute(state_bank_address.clone(), cosmos_msg).unwrap();

                // expect 2 events, 1 for execute and 1 for update state
                assert_eq!(res.events.len(), 2, "expected 2 events");

                assert_eq!(
                    res.events[1],
                    Event::new("wasm-UpdateState")
                        .add_attribute("_contract_address", contract.addr())
                        .add_attribute("sender", state_bank_address.to_string())
                        .add_attribute("key", "weather")
                        .add_attribute("value", "winter"),
                    "expected UpdateState event and 4 attributes"
                );

                // assert the state is updated with the key-value pair
                let query_msg = bvs_state_bank::msg::QueryMsg::Get {
                    key: "weather".to_string(),
                };
                let res: ValueResponse = app
                    .wrap()
                    .query_wasm_smart(contract.addr(), &query_msg)
                    .unwrap();

                assert_eq!(res.value, "winter".to_string());
            }
            {
                // Execute set to update state with a different key-value pair
                let msg = ExecuteMsg::Set {
                    key: "temperature".to_string(),
                    value: "cold".to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                let res = app.execute(state_bank_address.clone(), cosmos_msg).unwrap();

                // expect 2 events, 1 for execute and 1 for update state
                assert_eq!(res.events.len(), 2, "expected 2 events");

                assert_eq!(
                    res.events[1],
                    Event::new("wasm-UpdateState")
                        .add_attribute("_contract_address", contract.addr())
                        .add_attribute("sender", state_bank_address.to_string())
                        .add_attribute("key", "temperature")
                        .add_attribute("value", "cold"),
                    "expected UpdateState event and 4 attributes"
                );

                // assert the state is updated with the key-value pair
                let query_msg = bvs_state_bank::msg::QueryMsg::Get {
                    key: "temperature".to_string(),
                };
                let res: ValueResponse = app
                    .wrap()
                    .query_wasm_smart(contract.addr(), &query_msg)
                    .unwrap();

                assert_eq!(res.value, "cold".to_string());

                // assert the previous state is preserved
                let query_msg = bvs_state_bank::msg::QueryMsg::Get {
                    key: "weather".to_string(),
                };
                let res: ValueResponse = app
                    .wrap()
                    .query_wasm_smart(contract.addr(), &query_msg)
                    .unwrap();

                assert_eq!(res.value, "winter".to_string());
            }
        }

        #[test]
        fn execute_set_update_same_key() {
            let (mut app, contract) = instantiate();

            let state_bank_address = app.api().addr_make("state_bank");

            {
                // Register BVS contract into State Bank
                let msg = ExecuteMsg::AddRegisteredBvsContract {
                    address: state_bank_address.to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(Addr::unchecked("anyone"), cosmos_msg).unwrap();
            }
            {
                // Execute set to update state
                let msg = ExecuteMsg::Set {
                    key: "weather".to_string(),
                    value: "winter".to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(state_bank_address.clone(), cosmos_msg).unwrap();

                // assert the state is updated with the key-value pair
                let query_msg = bvs_state_bank::msg::QueryMsg::Get {
                    key: "weather".to_string(),
                };
                let res: ValueResponse = app
                    .wrap()
                    .query_wasm_smart(contract.addr(), &query_msg)
                    .unwrap();

                assert_eq!(res.value, "winter".to_string());
            }
            {
                // Execute set to update state with the same key
                let msg = ExecuteMsg::Set {
                    key: "weather".to_string(),
                    value: "fall".to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(state_bank_address.clone(), cosmos_msg).unwrap();

                // assert the state is updated with the key-value pair
                let query_msg = bvs_state_bank::msg::QueryMsg::Get {
                    key: "weather".to_string(),
                };
                let res: ValueResponse = app
                    .wrap()
                    .query_wasm_smart(contract.addr(), &query_msg)
                    .unwrap();

                assert_eq!(res.value, "fall".to_string());
            }
        }

        #[test]
        fn add_registered_bvs_contract() {
            let (mut app, contract) = instantiate();

            let bvs_addr = app.api().addr_make("bvs_contract");
            let msg = ExecuteMsg::AddRegisteredBvsContract {
                address: bvs_addr.to_string(),
            };
            let cosmos_msg = contract.call(msg).unwrap();
            app.execute(Addr::unchecked("anyone"), cosmos_msg).unwrap();
        }

        #[test]
        fn transfer_ownership() {
            let (mut app, contract) = instantiate();
            let owner = app.api().addr_make("owner");
            let new_owner = app.api().addr_make("new_owner");

            let msg = ExecuteMsg::TwoStepTransferOwnership {
                new_owner: new_owner.to_string(),
            };
            let cosmos_msg = contract.call(msg).unwrap();
            app.execute(owner, cosmos_msg).unwrap();
        }

        #[test]
        fn transfer_ownership_not_owner() {
            let (mut app, contract) = instantiate();
            let owner = app.api().addr_make("fake_owner");
            let new_owner = app.api().addr_make("new_owner");

            let msg = ExecuteMsg::TwoStepTransferOwnership {
                new_owner: new_owner.to_string(),
            };
            let cosmos_msg = contract.call(msg).unwrap();
            let error = app.execute(owner, cosmos_msg).unwrap_err();
            assert_eq!(
                error.root_cause().to_string(),
                bvs_state_bank::ContractError::Unauthorized {}.to_string()
            );
        }
    }
}
