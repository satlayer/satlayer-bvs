use bvs_library::testing::TestingContract;
use bvs_pauser::testing::PauserContract;
use bvs_registry::testing::RegistryContract;
use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::Env;
use cw_multi_test::App;

pub struct BvsMultiTest {
    pub pauser: PauserContract,
    pub registry: RegistryContract,
    pub vault_router: VaultRouterContract,
}

impl BvsMultiTest {
    pub fn new(app: &mut App, env: &Env) -> Self {
        let pauser = PauserContract::new(app, &env, None);
        let registry = RegistryContract::new(app, &env, None);
        let vault_router = VaultRouterContract::new(app, &env, None);

        Self {
            pauser,
            registry,
            vault_router,
        }
    }
}
