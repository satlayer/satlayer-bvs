use crate::error::VaultError;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Api, Uint128};

/// Vault `ExecuteMsg`, to be implemented by the vault contract.
/// Callable by any `sender`, redeemable by any `recipient`.
/// The `sender` can be the same as the `recipient` in some cases.
#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum VaultExecuteMsg {
    /// ExecuteMsg Deposit assets into the vault.
    /// Sender must transfer the assets to the vault contract (this is implementation agnostic).
    /// The vault contract must mint shares to the `recipient`.
    /// Vault must be whitelisted in the `vault-router` to accept deposits.
    Deposit(RecipientAmount),

    /// ExecuteMsg Withdraw assets from the vault.
    /// Sender must have enough shares to withdraw the requested amount to the `recipient`.
    /// If the Vault is delegated to an `operator`, withdrawals must be queued.
    /// Operator must not be validating any services for instant withdrawals.
    Withdraw(RecipientAmount),
    // /// ExecuteMsg QueueWithdrawal assets from the vault.
    // /// Sender must have enough shares to queue the requested amount to the `recipient`.
    // /// Once the withdrawal is queued,
    // /// the `recipient` can redeem the withdrawal after the lock period.
    // /// Once the withdrawal is locked,
    // /// the `sender` cannot cancel the withdrawal.
    // /// The time-lock is enforced by the vault and cannot be changed retroactively.
    // ///
    // /// ### Lock Period Extension
    // /// New withdrawals will extend the lock period of any existing withdrawals.
    // /// You can queue the withdrawal to a different `recipient` than the `sender` to avoid this.
    // QueueWithdrawal(RecipientAmount),
    //
    // /// ExecuteMsg RedeemWithdrawal assets from the vault for withdrawal.
    // /// After the lock period, the `sender` (must be the `recipient` of the original withdrawal)
    // /// can redeem the withdrawal.
    // RedeemWithdrawal(RecipientAmount),
}

/// This struct is used to represent the recipient and amount fields together.
#[cw_serde]
pub struct RecipientAmount {
    pub recipient: Addr,
    pub amount: Uint128,
}

impl RecipientAmount {
    /// Validate the recipient: [`Addr`] and amount: [`Uint128`] fields.
    /// The recipient must be a valid [`Addr`], and the amount must be greater than zero.
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        if self.amount.is_zero() {
            return Err(VaultError::zero("Amount cannot be zero."));
        }

        api.addr_validate(self.recipient.as_str())?;
        Ok(())
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum VaultQueryMsg {
    /// QueryMsg Shares: get the shares of a staker.
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

    /// QueryMsg VaultInfo: get the vault information.
    #[returns(VaultInfoResponse)]
    VaultInfo {},
}

/// The response to the `Shares` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct SharesResponse(Uint128);

/// The response to the `Assets` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct AssetsResponse(Uint128);

/// The response to the `ConvertToAssets` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct ConvertToAssetsResponse(Uint128);

/// The response to the `ConvertToShares` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct ConvertToSharesResponse(Uint128);

/// The response to the `TotalShares` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct TotalSharesResponse(Uint128);

/// The response to the `TotalAssets` query.
/// Not exported.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
struct TotalAssetsResponse(Uint128);

#[cw_serde]
pub struct VaultInfoResponse {
    /// The total shares in circulation
    pub total_shares: Uint128,

    /// The total assets under management
    pub total_assets: Uint128,

    /// The `vault-router` contract address
    pub router: Addr,

    /// The `pauser` contract address
    pub pauser: Addr,

    /// The `operator` that this vault is delegated to
    pub operator: Addr,

    /// Whether the vault has enabled slashing
    pub slashing: bool,

    /// Asset identifier, using the CAIP-19 format.
    pub asset_id: String,

    /// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub contract: String,

    /// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub version: String,
}
