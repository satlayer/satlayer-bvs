use cosmwasm_std::{to_json_binary, Addr, Empty, Env, WasmMsg};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, Contract, Executor};

/// TestingContract is a trait that provides a common interface for setting up testing contracts.
pub trait TestingContract<IM, EM, QM>
where
    IM: serde::Serialize,
    EM: serde::Serialize,
    QM: serde::Serialize,
{
    fn wrapper() -> Box<dyn Contract<Empty>>;

    fn default_init(app: &mut App, env: &Env) -> IM;

    fn new(app: &mut App, env: &Env, msg: Option<IM>) -> Self;

    fn store_code(app: &mut App) -> u64 {
        app.store_code(Self::wrapper())
    }

    fn instantiate(app: &mut App, code_id: u64, msg: &IM) -> Addr {
        let addr = app
            .instantiate_contract(
                code_id,
                app.api().addr_make("sender"),
                msg,
                &[],
                "BVS Contract Initialize",
                Some(app.api().addr_make("admin").to_string()),
            )
            .unwrap();

        addr
    }

    fn addr(&self) -> &Addr;

    fn execute(&self, app: &mut App, sender: &Addr, msg: &EM) -> AnyResult<AppResponse> {
        let msg_bin = to_json_binary(&msg).expect("cannot serialize ExecuteMsg");
        let execute_msg = WasmMsg::Execute {
            contract_addr: self.addr().to_string(),
            msg: msg_bin,
            funds: vec![],
        };

        app.execute(sender.clone(), execute_msg.into())
    }

    // TODO: fn query
    // TODO: fn migrate
}
