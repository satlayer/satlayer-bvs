use cosmwasm_std::{to_json_binary, Addr, Empty, Env, StdResult, Storage, WasmMsg};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, Contract, Executor};
use serde::de::DeserializeOwned;

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

    fn instantiate(app: &mut App, code_id: u64, label: &str, msg: &IM) -> Addr {
        let admin = app.api().addr_make("admin");
        let addr = app
            .instantiate_contract(
                code_id,
                app.api().addr_make("sender"),
                msg,
                &[],
                label,
                Some(admin.to_string()),
            )
            .unwrap();
        Self::set_contract_addr(app, label, &addr);
        addr
    }

    /// Set the contract address in the storage for the given label.
    /// Using the storage system for easy orchestration of contract addresses for testing.
    fn set_contract_addr(app: &mut App, label: &str, addr: &Addr) {
        let key = format!("CONTRACT:{}", label);
        let value = String::from_utf8(addr.as_bytes().to_vec()).unwrap();
        app.storage_mut().set(key.as_bytes(), value.as_bytes());
    }

    /// Get the contract address in the storage for the given label.
    fn get_contract_addr(app: &App, label: &str) -> Addr {
        let key = format!("CONTRACT:{}", label);
        let value = app.storage().get(key.as_bytes()).unwrap();
        Addr::unchecked(String::from_utf8(value).unwrap())
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

    fn query<T: DeserializeOwned>(&self, app: &App, msg: &QM) -> StdResult<T> {
        app.wrap().query_wasm_smart(self.addr(), &msg)
    }

    // TODO: fn migrate
}
