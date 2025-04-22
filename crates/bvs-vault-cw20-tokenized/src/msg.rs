use bvs_vault_base::msg::{
    AssetsResponse, ConvertToAssetsResponse, ConvertToSharesResponse, QueuedWithdrawalResponse,
    Recipient, RecipientAmount, SharesResponse, TotalAssetsResponse, TotalSharesResponse,
    VaultInfoResponse,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use cw20::{Expiration, Logo};

#[cw_serde]
pub enum ExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Only with "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    /// Only with the "mintable" extension. If authorized, creates amount new tokens
    /// and adds to the recipient balance.
    Mint { recipient: String, amount: Uint128 },
    /// Only with the "mintable" extension. The current minter may set
    /// a new minter. Setting the minter to None will remove the
    /// token's minter forever.
    UpdateMinter { new_minter: Option<String> },
    /// Only with the "marketing" extension. If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(Logo),

    /// ExecuteMsg Deposit assets into the vault.
    /// Sender must transfer the assets to the vault contract (this is implementation agnostic).
    /// The vault contract must mint shares to the `recipient`.
    /// Vault must be whitelisted in the `vault-router` to accept deposits.
    DepositFor(RecipientAmount),

    /// ExecuteMsg Withdraw assets from the vault.
    /// Sender must have enough shares to withdraw the requested amount to the `recipient`.
    /// If the Vault is delegated to an `operator`, withdrawals must be queued.
    /// Operator must not be validating any services for instant withdrawals.
    WithdrawTo(RecipientAmount),

    /// ExecuteMsg QueueWithdrawal assets from the vault.
    /// Sender must have enough shares to queue the requested amount to the `recipient`.
    /// Once the withdrawal is queued,
    /// the `recipient` can redeem the withdrawal after the lock period.
    /// Once the withdrawal is locked,
    /// the `sender` cannot cancel the withdrawal.
    /// The time-lock is enforced by the vault and cannot be changed retroactively.
    ///
    /// ### Lock Period Extension
    /// New withdrawals will extend the lock period of any existing withdrawals.
    /// You can queue the withdrawal to a different `recipient` than the `sender` to avoid this.
    QueueWithdrawalTo(RecipientAmount),

    /// ExecuteMsg RedeemWithdrawal all queued shares into assets from the vault for withdrawal.
    /// After the lock period, the `sender` (must be the `recipient` of the original withdrawal)
    /// can redeem the withdrawal.
    RedeemWithdrawalTo(Recipient),
}

