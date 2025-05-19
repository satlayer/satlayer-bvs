// This file was automatically generated from vault-router/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * The response to the `IsValidating` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 *
 * The response to the `IsWhitelisted` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 */
type IsValidatingResponse = boolean;

/**
 * The response to the `IsValidating` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 *
 * The response to the `IsWhitelisted` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 */
type IsWhitelistedResponse = boolean;

/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the
 * full u64 range can be used for clients that convert JSON numbers to floats, like
 * JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 *
 * The timestamp at which the slashing condition occurred.
 *
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202);
 * assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1);
 * assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202);
 * assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 *
 * SlashingRequestId stores the id in hexbinary. It's a 32-byte hash of the slashing
 * request
 *
 * This is a wrapper around Vec<u8> to add hex de/serialization with serde. It also adds
 * some helper methods to help encode inline.
 *
 * This is similar to `cosmwasm_std::Binary` but uses hex. See also
 * <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 *
 * A human readable address.
 *
 * In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no
 * assumptions should be made other than being UTF-8 encoded and of reasonable length.
 *
 * This type represents a validated address. It can be created in the following ways 1. Use
 * `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3.
 * Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from
 * JSON. This must only be done from JSON that was validated before such as a contract's
 * state. `Addr` must not be used in messages sent by the user because this would result in
 * unvalidated instances.
 *
 * This type is immutable. If you really need to mutate it (Really? Are you sure?), create a
 * mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String`
 * instance.
 *
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that
 * the full u128 range can be used for clients that convert JSON numbers to floats, like
 * JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(),
 * 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 *
 * The timestamp after which the request is no longer valid. This will be `request_time` +
 * `resolution_window` * 2 (as per current slashing parameters)
 *
 * The timestamp when the request resolution window will end and becomes eligible for
 * locking. This will be `request_time` + `resolution_window`
 *
 * The timestamp when the request was submitted.
 *
 * The service that initiated the slashing request.
 *
 * The response to the `WithdrawalLockPeriod` query. Not exported. This is just a wrapper
 * around `Uint64`, so that the schema can be generated.
 */
type WithdrawalLockPeriodResponse = string;

export interface InstantiateMsg {
  guardrail: string;
  owner: string;
  pauser: string;
  registry: string;
}

/**
 * ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it. Only
 * the `owner` can call this message.
 *
 * ExecuteMsg SetWithdrawalLockPeriod the lock period for withdrawal. Only the `owner` can
 * call this message.
 *
 * ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
 * information on this field
 *
 * ExecuteMsg RequestSlashing initiates a slashing request against an active operator of the
 * service (info.sender).
 *
 * This ExecuteMsg allows a registered service to request a slash of an operator's staked
 * tokens as a penalty for violations or non-compliance. The slashing request must meet
 * several criteria:
 *
 * - The service must be actively registered with the operator at the specified timestamp -
 * The slashing amount (in bips) must not exceed the max_slashing_bips set by the service -
 * The operator must have opted in to slashing at the specified timestamp - The timestamp
 * must be within the allowable slashing window (not too old or in the future) - The service
 * must not have another active slashing request against the same operator - The reason
 * provided in metadata must not exceed the maximum allowed length
 *
 * When successful, this creates a slashing request with an expiry time based on the
 * resolution_window parameter and returns a unique slashing request ID.
 *
 * #### Returns On success, returns events with a data field set as
 * [`RequestSlashingResponse`] containing the generated slashing request ID.
 *
 * ExecuteMsg LockSlashing initiates the movement of slashed collateral from vaults to the
 * router which will later be finalized and handle according to the service slashing rules.
 *
 * ExecuteMsg CancelSlashing cancels a resolved slashing request.
 *
 * The service (slash initiator) should cancel the slashing process if the operator has
 * resolved the issue. The definition of “resolved” is up to the service to define.
 *
 * ExecuteMsg FinalizeSlashing moves the slashed collateral from the router to the
 * destination specified in the slashing parameters that were agreed upon by the service and
 * operator.
 *
 * This is the final step in the slashing process and should only be called after the
 * request has been locked, and the guardrail proposal has been voted on and passed.
 */
export interface ExecuteMsg {
  set_vault?: SetVault;
  set_withdrawal_lock_period?: string;
  transfer_ownership?: TransferOwnership;
  request_slashing?: RequestSlashingClass;
  lock_slashing?: string;
  cancel_slashing?: string;
  finalize_slashing?: string;
}

