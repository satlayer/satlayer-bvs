use cosmwasm_schema::cw_serde;

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
    /// Underlying assets that are not strictly CW20 compliant may cause unexpected behavior in token balances.
    /// For example, any token with a fee-on-transfer mechanism is not supported.
    ///
    /// Therefore, we do not support token non-standard CW20 tokens.
    /// Vault deployed with such tokens will be blacklisted in the vault-router.
    pub cw20_contract: String,
}

/// Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the `bvs-vault-base` contract.
pub type ExecuteMsg = bvs_vault_base::msg::VaultExecuteMsg;

/// Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the `bvs-vault-base` contract.
pub type QueryMsg = bvs_vault_base::msg::VaultQueryMsg;

#[cw_serde]
pub struct MigrateMsg {}
