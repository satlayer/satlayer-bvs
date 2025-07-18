use crate::error::VaultError;
use crate::shares::QueuedWithdrawalInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Api, Uint128};

/// Vault `ExecuteMsg`, to be implemented by the vault contract.
/// Callable by any `sender`, redeemable by any `recipient`.
/// The `sender` can be the same as the `recipient` in some cases.
#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum VaultExecuteMsg {
    /// ExecuteMsg DepositFor assets into the vault.
    /// Sender must transfer the assets to the vault contract (this is implementation agnostic).
    /// The vault contract must mint shares to the `recipient`.
    /// Vault must be whitelisted in the `vault-router` to accept deposits.
    DepositFor(RecipientAmount),

    /// ExecuteMsg QueueWithdrawalTo assets from the vault.
    /// Sender must have enough shares to queue the requested amount to the `controller`.
    /// Once the withdrawal is queued,
    /// the `controller` can redeem the withdrawal after the lock period.
    /// Once the withdrawal is locked,
    /// the `sender` cannot cancel the withdrawal.
    /// The time-lock is enforced by the vault and cannot be changed retroactively.
    ///
    /// ### Lock Period Extension
    /// New withdrawals will extend the lock period of any existing withdrawals.
    /// You can queue the withdrawal to a different `controller` than the `sender` to avoid this.
    QueueWithdrawalTo(QueueWithdrawalToParams),

    /// ExecuteMsg RedeemWithdrawalTo all queued shares into assets from the vault for withdrawal.
    /// After the lock period, the `sender` (must be the `controller` of the original withdrawal)
    /// can redeem the withdrawal to the `recipient`
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
/// This struct represents amount of assets.
pub struct Amount(pub Uint128);

impl Amount {
    /// Validate the amount: [`Uint128`] field.
    /// The amount must be greater than zero.
    pub fn validate(&self, _api: &dyn Api) -> Result<(), VaultError> {
        if self.0.is_zero() {
            return Err(VaultError::zero("Amount cannot be zero"));
        }
        Ok(())
    }
}

/// This struct is used to represent the controller and amount fields together.
#[cw_serde]
pub struct QueueWithdrawalToParams {
    /// the controller is the address that can redeem the withdrawal after the lock period
    pub controller: Addr,
    /// the owner is the address that owns the shares being withdrawn
    pub owner: Addr,
    /// the amount is the amount of shares to be withdrawn
    pub amount: Uint128,
}

impl QueueWithdrawalToParams {
    /// Validate the controller: [`Addr`] and amount: [`Uint128`] fields.
    /// The controller must be a valid [`Addr`], and the amount must be greater than zero.
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        if self.amount.is_zero() {
            return Err(VaultError::zero("Amount cannot be zero"));
        }

        api.addr_validate(self.controller.as_str())?;
        api.addr_validate(self.owner.as_str())?;
        Ok(())
    }
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
            return Err(VaultError::zero("Amount cannot be zero"));
        }

        api.addr_validate(self.recipient.as_str())?;
        Ok(())
    }
}

/// This struct is used to represent a recipient for RedeemWithdrawalTo.
#[cw_serde]
pub struct RedeemWithdrawalToParams {
    pub controller: Addr,
    pub recipient: Addr,
}

impl RedeemWithdrawalToParams {
    /// The recipient must be a valid [`Addr`].
    /// The controller must be a valid [`Addr`].
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        api.addr_validate(self.controller.as_str())?;
        api.addr_validate(self.recipient.as_str())?;
        Ok(())
    }
}

#[cw_serde]
pub struct SetApproveProxyParams {
    /// The proxy address that is being approved.
    pub proxy: Addr,
    /// whether the proxy is approved or not.
    pub approve: bool,
}

impl SetApproveProxyParams {
    /// Validate the proxy: [`Addr`] field.
    /// The proxy must be a valid [`Addr`].
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        api.addr_validate(self.proxy.as_str())?;
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

    /// QueryMsg QueuedWithdrawal: get the queued withdrawal and unlock timestamp under vault.
    #[returns(QueuedWithdrawalResponse)]
    QueuedWithdrawal { controller: String },

    /// QueryMsg VaultInfo: get the vault information.
    #[returns(VaultInfoResponse)]
    VaultInfo {},
}

/// The response to the `Shares` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct SharesResponse(Uint128);

/// The response to the `Assets` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct AssetsResponse(Uint128);

/// The response to the `ConvertToAssets` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct ConvertToAssetsResponse(Uint128);

/// The response to the `ConvertToShares` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct ConvertToSharesResponse(Uint128);

/// The response to the `TotalShares` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct TotalSharesResponse(Uint128);

/// The response to the `TotalAssets` query.
/// This is just a wrapper around `Uint128`, so that the schema can be generated.
#[cw_serde]
pub struct TotalAssetsResponse(Uint128);

/// The response to the `QueuedWithdrawal` query.
/// This is just a wrapper around `QueuedWithdrawalInfo`, so that the schema can be generated.
#[cw_serde]
pub struct QueuedWithdrawalResponse(QueuedWithdrawalInfo);

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

    /// Asset identifier, using the CAIP-19 format.
    pub asset_id: String,

    /// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
    pub asset_type: AssetType,

    /// The asset reference stores the cw20 contract address or the bank denom.
    pub asset_reference: String,

    /// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub contract: String,

    /// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub version: String,
}

#[cw_serde]
pub enum AssetType {
    Cw20,
    Bank,
}
