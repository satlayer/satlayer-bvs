use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the `pauser` contract.
    /// See [auth::set_pauser] for more information.
    pub pauser: String,
    /// The address of the `router` contract.
    /// See [auth::set_router] for more information.
    pub router: String,
    /// The address of the `operator`.
    /// Each vault is delegated to an `operator`.
    pub operator: String,
    /// The denom supported by this vault.
    pub denom: String,
}

/// Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the `bvs-vault-base` contract.
pub type ExecuteMsg = bvs_vault_base::msg::VaultExecuteMsg;

/// Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the `bvs-vault-base` contract.
pub type QueryMsg = bvs_vault_base::msg::VaultQueryMsg;

#[cw_serde]
pub struct MigrateMsg {}
