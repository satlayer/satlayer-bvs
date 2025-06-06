use crate::state::SlashingRequest;
use bvs_library::slashing::SlashingRequestId;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{from_json, to_json_binary, Addr, Binary, Timestamp, Uint128, Uint64};

#[cw_serde]
pub struct MigrateMsg {
    pub guardrail: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub pauser: String,
    pub guardrail: String,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    /// ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    SetVault { vault: String, whitelisted: bool },

    /// ExecuteMsg SetWithdrawalLockPeriod the lock period for withdrawal.
    /// Only the `owner` can call this message.
    SetWithdrawalLockPeriod(Uint64),

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    TransferOwnership { new_owner: String },

    /// ExecuteMsg RequestSlashing initiates a slashing request against an active operator of the service (info.sender).
    ///
    /// This ExecuteMsg allows a registered service to request a slash of an operator's staked tokens
    /// as a penalty for violations or non-compliance. The slashing request must meet several criteria:
    ///
    /// - The service must be actively registered with the operator at the specified timestamp
    /// - The slashing amount (in bips) must not exceed the max_slashing_bips set by the service
    /// - The operator must have opted in to slashing at the specified timestamp
    /// - The timestamp must be within the allowable slashing window (not too old or in the future)
    /// - The service must not have another active slashing request against the same operator
    /// - The reason provided in metadata must not exceed the maximum allowed length
    ///
    /// When successful, this creates a slashing request with an expiry time based on the
    /// resolution_window parameter and returns a unique slashing request ID.
    ///
    /// #### Returns
    /// On success, returns events with a data field set as [`RequestSlashingResponse`] containing the generated slashing request ID.
    RequestSlashing(RequestSlashingPayload),

    /// ExecuteMsg LockSlashing initiates the movement of slashed collateral from vaults to the
    /// router which will later be finalized and handle according to the service slashing
    /// rules.
    LockSlashing(SlashingRequestId),

    /// ExecuteMsg CancelSlashing cancels a resolved slashing request.
    ///
    /// The service (slash initiator) should cancel the slashing process if the operator
    /// has resolved the issue. The definition of “resolved” is up to the service to define.
    CancelSlashing(SlashingRequestId),

    /// ExecuteMsg FinalizeSlashing moves the slashed collateral from the router to the destination
    /// specified in the slashing parameters that were agreed upon by the service and operator.
    ///
    /// This is the final step in the slashing process
    /// and should only be called after the request has been locked,
    /// and the guardrail proposal has been voted on and passed.
    FinalizeSlashing(SlashingRequestId),
}

#[cw_serde]
pub struct RequestSlashingResponse(pub SlashingRequestId);

impl From<RequestSlashingResponse> for Binary {
    fn from(response: RequestSlashingResponse) -> Self {
        to_json_binary(&response).unwrap()
    }
}

impl From<Binary> for RequestSlashingResponse {
    fn from(binary: Binary) -> Self {
        from_json(&binary).unwrap()
    }
}

#[cw_serde]
pub struct RequestSlashingPayload {
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
    /// The reason for the slashing request.
    /// Must contain human-readable string.
    /// Max length of 250 characters, empty string is allowed but not recommended.
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

    #[returns(SlashingLockedResponse)]
    SlashingLocked {
        slashing_request_id: SlashingRequestId,
    },
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

#[cw_serde]
pub struct SlashingLockedResponse(pub Vec<SlashingLockedResponseItem>);

#[cw_serde]
pub struct SlashingLockedResponseItem {
    pub vault: Addr,
    pub amount: Uint128,
}
