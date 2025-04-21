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

// /// Supports the same [Cw20ExecuteMsg](cw20_base::msg::ExecuteMsg) as the `cw20-base` contract.
// /// Cw20 compliant messages are passed to the `cw20-base` contract.
// /// EXCEPT for the `Burn` and `BurnFrom` messages.
// #[cw_serde]
// pub enum Cw20ExecuteMsgWithoutBurn {
//     /// Transfer is a base message to move tokens to another account without triggering actions
//     Transfer { recipient: String, amount: Uint128 },
//     /// Send is a base message to transfer tokens to a contract and trigger an action
//     /// on the receiving contract.
//     Send {
//         contract: String,
//         amount: Uint128,
//         msg: Binary,
//     },
//     /// Only with "approval" extension. Allows spender to access an additional amount tokens
//     /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
//     /// expiration with this one.
//     IncreaseAllowance {
//         spender: String,
//         amount: Uint128,
//         expires: Option<Expiration>,
//     },
//     /// Only with "approval" extension. Lowers the spender's access of tokens
//     /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
//     /// allowance expiration with this one.
//     DecreaseAllowance {
//         spender: String,
//         amount: Uint128,
//         expires: Option<Expiration>,
//     },
//     /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
//     /// if `env.sender` has sufficient pre-approval.
//     TransferFrom {
//         owner: String,
//         recipient: String,
//         amount: Uint128,
//     },
//     /// Only with "approval" extension. Sends amount tokens from owner -> contract
//     /// if `env.sender` has sufficient pre-approval.
//     SendFrom {
//         owner: String,
//         contract: String,
//         amount: Uint128,
//         msg: Binary,
//     },
//     /// Only with the "mintable" extension. If authorized, creates amount new tokens
//     /// and adds to the recipient balance.
//     Mint { recipient: String, amount: Uint128 },
//     /// Only with the "mintable" extension. The current minter may set
//     /// a new minter. Setting the minter to None will remove the
//     /// token's minter forever.
//     UpdateMinter { new_minter: Option<String> },
//     /// Only with the "marketing" extension. If authorized, updates marketing metadata.
//     /// Setting None/null for any of these will leave it unchanged.
//     /// Setting Some("") will clear this field on the contract storage
//     UpdateMarketing {
//         /// A URL pointing to the project behind this token.
//         project: Option<String>,
//         /// A longer description of the token and it's utility. Designed for tooltips or such
//         description: Option<String>,
//         /// The address (if any) who can update this data structure
//         marketing: Option<String>,
//     },
//     /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
//     UploadLogo(Logo),
// }

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
