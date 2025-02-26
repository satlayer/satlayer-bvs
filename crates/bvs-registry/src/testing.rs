use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{to_json_binary, Addr, Empty, WasmMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegistryContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl RegistryContract {
    fn execute(&self, app: &mut App, sender: Addr, msg: ExecuteMsg) {
        let msg = to_json_binary(&msg).unwrap();
        let execute_msg = WasmMsg::Execute {
            contract_addr: self.addr.to_string(),
            msg,
            funds: vec![],
        };

        app.execute(sender, execute_msg.into()).unwrap();
    }

    pub fn pause(&self, app: &mut App) {
        let sender = Addr::unchecked(self.init.owner.clone());
        self.execute(app, sender, ExecuteMsg::Pause {});
    }

    pub fn unpause(&self, app: &mut App) {
        let sender = Addr::unchecked(self.init.owner.clone());
        self.execute(app, sender, ExecuteMsg::Unpause {});
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

impl InstantiateMsg {
    pub fn default(app: &mut App) -> Self {
        Self {
            owner: app.api().addr_make("owner").to_string(),
            initial_paused: false,
        }
    }
}

pub fn instantiate(app: &mut App, code_id: u64, msg: Option<InstantiateMsg>) -> RegistryContract {
    let msg = msg.unwrap_or(InstantiateMsg::default(app));

    let addr = app
        .instantiate_contract(
            code_id,
            app.api().addr_make("sender"),
            &msg,
            &[],
            "BVS Registry",
            None,
        )
        .unwrap();

    RegistryContract { addr, init: msg }
}