export interface RequestSlashingClass {
  /**
   * The percentage of tokens to slash in basis points (1/100th of a percent). Max bips to
   * slash is set by the service slashing parameters at the timestamp and the operator must
   * have opted in.
   */
  bips: number;
  /**
   * Additional contextual information about the slashing request.
   */
  metadata: RequestSlashingMetadata;
  /**
   * The operator address to slash. (service, operator) must have active registration at the
   * timestamp.
   */
  operator: string;
  /**
   * The timestamp at which the slashing condition occurred.
   */
  timestamp: string;
}

/**
 * Additional contextual information about the slashing request.
 */
export interface RequestSlashingMetadata {
  /**
   * The reason for the slashing request. Must contain human-readable string. Max length of
   * 250 characters, empty string is allowed but not recommended.
   */
  reason: string;
}

export interface SetVault {
  vault: string;
  whitelisted: boolean;
}

export interface TransferOwnership {
  new_owner: string;
}

/**
 * QueryMsg IsWhitelisted: returns true if the vault is whitelisted. See
 * [`ExecuteMsg::SetVault`]
 *
 * QueryMsg IsValidating: returns true if the operator is validating services. See BVS
 * Registry for more information.
 *
 * QueryMsg ListVaults: returns a list of vaults. You can provide `limit` and `start_after`
 * to paginate the results. The max `limit` is 100.
 *
 * QueryMsg ListVaultsByOperator: returns a list of vaults managed by given operator. You
 * can provide `limit` and `start_after` to paginate the results. The max `limit` is 100.
 *
 * QueryMsg WithdrawalLockPeriod: returns the withdrawal lock period.
 */
export interface QueryMsg {
  is_whitelisted?: IsWhitelisted;
  is_validating?: IsValidating;
  list_vaults?: ListVaults;
  list_vaults_by_operator?: ListVaultsByOperator;
  withdrawal_lock_period?: WithdrawalLockPeriod;
  slashing_request_id?: SlashingRequestID;
  slashing_request?: string;
  slashing_locked?: SlashingLocked;
}

export interface IsValidating {
  operator: string;
}

export interface IsWhitelisted {
  vault: string;
}

export interface ListVaults {
  limit?: number | null;
  start_after?: null | string;
}

export interface ListVaultsByOperator {
  limit?: number | null;
  operator: string;
  start_after?: null | string;
}

export interface SlashingLocked {
  slashing_request_id: string;
}

export interface SlashingRequestID {
  operator: string;
  service: string;
}

export interface WithdrawalLockPeriod {}

/**
 * The response to the `ListVaults` query. For pagination, the `start_after` field is the
 * last `vault` from the previous page.
 */
export interface VaultListResponse {
  vault: string;
  whitelisted: boolean;
}

export interface SlashingLockedResponse {
  amount: string;
  vault: string;
}

export interface SlashingRequestResponse {
  /**
   * The core slashing request data including operator, bips, timestamp, and metadata.
   */
  request: RequestClass;
  /**
   * The timestamp after which the request is no longer valid. This will be `request_time` +
   * `resolution_window` * 2 (as per current slashing parameters)
   */
  request_expiry: string;
  /**
   * The timestamp when the request resolution window will end and becomes eligible for
   * locking. This will be `request_time` + `resolution_window`
   */
  request_resolution: string;
  /**
   * The timestamp when the request was submitted.
   */
  request_time: string;
  /**
   * The service that initiated the slashing request.
   */
  service: string;
  /**
   * The status of the slashing request.
   */
  status: number;
}

/**
 * The core slashing request data including operator, bips, timestamp, and metadata.
 */
export interface RequestClass {
  /**
   * The percentage of tokens to slash in basis points (1/100th of a percent). Max bips to
   * slash is set by the service slashing parameters at the timestamp and the operator must
   * have opted in.
   */
  bips: number;
  /**
   * Additional contextual information about the slashing request.
   */
  metadata: RequestMetadata;
  /**
   * The operator address to slash. (service, operator) must have active registration at the
   * timestamp.
   */
  operator: string;
  /**
   * The timestamp at which the slashing condition occurred.
   */
  timestamp: string;
}

/**
 * Additional contextual information about the slashing request.
 */
export interface RequestMetadata {
  /**
   * The reason for the slashing request. Must contain human-readable string. Max length of
   * 250 characters, empty string is allowed but not recommended.
   */
  reason: string;
}
