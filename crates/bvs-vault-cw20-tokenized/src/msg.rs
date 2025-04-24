use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use cw20_base::msg::QueryMsg as Cw20QueryMsg;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the `pauser` contract.
    pub pauser: String,
    /// The address of the `router` contract.
    pub router: String,
    /// The address of the `operator`.
    /// Each vault is delegated to an `operator`.
    pub operator: String,
    /// The address of the CW20 contract, underlying asset of the vault.
    ///
    /// ### CW20 Variant Warning
    ///
    /// Underlying assets that are not strictly CW20 compliant may cause unexpected behavior in token balances.
    /// For example, any token with a fee-on-transfer mechanism is not supported.
    ///
    /// Therefore, we do not support non-standard CW20 tokens.
    /// Vault deployed with such tokens will be blacklisted in the vault-router.
    pub staking_cw20_contract: String,

    pub receipt_cw20_instantiate_base: cw20_base::msg::InstantiateMsg,
}

/// Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the `bvs-vault-base` contract.
type ExtendedExecuteMsg = bvs_vault_base::msg::VaultExecuteMsg;

pub enum ExecuteMsg {
    Base(Cw20ExecuteMsg),
    Extended(ExtendedExecuteMsg),
}

/// Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the `bvs-vault-base` contract.
type ExtendedQueryMsg = bvs_vault_base::msg::VaultQueryMsg;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Binary)]
    Base(Cw20QueryMsg),

    #[returns(cosmwasm_std::Binary)]
    Extended(ExtendedQueryMsg),
}
#[cw_serde]
pub struct MigrateMsg {}
