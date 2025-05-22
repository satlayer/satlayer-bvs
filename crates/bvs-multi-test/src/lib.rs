#![cfg(not(target_arch = "wasm32"))]
// Only exposed on unit and integration testing, not compiled to Wasm.

use bvs_guardrail::testing::GuardrailContract;
pub use bvs_library::testing::{Cw20TokenContract, TestingContract};
pub use bvs_pauser::testing::PauserContract;
pub use bvs_registry::testing::RegistryContract;
pub use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_base::msg::{RecipientAmount, VaultExecuteMsg};
pub use bvs_vault_cw20::testing::VaultCw20Contract;
pub use bvs_vault_router::testing::VaultRouterContract;
use cosmwasm_std::{coins, Addr, Env, Uint128};
use cw_multi_test::{App, Executor};

pub struct BvsMultiTest {
    pub app: App,
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
    owner: Addr,
}

/// [BvsMultiTest] provides a convenient way to bootstrap all the necessary contracts
/// for testing a Service in the BVS ecosystem.
impl BvsMultiTestBuilder {
    /// Creates a new instance of [BvsMultiTestBuilder] with the given [App] and [Env].
    pub fn new(app: App, env: Env, owner: Addr) -> Self {
        Self { app, env, owner }
    }

    /// Builds the [BvsMultiTest] instance.
    ///
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

        let cw20_token = Self::deploy_cw20_token(&mut self.app, &self.env);
        let cw20_vault = Self::deploy_cw20_vault(
            &mut self.app,
            &self.env,
            &pauser,
            &vault_router,
            operator,
            cw20_token.clone().addr,
        );

        Self::deposit_to_vault(
            &mut self.app,
            self.owner,
            &bank_vault,
            &cw20_vault,
            &cw20_token,
        );

        BvsMultiTest {
            app: self.app,
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
    fn deploy_bank_vault(
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

    /// Creates 10 users and each deposits different amounts to the bank vault and cw20 vault.
    fn deposit_to_vault(
        app: &mut App,
        owner: Addr,
        bank_vault: &VaultBankContract,
        cw20_vault: &VaultCw20Contract,
        cw20_token: &Cw20TokenContract,
    ) {
        let denom = "denom";
        let amounts: Vec<u128> = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];

        let deposits: Vec<_> = (1..=10)
            .map(|i| {
                let user = app.api().addr_make(&format!("user/{}", i));

                // trasnsfer native tokens to user
                // NOTICE: owner must have enough native tokens
                let amount = amounts[i - 1];
                app.send_tokens(owner.clone(), user.clone(), &coins(amount, denom))
                    .unwrap();

                // mint CW20 tokens to user
                cw20_token.fund(app, &user.clone(), amount);
                // increase cw20 vault allowance
                cw20_token.increase_allowance(app, &user.clone(), &cw20_vault.addr, amount);

                let msg = VaultExecuteMsg::DepositFor(RecipientAmount {
                    amount: Uint128::new(amount),
                    recipient: user.clone(),
                });

                (user, msg, amount)
            })
            .collect();

        deposits
            .clone()
            .into_iter()
            .for_each(|(user, msg, amount)| {
                bank_vault
                    .execute_with_funds(app, &user, &msg, coins(amount, denom))
                    .unwrap();
            });

        deposits.into_iter().for_each(|(user, msg, _)| {
            cw20_vault.execute(app, &user, &msg).unwrap();
        });
    }

    /// Deploys a new [VaultCw20Contract] with the given operator and cw20 contract address.
    fn deploy_cw20_vault(
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
    fn deploy_cw20_token(app: &mut App, env: &Env) -> Cw20TokenContract {
        Cw20TokenContract::new(app, env, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bvs_vault_bank::msg::QueryMsg;
    use bvs_vault_base::msg::{RecipientAmount, VaultExecuteMsg};
    use cosmwasm_std::{coin, testing::mock_env};
    use cw_multi_test::App;

    #[test]
    fn test_new() {
        let app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });
        let env = mock_env();

        let owner = app.api().addr_make("owner");
        let mut bvs = BvsMultiTestBuilder::new(app, env, owner).build();

        let denom = "denom";
        let amounts: Vec<u128> = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];

        // test bank vault and cw20 vault
        for i in 1..=10 {
            let user = bvs.app.api().addr_make(&format!("user/{}", i));
            let amount = Uint128::new(amounts[i - 1]);

            // these two msgs are general for bank vault and cw20 vault
            let query_shares = QueryMsg::Shares {
                staker: user.to_string(),
            };
            let msg = VaultExecuteMsg::WithdrawTo(RecipientAmount {
                recipient: user.clone(),
                amount,
            });

            // bank vault test
            {
                // query user left native token balance
                let user_balance = bvs.app.wrap().query_balance(&user, denom).unwrap();
                assert_eq!(user_balance, coin(0, denom));

                // query bank vault shares
                let bank_vault_shares: Uint128 =
                    bvs.bank_vault.query(&bvs.app, &query_shares).unwrap();
                assert_eq!(bank_vault_shares, amount);

                // withdraw from bank vault
                bvs.bank_vault.execute(&mut bvs.app, &user, &msg).unwrap();

                // query user left native token balance
                let user_balance = bvs.app.wrap().query_balance(&user, denom).unwrap();
                assert_eq!(user_balance, coin(u128::from(amount), denom));
            }

            // test cw20 vault
            {
                // query user1 left cw20 balance
                let user_balance = bvs.cw20_token.balance(&bvs.app, &user);
                assert_eq!(user_balance, 0);

                // query cw20 vault shares
                let cw20_vault_shares: Uint128 =
                    bvs.cw20_vault.query(&bvs.app, &query_shares).unwrap();
                assert_eq!(cw20_vault_shares, amount);

                // withdraw from cw20 vault
                bvs.cw20_vault.execute(&mut bvs.app, &user, &msg).unwrap();

                // query user left cw20 balance
                let user_balance = bvs.cw20_token.balance(&bvs.app, &user);
                assert_eq!(user_balance, u128::from(amount));
            }
        }
    }
}
