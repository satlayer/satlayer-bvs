use crate::msg::ExecuteMsg;
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BvsDriverContract(pub Addr);

impl BvsDriverContract {
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
    use super::BvsDriverContract;
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

    fn instantiate() -> (App, BvsDriverContract) {
        let mut app = mock_app();

        let contract_id = app.store_code(contract());
        let admin = app.api().addr_make("admin");
        let owner = app.api().addr_make("owner");

        let msg = InstantiateMsg {
            initial_owner: owner.to_string(),
        };
        let contract_addr = app
            .instantiate_contract(contract_id, admin, &msg, &[], "BVS Driver", None)
            .unwrap();

        let contract = BvsDriverContract(contract_addr);
        (app, contract)
    }

    mod tasks {
        use super::*;
        use crate::msg::ExecuteMsg;
        use crate::ContractError;

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
        fn execute_bvs_offchain() {
            let (mut app, contract) = instantiate();

            let bvs_addr = app.api().addr_make("bvs_contract");

            // Register the BVS contract
            {
                let msg = ExecuteMsg::AddRegisteredBvsContract {
                    address: bvs_addr.to_string(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(Addr::unchecked("anyone"), cosmos_msg).unwrap();
            }

            // ExecuteBvsOffchain task
            {
                let msg = ExecuteMsg::ExecuteBvsOffchain {
                    task_id: "123".into(),
                };
                let cosmos_msg = contract.call(msg).unwrap();
                app.execute(bvs_addr, cosmos_msg).unwrap();
            }
        }

        #[test]
        fn transfer_ownership() {
            let (mut app, contract) = instantiate();
            let owner = app.api().addr_make("owner");
            let new_owner = app.api().addr_make("new_owner");

            let msg = ExecuteMsg::TransferOwnership {
                new_owner: new_owner.to_string(),
            };
            let cosmos_msg = contract.call(msg).unwrap();
            app.execute(owner, cosmos_msg).unwrap();
        }

        #[test]
        fn transfer_ownership_not_owner() {
            let (mut app, contract) = instantiate();
            let not_owner = app.api().addr_make("not_owner");
            let new_owner = app.api().addr_make("new_owner");

            let msg = ExecuteMsg::TransferOwnership {
                new_owner: new_owner.to_string(),
            };
            let cosmos_msg = contract.call(msg).unwrap();
            let error = app.execute(not_owner, cosmos_msg).unwrap_err();
            assert_eq!(
                error.root_cause().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }
}
