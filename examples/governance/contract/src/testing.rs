#![cfg(not(target_arch = "wasm32"))]
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Decimal, Empty, Env};
use cw3_fixed_multisig;
use cw3_fixed_multisig::msg::Voter;
use cw_multi_test::{App, Contract, ContractWrapper};
use cw_utils::{Duration, Threshold};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for GovernanceContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        let threshold: Threshold = Threshold::AbsolutePercentage {
            percentage: Decimal::percent(50),
        };
        let voters = vec![
            Voter {
                addr: app.api().addr_make("voter1").to_string(),
                weight: 25,
            },
            Voter {
                addr: app.api().addr_make("voter2").to_string(),
                weight: 25,
            },
            Voter {
                addr: app.api().addr_make("voter3").to_string(),
                weight: 25,
            },
            Voter {
                addr: app.api().addr_make("voter4").to_string(),
                weight: 25,
            },
        ];

        let duration = Duration::Time(6000);

        InstantiateMsg {
            registry: Self::get_contract_addr(app, "registry").to_string(),
            router: Self::get_contract_addr(app, "vault_router").to_string(),
            owner: app.api().addr_make("owner").to_string(),
            cw3_instantiate_msg: cw3_fixed_multisig::msg::InstantiateMsg {
                voters,
                threshold,
                max_voting_period: duration,
            },
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "governance", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
