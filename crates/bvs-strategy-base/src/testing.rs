use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env};
use cw20_base;
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StrategyBaseContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for StrategyBaseContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            strategy_manager: app.api().addr_make("strategy_manager").to_string(),
            underlying_token: app.api().addr_make("SAT").to_string(),
            registry: app.api().addr_make("registry").to_string(),
            owner: app.api().addr_make("strategy_owner").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "registry", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
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
            marketing: None,
            symbol: "LOTR".to_string(),
            name: "Lord_Of_The_Ring".to_string(),
            decimals: 6,
            initial_balances: vec![cw20::Cw20Coin {
                address: app.api().addr_make("some_dude").to_string(),
                amount: cosmwasm_std::Uint128::new(1000000),
            }],
            mint: None,
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<cw20_base::msg::InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "dummy_cw20_token", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
