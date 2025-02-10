use crate::tester::BVSApp;
use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};
use cw_multi_test::error::{AnyResult, Error};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

pub struct BVSDriver {
    pub addr: Addr,
    pub contract_id: u64,
    pub contract_admin: Addr,
    pub init_msg: bvs_driver::msg::InstantiateMsg,
}

impl BVSDriver {
    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_driver::contract::execute,
            bvs_driver::contract::instantiate,
            bvs_driver::contract::query,
        );
        Box::new(contract)
    }

    pub fn instantiate(app: &mut App) -> BVSDriver {
        let contract_admin = app.api().addr_make("BVSDriverContract:admin");
        let owner = app.api().addr_make("BVSDriverContract:owner");
        let contract_id = app.store_code(BVSDriver::contract());

        let init_msg = bvs_driver::msg::InstantiateMsg {
            initial_owner: owner.to_string(),
        };
        let addr = app
            .instantiate_contract(
                contract_id,
                contract_admin.clone(),
                &init_msg,
                &[],
                "BVS Driver",
                None,
            )
            .unwrap();

        BVSDriver {
            addr,
            contract_id,
            contract_admin,
            init_msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockApi;

    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "tSATLAYER";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(ADMIN),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100),
                    }],
                )
                .unwrap();
        })
    }

    #[test]
    fn instantiate() {
        let mut app = mock_app();
        BVSDriver::instantiate(&mut app);
    }
}
