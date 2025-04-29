use crate::state::{SlashingRequest, SlashingRequestId};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint64};

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub pauser: String,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    /// ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    SetVault {
        vault: String,
        whitelisted: bool,
    },

    /// ExecuteMsg SetWithdrawalLockPeriod the lock period for withdrawal.
    /// Only the `owner` can call this message.
    SetWithdrawalLockPeriod(Uint64),

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    TransferOwnership {
        new_owner: String,
    },

    SlashingRequest(SlashingRequestPayload),
}

#[cw_serde]
pub struct SlashingRequestPayload {
    /// The operator address to slash.
    /// (service, operator) must have active registration at the timestamp.
    pub operator: String,
    /// The percentage of tokens to slash in basis points (1/100th of a percent).
    /// Max bips to slash is set by the service slashing parameters at the timestamp and the operator
    /// must have opted in.
    pub bips: u16,
    /// The timestamp at which the slashing condition occurred.
    pub timestamp: Timestamp,
    /// Additional contextual information about the slashing request.
    pub metadata: SlashingMetadata,
}

#[cw_serde]
pub struct SlashingMetadata {
    /// The reason for the slashing request. Must contain human-readable string.
    pub reason: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// QueryMsg IsWhitelisted: returns true if the vault is whitelisted.
    /// See [`ExecuteMsg::SetVault`]
    #[returns(IsWhitelistedResponse)]
    IsWhitelisted { vault: String },

    /// QueryMsg IsValidating: returns true if the operator is validating services.
    /// See BVS Registry for more information.
    #[returns(IsValidatingResponse)]
    IsValidating { operator: String },

    /// QueryMsg ListVaults: returns a list of vaults.
    /// You can provide `limit` and `start_after` to paginate the results.
    /// The max `limit` is 100.
    #[returns(VaultListResponse)]
    ListVaults {
        limit: Option<u32>,
        start_after: Option<String>,
    },

    /// QueryMsg ListVaultsByOperator: returns a list of vaults managed by given operator.
    /// You can provide `limit` and `start_after` to paginate the results.
    /// The max `limit` is 100.
    #[returns(VaultListResponse)]
    ListVaultsByOperator {
        operator: String,
        limit: Option<u32>,
        start_after: Option<String>,
    },

    /// QueryMsg WithdrawalLockPeriod: returns the withdrawal lock period.
    #[returns(WithdrawalLockPeriodResponse)]
    WithdrawalLockPeriod {},

    #[returns(SlashingRequestIdResponse)]
    SlashingRequestId { service: String, operator: String },

    #[returns(SlashingRequestResponse)]
    SlashingRequest(SlashingRequestId),
}

/// The response to the `IsWhitelisted` query.
/// Not exported.
/// This is just a wrapper around `bool`, so that the schema can be generated.
#[cw_serde]
struct IsWhitelistedResponse(bool);

/// The response to the `IsValidating` query.
/// Not exported.
/// This is just a wrapper around `bool`, so that the schema can be generated.
#[cw_serde]
struct IsValidatingResponse(bool);

/// The response to the `ListVaults` query.
/// For pagination, the `start_after` field is the last `vault` from the previous page.
#[cw_serde]
pub struct VaultListResponse(pub Vec<Vault>);

#[cw_serde]
pub struct Vault {
    pub vault: Addr,
    pub whitelisted: bool,
}

/// The response to the `WithdrawalLockPeriod` query.
/// Not exported.
/// This is just a wrapper around `Uint64`, so that the schema can be generated.
#[cw_serde]
struct WithdrawalLockPeriodResponse(Uint64);

#[cw_serde]
pub struct SlashingRequestIdResponse(pub Option<SlashingRequestId>);

#[cw_serde]
pub struct SlashingRequestResponse(pub Option<SlashingRequest>);
