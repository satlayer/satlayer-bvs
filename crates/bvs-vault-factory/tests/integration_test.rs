use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_pauser::testing::PauserContract;
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_factory::testing::VaultFactoryContract;
use bvs_vault_router::{
    msg::{ExecuteMsg, QueryMsg, VaultListResponse},
    testing::VaultRouterContract,
    ContractError,
};
use cosmwasm_std::{testing::mock_env, Empty, Event};
use cw_multi_test::{App, Contract};

struct TestContracts {
    vault_router: VaultRouterContract,
    vault_factory: VaultFactoryContract,
    cw20_token: Cw20TokenContract,
    bank_wrapper: Box<dyn Contract<Empty>>,
    cw20_vault_wrapper: Box<dyn Contract<Empty>>,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::default();
        let env = mock_env();

        let _ = PauserContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);
        let vault_factory = VaultFactoryContract::new(&mut app, &env, None);

        let bank_wrapper = VaultBankContract::wrapper();
        let cw20_vault_wrapper = VaultCw20Contract::wrapper();

        (
            app,
            Self {
                cw20_token: cw20,
                vault_router,
                vault_factory,
                bank_wrapper,
                cw20_vault_wrapper,
            },
        )
    }
}

#[test]
fn test_cw20_vault_deployment() {
    let (mut app, contracts) = TestContracts::init();

    let factory = contracts.vault_factory;
    let cw20_token = contracts.cw20_token;
    let owner = app.api().addr_make("owner");

    let msg = bvs_vault_factory::msg::ExecuteMsg::SetRouter {
        router: contracts.vault_router.addr().to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap();

    println!("{:?}", res);

    let cw20_vault_code_id = app.store_code(contracts.cw20_vault_wrapper);

    let msg = bvs_vault_factory::msg::ExecuteMsg::DeployCw20 {
        code_id: cw20_vault_code_id,
        cw20: cw20_token.addr().to_string(),
    };

    let res = factory.execute(&mut app, &owner, &msg).unwrap();

    println!("{:?}", res);
}