impl From<cw20_base::msg::ExecuteMsg> for ExecuteMsg {
    // Theoretically, this will never failed
    // as this vault plan to compliant with cw20 specs in terms of interface.
    fn from(msg: cw20_base::msg::ExecuteMsg) -> Self {
        match msg {
            cw20_base::msg::ExecuteMsg::Transfer { recipient, amount } => {
                ExecuteMsg::Transfer { recipient, amount }
            }
            cw20_base::msg::ExecuteMsg::Burn { amount } => ExecuteMsg::Burn { amount },
            cw20_base::msg::ExecuteMsg::Send {
                contract,
                amount,
                msg,
            } => ExecuteMsg::Send {
                contract,
                amount,
                msg,
            },
            cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            } => ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            },
            cw20_base::msg::ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            } => ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            },
            cw20_base::msg::ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            },
            cw20_base::msg::ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            } => ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            },
            cw20_base::msg::ExecuteMsg::BurnFrom { owner, amount } => {
                ExecuteMsg::BurnFrom { owner, amount }
            }
            cw20_base::msg::ExecuteMsg::Mint { recipient, amount } => {
                ExecuteMsg::Mint { recipient, amount }
            }
            cw20_base::msg::ExecuteMsg::UpdateMinter { new_minter } => {
                ExecuteMsg::UpdateMinter { new_minter }
            }
            cw20_base::msg::ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            } => ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            },
            cw20_base::msg::ExecuteMsg::UploadLogo(logo) => ExecuteMsg::UploadLogo(logo),
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    #[returns(cw20::MinterResponse)]
    Minter {},
    /// Only with "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this spender has been granted. Supports pagination.
    #[returns(cw20::AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "marketing" extension
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    #[returns(cw20::MarketingInfoResponse)]
    MarketingInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    #[returns(cw20::DownloadLogoResponse)]
    DownloadLogo {},

    /// QueryMsg Shares: get the shares of a staker.
    /// Shares in this tokenized vault are CW20 receipt tokens.
    /// The interface is kept the same as the original vault.
    /// to avoid breaking and minimize changes in vault consumer/frontend code.
    #[returns(SharesResponse)]
    Shares { staker: String },

    /// QueryMsg Assets: get the assets of a staker, converted from shares.
    #[returns(AssetsResponse)]
    Assets { staker: String },

    /// QueryMsg ConvertToAssets: convert shares to assets.
    #[returns(ConvertToAssetsResponse)]
    ConvertToAssets { shares: Uint128 },

    /// QueryMsg ConvertToShares: convert assets to shares.
    #[returns(ConvertToSharesResponse)]
    ConvertToShares { assets: Uint128 },

    /// QueryMsg TotalShares: get the total shares in circulation.
    #[returns(TotalSharesResponse)]
    TotalShares {},

    /// QueryMsg TotalAssets: get the total assets under vault.
    #[returns(TotalAssetsResponse)]
    TotalAssets {},

    /// QueryMsg QueuedWithdrawal: get the queued withdrawal and unlock timestamp under vault.
    #[returns(QueuedWithdrawalResponse)]
    QueuedWithdrawal { staker: String },

    /// QueryMsg VaultInfo: get the vault information.
    #[returns(VaultInfoResponse)]
    VaultInfo {},
}

impl From<QueryMsg> for cw20_base::msg::QueryMsg {
    fn from(val: QueryMsg) -> Self {
        match val {
            QueryMsg::Balance { address } => cw20_base::msg::QueryMsg::Balance { address },
            QueryMsg::TokenInfo {} => cw20_base::msg::QueryMsg::TokenInfo {},
            QueryMsg::Minter {} => cw20_base::msg::QueryMsg::Minter {},
            QueryMsg::Allowance { owner, spender } => {
                cw20_base::msg::QueryMsg::Allowance { owner, spender }
            }
            QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            } => cw20_base::msg::QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            } => cw20_base::msg::QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            },
            QueryMsg::AllAccounts { start_after, limit } => {
                cw20_base::msg::QueryMsg::AllAccounts { start_after, limit }
            }
            QueryMsg::MarketingInfo {} => cw20_base::msg::QueryMsg::MarketingInfo {},
            QueryMsg::DownloadLogo {} => cw20_base::msg::QueryMsg::DownloadLogo {},

            _ => panic!("This QueryMsg cannot be converted into cw20_base::msg::QueryMsg"),
        }
    }
}

impl From<cw20_base::msg::QueryMsg> for QueryMsg {
    fn from(msg: cw20_base::msg::QueryMsg) -> Self {
        match msg {
            cw20_base::msg::QueryMsg::Balance { address } => QueryMsg::Balance { address },
            cw20_base::msg::QueryMsg::TokenInfo {} => QueryMsg::TokenInfo {},
            cw20_base::msg::QueryMsg::Minter {} => QueryMsg::Minter {},
            cw20_base::msg::QueryMsg::Allowance { owner, spender } => {
                QueryMsg::Allowance { owner, spender }
            }
            cw20_base::msg::QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            } => QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            },
            cw20_base::msg::QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            } => QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            },
            cw20_base::msg::QueryMsg::AllAccounts { start_after, limit } => {
                QueryMsg::AllAccounts { start_after, limit }
            }
            cw20_base::msg::QueryMsg::MarketingInfo {} => QueryMsg::MarketingInfo {},
            cw20_base::msg::QueryMsg::DownloadLogo {} => QueryMsg::DownloadLogo {},
        }
    }
}

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
pub struct MigrateMsg {}
