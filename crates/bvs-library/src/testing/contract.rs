use cosmwasm_std::{to_json_binary, Addr, Coin, Empty, Env, StdResult, Storage, Uint128, WasmMsg};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// TestingContract is a trait that provides a common interface for setting up testing contracts.
pub trait TestingContract<IM, EM, QM, MM = Empty>
where
    IM: serde::Serialize,
    EM: serde::Serialize + Debug,
    QM: serde::Serialize,
    MM: serde::Serialize,
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
        let key = format!("CONTRACT:{label}");
        let value = String::from_utf8(addr.as_bytes().to_vec()).unwrap();
        app.storage_mut().set(key.as_bytes(), value.as_bytes());
    }

    /// Get the contract address in the storage for the given label.
    fn get_contract_addr(app: &App, label: &str) -> Addr {
        let key = format!("CONTRACT:{label}");
        match app.storage().get(key.as_bytes()) {
            Some(value) => Addr::unchecked(String::from_utf8(value).unwrap()),
            None => app.api().addr_make(key.as_str()), // fallback to dummy address
        }
    }

    fn addr(&self) -> &Addr;

    fn execute(&self, app: &mut App, sender: &Addr, msg: &EM) -> AnyResult<AppResponse> {
        self.execute_with_funds(app, sender, msg, vec![])
    }

    fn execute_with_funds(
        &self,
        app: &mut App,
        sender: &Addr,
        msg: &EM,
        funds: Vec<Coin>,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(sender.clone(), self.addr().clone(), msg, &funds)
    }

    fn query<T: DeserializeOwned>(&self, app: &App, msg: &QM) -> StdResult<T> {
        app.wrap().query_wasm_smart(self.addr(), &msg)
    }

    fn migrate(&self, app: &mut App, sender: &Addr, msg: &MM) -> AnyResult<AppResponse> {
        let msg_bin = to_json_binary(&msg).expect("cannot serialize MigrateMsg");
        let code_id = Self::store_code(app);
        let migrate_msg = WasmMsg::Migrate {
            contract_addr: self.addr().to_string(),
            new_code_id: code_id,
            msg: msg_bin,
        };

        app.execute(sender.clone(), migrate_msg.into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Cw20TokenContract {
    pub addr: Addr,
    pub init: cw20_base::msg::InstantiateMsg,
}

impl
    TestingContract<
        cw20_base::msg::InstantiateMsg,
        cw20_base::msg::ExecuteMsg,
        cw20_base::msg::QueryMsg,
    > for Cw20TokenContract
{
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> cw20_base::msg::InstantiateMsg {
        cw20_base::msg::InstantiateMsg {
            symbol: "SATL".to_string(),
            name: "Satlayer Test Token".to_string(),
            decimals: 18,
            initial_balances: vec![Cw20Coin {
                address: app.api().addr_make("owner").to_string(),
                amount: Uint128::new(1_000_000e18 as u128),
            }],
            mint: Some(MinterResponse {
                minter: app.api().addr_make("owner").to_string(),
                cap: Some(Uint128::new(1_000_000_000e18 as u128)), // 1000e18 = 1e21
            }),
            marketing: None,
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<cw20_base::msg::InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "cw20", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}

impl Cw20TokenContract {
    /// For testing with pre-approved spending for x address.
    pub fn increase_allowance(&self, app: &mut App, sender: &Addr, spender: &Addr, amount: u128) {
        let msg = &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
            spender: spender.to_string(),
            amount: Uint128::new(amount),
            expires: None,
        };
        self.execute(app, sender, msg).unwrap();
    }

    /// Fund a recipient with some tokens
    pub fn fund(&self, app: &mut App, recipient: &Addr, amount: u128) {
        let owner = Addr::unchecked(&self.init.initial_balances[0].address);
        let msg = &cw20_base::msg::ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: Uint128::new(amount),
        };
        self.execute(app, &owner, msg).unwrap();
    }

    pub fn transfer(&self, app: &mut App, sender: &Addr, recipient: &Addr, amount: u128) {
        let msg = &cw20_base::msg::ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: Uint128::new(amount),
        };
        self.execute(app, sender, msg).unwrap();
    }

    pub fn balance(&self, app: &App, address: &Addr) -> u128 {
        let query = cw20_base::msg::QueryMsg::Balance {
            address: address.to_string(),
        };
        let res: cw20::BalanceResponse = self.query(app, &query).unwrap();
        res.balance.into()
    }
}
