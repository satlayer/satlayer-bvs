use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_json_binary, Addr, WasmMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
struct BvsRegistryContract(pub Addr);

impl BvsRegistryContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bvs_registry::msg::{ExecuteMsg, InstantiateMsg, IsPausedResponse, QueryMsg};
    use cosmwasm_std::{Empty, Event};
    use cw_multi_test::{App, BasicApp, Contract, ContractWrapper, Executor};

    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_registry::contract::execute,
            bvs_registry::contract::instantiate,
            bvs_registry::contract::query,
        );
        Box::new(contract)
    }

    fn instantiate(msg: Option<InstantiateMsg>) -> (App, BvsRegistryContract) {
        let mut app = BasicApp::default();
        let code_id = app.store_code(contract());

        let sender = app.api().addr_make("sender");
        let owner = app.api().addr_make("owner");
        let msg = msg.unwrap_or(InstantiateMsg {
            owner: owner.to_string(),
            initial_paused: false,
        });
        let registry_contract_addr = app
            .instantiate_contract(code_id, sender, &msg, &[], "test", None)
            .unwrap();

        let cw_template_contract = BvsRegistryContract(registry_contract_addr);

        (app, cw_template_contract)
    }

    #[test]
    fn pause_unpause() {
        let (mut app, contract) = instantiate(None);

        {
            let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
            let execute_msg = WasmMsg::Execute {
                contract_addr: contract.addr().to_string(),
                msg,
                funds: vec![],
            };

            let owner = app.api().addr_make("owner");
            let res = app.execute(owner, execute_msg.into()).unwrap();

            assert_eq!(res.events.len(), 2);

            assert_eq!(
                res.events[1],
                Event::new("wasm")
                    .add_attribute("_contract_address", contract.addr())
                    .add_attribute("method", "pause")
                    .add_attribute("sender", app.api().addr_make("owner").to_string())
            );
        }

        {
            let query_msg = QueryMsg::IsPaused {
                sender: app.api().addr_make("caller").to_string(),
            };
            let res: IsPausedResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();

            assert_eq!(res.paused, true);
        }

        {
            let msg = to_json_binary(&ExecuteMsg::Unpause {}).unwrap();
            let execute_msg = WasmMsg::Execute {
                contract_addr: contract.addr().to_string(),
                msg,
                funds: vec![],
            };

            let owner = app.api().addr_make("owner");
            let res = app.execute(owner, execute_msg.into()).unwrap();

            assert_eq!(res.events.len(), 2);

            assert_eq!(
                res.events[1],
                Event::new("wasm")
                    .add_attribute("_contract_address", contract.addr())
                    .add_attribute("method", "unpause")
                    .add_attribute("sender", app.api().addr_make("owner").to_string())
            );
        }

        {
            let query_msg = QueryMsg::IsPaused {
                sender: app.api().addr_make("caller").to_string(),
            };
            let res: IsPausedResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();

            assert_eq!(res.paused, false);
        }
    }

    #[test]
    fn unauthorized_pause() {
        let (mut app, contract) = instantiate(Some(InstantiateMsg {
            owner: BasicApp::default().api().addr_make("owner").to_string(),
            initial_paused: false,
        }));

        {
            let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
            let execute_msg = WasmMsg::Execute {
                contract_addr: contract.addr().to_string(),
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
            sender: app.api().addr_make("caller").to_string(),
        };
        let res: IsPausedResponse = app
            .wrap()
            .query_wasm_smart(contract.addr(), &query_msg)
            .unwrap();

        assert_eq!(res.paused, false);
    }

    #[test]
    fn unauthorized_unpause() {
        let (mut app, contract) = instantiate(Some(InstantiateMsg {
            owner: BasicApp::default().api().addr_make("owner").to_string(),
            initial_paused: true,
        }));

        {
            let msg = to_json_binary(&ExecuteMsg::Pause {}).unwrap();
            let execute_msg = WasmMsg::Execute {
                contract_addr: contract.addr().to_string(),
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
            sender: app.api().addr_make("caller").to_string(),
        };
        let res: IsPausedResponse = app
            .wrap()
            .query_wasm_smart(contract.addr(), &query_msg)
            .unwrap();

        assert_eq!(res.paused, true);
    }
}
