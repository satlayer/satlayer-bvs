#![cfg(not(target_arch = "wasm32"))]

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

///  This is a testing wrapper around the VaultCw20Contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VaultCw20TokenizedContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for VaultCw20TokenizedContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            pauser: Self::get_contract_addr(app, "pauser").to_string(),
            router: Self::get_contract_addr(app, "vault_router").to_string(),
            operator: app.api().addr_make("operator").to_string(),
            cw20_contract: Self::get_contract_addr(app, "cw20").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "vault_cw20_tokenized", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}

impl VaultCw20TokenizedContract {
    /// For testing with pre-approved spending for x address.
    pub fn increase_allowance(&self, app: &mut App, sender: &Addr, spender: &Addr, amount: u128) {
        let msg = &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
            spender: spender.to_string(),
            amount: Uint128::new(amount),
            expires: None,
        }
        .into();
        self.execute(app, sender, msg).unwrap();
    }

    pub fn transfer(&self, app: &mut App, sender: &Addr, recipient: &Addr, amount: u128) {
        let msg = &cw20_base::msg::ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: Uint128::new(amount),
        }
        .into();
        self.execute(app, sender, msg).unwrap();
    }

    pub fn balance(&self, app: &App, address: &Addr) -> u128 {
        let query = cw20_base::msg::QueryMsg::Balance {
            address: address.to_string(),
        }
        .into();
        let res: cw20::BalanceResponse = self.query(app, &query).unwrap();
        res.balance.into()
    }
}
