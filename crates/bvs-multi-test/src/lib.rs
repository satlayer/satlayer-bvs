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
    pub bank_vault: VaultBankContract,
    pub cw20_vault: VaultCw20Contract,
    pub cw20_token: Cw20TokenContract,
}

pub struct BvsMultiTestBuilder {
    app: App,
    env: Env,
}

/// [BvsMultiTest] provides a convenient way to bootstrap all the necessary contracts
/// for testing a Service in the BVS ecosystem.
impl BvsMultiTestBuilder {
    /// Creates a new instance of [BvsMultiTestBuilder] with the given [App] and [Env].
    pub fn new(app: App, env: Env) -> Self {
        Self { app, env }
    }

    /// Builds the [BvsMultiTest] instance.
    /// It initializes the [PauserContract], [RegistryContract], [GuardrailContract], [VaultRouterContract],
    /// [VaultBankContract], [VaultCw20Contract], and [Cw20TokenContract].
    pub fn build(mut self) -> BvsMultiTest {
        let pauser = PauserContract::new(&mut self.app, &self.env, None);
        let registry = RegistryContract::new(&mut self.app, &self.env, None);
        let guardrail = GuardrailContract::new(&mut self.app, &self.env, None);
        let vault_router = VaultRouterContract::new(&mut self.app, &self.env, None);

        let operator = self.app.api().addr_make("operator");
        let denom = "denom";
        let bank_vault = Self::deploy_bank_vault(
            &mut self.app,
            &self.env,
            &pauser,
            &vault_router,
            operator.clone(),
            denom,
        );

        let cw20_token =
            Self::deploy_cw20_token(&mut self.app, &self.env, "TEST", operator.clone());
        let cw20_vault = Self::deploy_cw20_vault(
            &mut self.app,
            &self.env,
            &pauser,
            &vault_router,
            operator,
            cw20_token.clone().addr,
        );

        BvsMultiTest {
            pauser,
            registry,
            guardrail,
            vault_router,
            bank_vault,
            cw20_vault,
            cw20_token,
        }
    }

    /// Deploys a new [VaultBankContract] with the given operator and denom.
    pub fn deploy_bank_vault(
        app: &mut App,
        env: &Env,
        pauser: &PauserContract,
        vault_router: &VaultRouterContract,
        operator: impl Into<String>,
        denom: impl Into<String>,
    ) -> VaultBankContract {
        let init_msg = bvs_vault_bank::msg::InstantiateMsg {
            pauser: pauser.addr.to_string(),
            router: vault_router.addr.to_string(),
            operator: operator.into(),
            denom: denom.into(),
        };

        let bank_contract = VaultBankContract::new(app, env, Some(init_msg));
        let set_vault = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: bank_contract.addr.to_string(),
            whitelisted: true,
        };

        let owner = app.api().addr_make("owner");
        vault_router.execute(app, &owner, set_vault).unwrap();

        bank_contract
    }

    /// Deploys a new [VaultCw20Contract] with the given operator and cw20 contract address.
    pub fn deploy_cw20_vault(
        app: &mut App,
        env: &Env,
        pauser: &PauserContract,
        vault_router: &VaultRouterContract,
        operator: impl Into<String>,
        cw20_contract: Addr,
    ) -> VaultCw20Contract {
        let init_msg = bvs_vault_cw20::msg::InstantiateMsg {
            pauser: pauser.addr.to_string(),
            router: vault_router.addr.to_string(),
            operator: operator.into(),
            cw20_contract: cw20_contract.to_string(),
        };

        let bank_contract = VaultCw20Contract::new(app, env, Some(init_msg));
        let set_vault = &bvs_vault_router::msg::ExecuteMsg::SetVault {
            vault: bank_contract.addr.to_string(),
            whitelisted: true,
        };

        let owner = app.api().addr_make("owner");
        vault_router.execute(app, &owner, set_vault).unwrap();

        bank_contract
    }

    /// Deploys a new [Cw20TokenContract] with the given symbol and minter address.
    pub fn deploy_cw20_token(
        app: &mut App,
        env: &Env,
        symbol: impl Into<String>,
        minter: impl Into<String>,
    ) -> Cw20TokenContract {
        let symbol = symbol.into();
        let init_msg = cw20_base::msg::InstantiateMsg {
            symbol: symbol.clone(),
            name: format!("Token {}", symbol),
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
        let app = App::default();
        let env = mock_env();

        BvsMultiTestBuilder::new(app, env).build();
    }
}
