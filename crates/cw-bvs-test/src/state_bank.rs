use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

pub struct StateBank {
    pub addr: Addr,
    pub contract_id: u64,
    pub contract_admin: Addr,
    pub init_msg: state_bank::msg::InstantiateMsg,
}

impl StateBank {
    pub fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            state_bank::contract::execute,
            state_bank::contract::instantiate,
            state_bank::contract::query,
        );
        Box::new(contract)
    }

    pub fn instantiate(app: &mut App) -> StateBank {
        let contract_admin = app.api().addr_make("BVSDriverContract:admin");
        let owner = app.api().addr_make("BVSDriverContract:owner");
        let contract_id = app.store_code(StateBank::contract());

        let init_msg = state_bank::msg::InstantiateMsg {
            initial_owner: owner.to_string(),
        };
        let addr = app
            .instantiate_contract(
                contract_id,
                contract_admin.clone(),
                &init_msg,
                &[],
                "State Bank",
                None,
            )
            .unwrap();

        StateBank {
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
        StateBank::instantiate(&mut app);
    }
}
