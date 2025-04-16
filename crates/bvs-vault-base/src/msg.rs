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

    /// ExecuteMsg TransferCustody release the right to be custodian of all or a portion of asset.
    /// In the event of an operator has been convicted of a crime, slash verdict has reached,
    /// The asset will be slashed.
    /// Asset being slashed will be sent to the jail address.
    /// The operator no longer has access to the slashed assets.
    /// The slashed assets will then be decided to be burned or allocated for further utility by
    /// the jail contract.
    /// The handling of the slashed asset should not concern the vault contract.
    TransferAssetCustody(JailDetail),

    /// ExecuteMsg SystemLockAsset size the asset by absolute amount.
    /// This message differs from the `TransferAssetCustody` message in that
    /// the asset is sized by absolute amount.
    /// The asset is moved to the router contract, instead of supplied arbitrary contract.
    /// The asset amount is determined by the router base on strategy.
    /// Callable by the router contract only.
    SystemLockAsset(LockAmount),

    /// ExecuteMsg Pause the vault contract.
    SetSlashable(bool),
}

#[cw_serde]
pub struct LockAmount(pub Uint128);

impl LockAmount {
    /// Validate the amount: [`Uint128`] field.
    /// The amount must be greater than zero.
    pub fn validate(&self, _api: &dyn Api) -> Result<(), VaultError> {
        if self.0.is_zero() {
            return Err(VaultError::zero("Amount cannot be zero."));
        }
        if self.0 < Uint128::zero() {
            return Err(VaultError::unauthorized("Amount cannot be negative."));
        }
        Ok(())
    }
}

#[cw_serde]
pub struct JailDetail {
    /// The percentage of asset to be seized.
    pub percentage: u64,

    /// The address that will receive the slashed asset.
    /// The address is recommended to be a contract that will handle what to do with slashed asset.
    /// The jail contract address will either simply burn
    /// or allocate for further utility is up to the third party project.
    pub jail_address: Addr,
}

impl JailDetail {
    /// Validate the percentage and jail address.
    /// The percentage must be between 0 and 100.
    /// The jail address must be a valid [`Addr`].
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        if self.percentage > 100 {
            return Err(VaultError::unauthorized(
                "Percentage must be between 0 and 100.",
            ));
        }

        if self.percentage == 0 {
            return Err(VaultError::zero("Percentage must be greater than 0."));
        }

        api.addr_validate(self.jail_address.as_str())?;

        // logical to also check if the percentage is not under 0.
        // But since percentage type is u64, it is not possible to be negative.
        // will fail at parsing.

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
            return Err(VaultError::zero("Amount cannot be zero."));
        }

        api.addr_validate(self.recipient.as_str())?;
        Ok(())
    }
}

/// This struct is used to represent a recipient for RedeemWithdrawalTo.
#[cw_serde]
pub struct Recipient(pub Addr);

impl Recipient {
    /// Validate the recipient: [`Addr`] field.
    /// The recipient must be a valid [`Addr`].
    pub fn validate(&self, api: &dyn Api) -> Result<(), VaultError> {
        api.addr_validate(self.0.as_str())?;
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
    QueuedWithdrawal { staker: String },

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

/// The response to the `QueuedWithdrawal` query.
///  Not exported.
/// This is just a wrapper around `QueuedWithdrawalInfo`, so that the schema can be generated.
#[cw_serde]
struct QueuedWithdrawalResponse(QueuedWithdrawalInfo);

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
