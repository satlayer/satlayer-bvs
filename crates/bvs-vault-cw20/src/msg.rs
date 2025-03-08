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
    pub cw20_contract: String,
}

pub type ExecuteMsg = bvs_vault_base::msg::VaultExecuteMsg;

pub type QueryMsg = bvs_vault_base::msg::VaultQueryMsg;
