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
 * The response to the `WithdrawalLockPeriod` query. Not exported. This is just a wrapper
 * around `Uint64`, so that the schema can be generated.
 */
type WithdrawalLockPeriodResponse = string;

export interface InstantiateMsg {
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
 */
export interface ExecuteMsg {
  set_vault?: SetVault;
  set_withdrawal_lock_period?: string;
  transfer_ownership?: TransferOwnership;
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
 * QueryMsg WithdrawalLockPeriod: returns the withdrawal lock period.
 */
export interface QueryMsg {
  is_whitelisted?: IsWhitelisted;
  is_validating?: IsValidating;
  list_vaults?: ListVaults;
  withdrawal_lock_period?: WithdrawalLockPeriod;
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

export interface WithdrawalLockPeriod {}

/**
 * The response to the `ListVaults` query. For pagination, the `start_after` field is the
 * last `vault` from the previous page.
 */
export interface VaultListResponse {
  vault: string;
  whitelisted: boolean;
}
