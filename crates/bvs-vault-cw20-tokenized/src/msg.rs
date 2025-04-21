use bvs_vault_base::msg::VaultExecuteMsg;
use bvs_vault_base::msg::VaultQueryMsg;
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

    /// The vault itself is a CW20 token, which will serve as receipt cw20 token.
    /// With extended functionality to be a vault.
    /// This field is the cw20 compliant `InstantiateMsg` for the receipt cw20 token.
    pub receipt_cw20_instantiate_base: cw20_base::msg::InstantiateMsg,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Supports the same [Cw20ExecuteMsg](cw20_base::msg::ExecuteMsg) as the `cw20-base` contract.
    /// Cw20 compliant messages are passed to the `cw20-base` contract.
    /// EXCEPT for the `Burn` and `BurnFrom` messages.
    Base(Cw20ExecuteMsg),

    /// Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the `bvs-vault-base` contract.
    Extended(VaultExecuteMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Binary)]
    Base(Cw20QueryMsg),

    /// Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the `bvs-vault-base` contract.
    #[returns(cosmwasm_std::Binary)]
    Extended(VaultQueryMsg),
}

#[cw_serde]
pub struct MigrateMsg {}
