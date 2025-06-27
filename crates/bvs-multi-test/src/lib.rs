#![cfg(not(target_arch = "wasm32"))]
// Only exposed on unit and integration testing, not compiled to Wasm.

use bvs_guardrail::testing::GuardrailContract;
pub use bvs_library::testing::{Cw20TokenContract, TestingContract};
pub use bvs_pauser::testing::PauserContract;
pub use bvs_registry::testing::RegistryContract;
pub use bvs_vault_bank::testing::VaultBankContract;
pub use bvs_vault_cw20::testing::VaultCw20Contract;
pub use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::{Addr, Env};
use cw20::MinterResponse;
use cw_multi_test::App;

pub struct BvsMultiTest {
    pub pauser: PauserContract,
    pub registry: RegistryContract,
    pub guardrail: GuardrailContract,
    pub vault_router: VaultRouterContract,
}

/// [BvsMultiTest] provides a convenient way to bootstrap all the necessary contracts
/// for testing a Service in the BVS ecosystem.
impl BvsMultiTest {
    /// Creates a new instance of [BvsMultiTest] with the given [App] and [Env].
    /// It initializes the [PauserContract], [RegistryContract], and [VaultRouterContract].
    pub fn new(app: &mut App, env: &Env) -> Self {
        let pauser = PauserContract::new(app, env, None);
        let registry = RegistryContract::new(app, env, None);
        let guardrail = GuardrailContract::new(app, env, None);
        let vault_router = VaultRouterContract::new(app, env, None);

        Self {
            pauser,
            registry,
            guardrail,
            vault_router,
        }
    }

    /// Deploys a new [VaultBankContract] with the given operator and denom.
    pub fn deploy_bank_vault(
        &self,
        app: &mut App,
        env: &Env,
        operator: impl Into<String>,
        denom: impl Into<String>,
    ) -> VaultBankContract {
        let init_msg = bvs_vault_bank::msg::InstantiateMsg {
            pauser: self.pauser.addr.to_string(),
            router: self.vault_router.addr.to_string(),
            operator: operator.into(),
            denom: denom.into(),
        };

        let bank_contract = VaultBankContract::new(app, env, Some(init_msg));
        let set_vault = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: bank_contract.addr.to_string(),
            whitelisted: true,
        };
        let owner = app.api().addr_make("owner");
        self.vault_router.execute(app, &owner, set_vault).unwrap();
        bank_contract
    }

    /// Deploys a new [VaultCw20Contract] with the given operator and cw20 contract address.
    pub fn deploy_cw20_vault(
        &self,
        app: &mut App,
        env: &Env,
        operator: impl Into<String>,
        cw20_contract: Addr,
    ) -> VaultCw20Contract {
        let init_msg = bvs_vault_cw20::msg::InstantiateMsg {
            pauser: self.pauser.addr.to_string(),
            router: self.vault_router.addr.to_string(),
            operator: operator.into(),
            cw20_contract: cw20_contract.to_string(),
        };

        let bank_contract = VaultCw20Contract::new(app, env, Some(init_msg));
        let set_vault = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: bank_contract.addr.to_string(),
            whitelisted: true,
        };
        let owner = app.api().addr_make("owner");
        self.vault_router.execute(app, &owner, set_vault).unwrap();
        bank_contract
    }

    /// Deploys a new [Cw20TokenContract] with the given symbol and minter address.
    pub fn deploy_cw20_token(
        &self,
        app: &mut App,
        env: &Env,
        symbol: impl Into<String>,
        minter: impl Into<String>,
    ) -> Cw20TokenContract {
        let symbol = symbol.into();
        let init_msg = cw20_base::msg::InstantiateMsg {
            symbol: symbol.clone(),
            name: format!("Token {symbol}"),
            decimals: 18,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: minter.into(),
                cap: None,
            }),
            marketing: None,
        };

        Cw20TokenContract::new(app, env, Some(init_msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_env;
    use cw_multi_test::App;

    #[test]
    fn test_new() {
        let mut app = App::default();
        let env = mock_env();
        BvsMultiTest::new(&mut app, &env);
    }

    #[test]
    fn test_deploy_bank_vault() {
        let mut app = App::default();
        let env = mock_env();
        let bvs = BvsMultiTest::new(&mut app, &env);

        let operator = app.api().addr_make("operator");
        let denom = "denom".to_string();

        let bank_vault = bvs.deploy_bank_vault(&mut app, &env, operator.clone(), denom.clone());
        assert_eq!(bank_vault.init.operator, operator.to_string());
        assert_eq!(bank_vault.init.denom, denom);
    }

    #[test]
    fn test_deploy_cw20_vault() {
        let mut app = App::default();
        let env = mock_env();
        let bvs = BvsMultiTest::new(&mut app, &env);

        let owner = app.api().addr_make("owner").to_string();
        let token = bvs.deploy_cw20_token(&mut app, &env, "FRUIT", owner.clone());

        let operator = app.api().addr_make("operator");
        let cw20_vault =
            bvs.deploy_cw20_vault(&mut app, &env, operator.clone(), token.addr.clone());
        assert_eq!(cw20_vault.init.operator, operator.to_string());
        assert_eq!(cw20_vault.init.cw20_contract, token.addr.to_string());
    }
}
