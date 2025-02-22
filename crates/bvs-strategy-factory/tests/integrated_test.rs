use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
struct StrategyFactoryContract(pub Addr);

impl StrategyFactoryContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<bvs_strategy_factory::msg::ExecuteMsg>>(
        &self,
        msg: T,
    ) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
struct StrategyManagerContract(pub Addr);

impl StrategyManagerContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<bvs_strategy_manager::msg::ExecuteMsg>>(
        &self,
        msg: T,
    ) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::{StrategyFactoryContract, StrategyManagerContract};
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn factory_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_strategy_factory::contract::execute,
            bvs_strategy_factory::contract::instantiate,
            bvs_strategy_factory::contract::query,
        )
        .with_reply(bvs_strategy_factory::contract::reply);
        Box::new(contract)
    }

    pub fn manager_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_strategy_manager::contract::execute,
            bvs_strategy_manager::contract::instantiate,
            bvs_strategy_manager::contract::query,
        );
        Box::new(contract)
    }

    pub fn base_strategy_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            bvs_strategy_base::contract::execute,
            bvs_strategy_base::contract::instantiate,
            bvs_strategy_base::contract::query,
        );
        Box::new(contract)
    }

    pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make("admin"),
                    vec![Coin {
                        denom: "SAT".to_string(),
                        amount: Uint128::new(1000),
                    }],
                )
                .unwrap();
        })
    }

    fn upload_instantiate_factory(
        app: &mut App,
        manager: Addr,
        strategy_id: u64,
    ) -> (StrategyFactoryContract, u64) {
        let contract_id = app.store_code(factory_contract());
        let owner = app.api().addr_make("owner");
        let admin = app.api().addr_make("admin");

        let msg = bvs_strategy_factory::msg::InstantiateMsg {
            initial_owner: owner.to_string(),
            strategy_code_id: strategy_id,
            strategy_manager: manager.to_string(),
            pauser: owner.to_string(),
            unpauser: owner.to_string(),
            initial_paused_status: 0,
        };
        let contract_addr = app
            .instantiate_contract(contract_id, admin, &msg, &[], "BVS Strategy Factory", None)
            .unwrap();

        let contract = StrategyFactoryContract(contract_addr);
        (contract, contract_id)
    }

    fn upload_instantiate_manager(app: &mut App) -> (StrategyManagerContract, u64) {
        let contract_id = app.store_code(manager_contract());
        let owner = app.api().addr_make("owner");
        let admin = app.api().addr_make("admin");
        let strategy_factory = app.api().addr_make("factory");
        let delegation_manager = app.api().addr_make("delegation");
        let slash_manager = app.api().addr_make("slash");
        let initial_strategy_whitelister = app.api().addr_make("whitelister");

        let msg = bvs_strategy_manager::msg::InstantiateMsg {
            delegation_manager: delegation_manager.to_string(),
            slash_manager: slash_manager.to_string(),
            strategy_factory: strategy_factory.to_string(),
            initial_strategy_whitelister: initial_strategy_whitelister.to_string(),
            initial_owner: owner.to_string(),
            pauser: owner.to_string(),
            unpauser: owner.to_string(),
            initial_paused_status: 0,
        };
        let contract_addr = app
            .instantiate_contract(contract_id, admin, &msg, &[], "BVS Strategy Manager", None)
            .unwrap();

        let contract = StrategyManagerContract(contract_addr);
        (contract, contract_id)
    }

    // strategy factory try to query the underlying cw20 token so we kinda need to make it real
    // bongo tokens address wouldn't work, even if it does it defeat the purpose of being this an
    // integration test
    fn upload_instantiate_token(app: &mut App) -> (Addr, u64) {
        let contract_id = app.store_code(cw20_contract());
        let admin = app.api().addr_make("admin");

        let msg = cw20_base::msg::InstantiateMsg {
            marketing: None,
            name: "Mock Token".to_string(),
            symbol: "MCK".to_string(),
            decimals: 6,
            initial_balances: vec![cw20::Cw20Coin {
                address: admin.to_string(),
                amount: Uint128::new(1_000_000), // Give admin some tokens
            }],
            mint: None,
        };
        let contract_addr = app
            .instantiate_contract(contract_id, admin, &msg, &[], "CW20 Token", None)
            .unwrap();

        (contract_addr, contract_id)
    }

    #[test]
    fn deploy_new_strategy() {
        let mut app = mock_app();
        let owner = app.api().addr_make("owner");
        let (token, _) = upload_instantiate_token(&mut app);
        let (manager, _) = upload_instantiate_manager(&mut app);

        let base_strategy_id = app.store_code(base_strategy_contract()); // we do not want initiate
                                                                         // it now, we will to test
                                                                         // factory doing the job
                                                                         // of initiating it
        let (factory, _) = upload_instantiate_factory(&mut app, manager.addr(), base_strategy_id);

        {
            // we gotta let factory in by the manager to give whitelist authority
            let whitelist_msg = bvs_strategy_manager::msg::ExecuteMsg::SetStrategyWhitelister {
                new_strategy_whitelister: factory.addr().to_string(),
            };
            let wrapped_msg = manager.call(whitelist_msg).unwrap();

            let _ = app.execute(owner.clone(), wrapped_msg).unwrap();
        }

        {
            let deploy_new_strategy_msg =
                bvs_strategy_factory::msg::ExecuteMsg::DeployNewStrategy {
                    token: token.to_string(),
                    pauser: app.api().addr_make("owner").to_string(),
                    unpauser: app.api().addr_make("owner").to_string(),
                };

            let wrapped_msg = factory.call(deploy_new_strategy_msg).unwrap();

            let res = app.execute(owner, wrapped_msg);

            assert_eq!(res.is_ok(), true);

            assert_eq!(res.unwrap().data, None);
        }
    }

    #[test]
    fn update_config() {
        let mut app = mock_app();
        let owner = app.api().addr_make("owner");
        let new_owner = app.api().addr_make("new_owner");
        let (token, _) = upload_instantiate_token(&mut app);
        let (manager, _) = upload_instantiate_manager(&mut app);

        let base_strategy_id = app.store_code(base_strategy_contract()); // we do not want initiate
                                                                         // it now, we will to test
                                                                         // factory doing the job
                                                                         // of initiating it
        let (factory, _) = upload_instantiate_factory(&mut app, manager.addr(), base_strategy_id);

        let msg = factory
            .call(bvs_strategy_factory::msg::ExecuteMsg::UpdateConfig {
                new_owner: owner.to_string(),
                strategy_code_id: 30,
            })
            .unwrap();

        let res = app.execute(owner, msg);

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn blacklist_token() {
        let mut app = mock_app();
        let owner = app.api().addr_make("owner");
        let (token, _) = upload_instantiate_token(&mut app);
        let (manager, _) = upload_instantiate_manager(&mut app);

        let base_strategy_id = app.store_code(base_strategy_contract()); // we do not want initiate
                                                                         // it now, we will to test
                                                                         // factory doing the job
                                                                         // of initiating it
        let (factory, _) = upload_instantiate_factory(&mut app, manager.addr(), base_strategy_id);

        let msg = factory
            .call(bvs_strategy_factory::msg::ExecuteMsg::BlacklistTokens {
                tokens: vec![token.to_string()],
            })
            .unwrap();

        let res = app.execute(owner, msg);

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn remove_add_strategies_from_whitelist() {
        let mut app = mock_app();
        let owner = app.api().addr_make("owner");
        let (token, _) = upload_instantiate_token(&mut app);
        let (manager, _) = upload_instantiate_manager(&mut app);

        let base_strategy_id = app.store_code(base_strategy_contract()); // we do not want initiate
                                                                         // it now, we will to test
                                                                         // factory doing the job
                                                                         // of initiating it
        let (factory, _) = upload_instantiate_factory(&mut app, manager.addr(), base_strategy_id);

        {
            // we gotta let factory in by the manager to give whitelist authority
            let whitelist_msg = bvs_strategy_manager::msg::ExecuteMsg::SetStrategyWhitelister {
                new_strategy_whitelister: factory.addr().to_string(),
            };
            let wrapped_msg = manager.call(whitelist_msg).unwrap();

            let _ = app.execute(owner.clone(), wrapped_msg).unwrap();
        }

        let mut strategy_address: Addr = Addr::unchecked("");

        {
            // deploy new strat so we can test removing it from whitelist
            let deploy_new_strategy_msg =
                bvs_strategy_factory::msg::ExecuteMsg::DeployNewStrategy {
                    token: token.to_string(),
                    pauser: app.api().addr_make("owner").to_string(),
                    unpauser: app.api().addr_make("owner").to_string(),
                };

            let wrapped_msg = factory.call(deploy_new_strategy_msg).unwrap();

            let res = app.execute(owner.clone(), wrapped_msg);

            assert_eq!(res.is_ok(), true);

            let app_res = res.unwrap();

            // find the new strategy address
            // we'll be using it later to test de-whitelisting and whitelisting
            for e in app_res.events.iter() {
                if e.ty == "wasm-StrategyAddedToDepositWhitelist" {
                    e.attributes.iter().for_each(|a| {
                        if a.key == "strategy" {
                            strategy_address = Addr::unchecked(a.value.as_str());
                        }
                    });
                    break;
                }
            }
        }

        {
            // let's remove the strategy we deployed earlier from the whitelist
            let msg = factory
                .call(
                    bvs_strategy_factory::msg::ExecuteMsg::RemoveStrategiesFromWhitelist {
                        strategies: vec![token.to_string()],
                    },
                )
                .unwrap();

            let res = app.execute(owner.clone(), msg);

            assert_eq!(res.is_ok(), true);
        }

        {
            // let's add the removed strategy back to the whitelist
            println!("strategy address: {}", strategy_address.to_string());
            let msg = factory
                .call(bvs_strategy_factory::msg::ExecuteMsg::WhitelistStrategies {
                    strategies_to_whitelist: vec![strategy_address.to_string()],
                    third_party_transfers_forbidden_values: vec![false],
                })
                .unwrap();

            let res = app.execute(owner.clone(), msg);

            assert!(res.is_ok());
        }
    }

    #[test]
    fn transfer_ownership() {
        let mut app = mock_app();
        let owner = app.api().addr_make("owner");
        let new_owner = app.api().addr_make("new_owner");
        let (factory, _) = upload_instantiate_factory(&mut app, owner.clone(), 30);

        let msg = factory
            .call(bvs_strategy_factory::msg::ExecuteMsg::TransferOwnership {
                new_owner: new_owner.to_string(),
            })
            .unwrap();

        let res = app.execute(owner, msg);

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn unauthorized_transfer_ownership() {
        let mut app = mock_app();
        let owner = app.api().addr_make("true_owner");
        let fake_owner = app.api().addr_make("fake_owner");
        let new_owner = app.api().addr_make("new_owner");
        let (factory, _) = upload_instantiate_factory(&mut app, owner.clone(), 30);

        let msg = factory
            .call(bvs_strategy_factory::msg::ExecuteMsg::TransferOwnership {
                new_owner: new_owner.to_string(),
            })
            .unwrap();

        let res = app.execute(fake_owner, msg);

        assert_eq!(res.is_err(), true);
    }
}
