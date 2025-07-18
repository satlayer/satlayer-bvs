use bvs_pauser::api::Display;
use bvs_vault_base::msg::{
    Amount, AssetsResponse, ConvertToAssetsResponse, ConvertToSharesResponse,
    QueueWithdrawalToParams, QueuedWithdrawalResponse, RecipientAmount, RedeemWithdrawalToParams,
    SetApproveProxyParams, SharesResponse, TotalAssetsResponse, TotalSharesResponse,
    VaultInfoResponse,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use cw20::Expiration;

#[cw_serde]
#[derive(Display)]
pub enum ExecuteMsg {
    /// ExecuteMsg Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },

    /// ExecuteMsg Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },

    /// ExecuteMsg IncreaseAllowance allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },

    /// ExecuteMsg DecreaseAllowance Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },

    /// ExecuteMsg TransferFrom tansfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },

    /// ExecuteMsg SendFrom Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },

    /// ExecuteMsg DepositFor assets into the vault.
    /// Sender must transfer the assets to the vault contract (this is implementation agnostic).
    /// The vault contract must mint shares to the `recipient`.
    /// Vault must be whitelisted in the `vault-router` to accept deposits.
    DepositFor(RecipientAmount),

    /// ExecuteMsg QueueWithdrawalTo assets from the vault.
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
    QueueWithdrawalTo(QueueWithdrawalToParams),

    /// ExecuteMsg RedeemWithdrawalTo all queued shares into assets from the vault for withdrawal.
    /// After the lock period, the `sender` (must be the `recipient` of the original withdrawal)
    /// can redeem the withdrawal.
    RedeemWithdrawalTo(RedeemWithdrawalToParams),

    /// ExecuteMsg SlashLocked moves the assets from the vault to the `vault-router` contract for custody.
    /// Part of the [https://build.satlayer.xyz/getting-started/slashing](Programmable Slashing) lifecycle.
    /// This function can only be called by `vault-router`, and takes an absolute `amount` of assets to be moved.
    /// The amount is calculated and enforced by the router.
    /// Further utility of the assets, post-locked, is implemented and enforced on the router level.
    SlashLocked(Amount),

    /// ExecuteMsg ApproveProxy allows the `proxy`
    /// to queue withdrawal and redeem withdrawal on behalf of the `owner`.
    SetApproveProxy(SetApproveProxyParams),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// QueryMsg Balance: get the balance of a given address.
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },

    /// QueryMsg TokenInfo: get the token info of the contract.
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},

    /// QueryMsg Allowance: get the allowance of a given address.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },

    /// QueryMsg AllAllowances: get all allowances of a given address.
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// QueryMsg AllSpenderAllowances: get all allowances of a given address.
    /// Returns all allowances this spender has been granted. Supports pagination.
    #[returns(cw20::AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// QueryMsg AllAccounts: get all accounts of the contract.
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },

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
    QueuedWithdrawal { controller: String },

    /// QueryMsg VaultInfo: get the vault information.
    #[returns(VaultInfoResponse)]
    VaultInfo {},
}

impl TryFrom<QueryMsg> for cw20_base::msg::QueryMsg {
    type Error = String;

    fn try_from(val: QueryMsg) -> Result<Self, Self::Error> {
        match val {
            QueryMsg::Balance { address } => Ok(cw20_base::msg::QueryMsg::Balance { address }),
            QueryMsg::TokenInfo {} => Ok(cw20_base::msg::QueryMsg::TokenInfo {}),
            QueryMsg::Allowance { owner, spender } => {
                Ok(cw20_base::msg::QueryMsg::Allowance { owner, spender })
            }
            QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            } => Ok(cw20_base::msg::QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            }),
            QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            } => Ok(cw20_base::msg::QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            }),
            QueryMsg::AllAccounts { start_after, limit } => {
                Ok(cw20_base::msg::QueryMsg::AllAccounts { start_after, limit })
            }
            _ => Err("This QueryMsg cannot be converted into cw20_base::msg::QueryMsg".to_string()),
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
    /// The denom supported by this vault.
    pub denom: String,
    /// The decimals of the receipt token.
    /// Must be the same as the denom's decimals.
    pub decimals: u8,
    /// The symbol for the receipt token.
    pub symbol: String,
    /// The name of the receipt token.
    pub name: String,
}

#[cw_serde]
pub struct MigrateMsg {}
