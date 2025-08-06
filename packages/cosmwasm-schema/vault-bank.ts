// This file was automatically generated from vault-bank/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type AssetsResponse = string;

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type ConvertToAssetsResponse = string;

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type ConvertToSharesResponse = string;

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type SharesResponse = string;

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type TotalAssetsResponse = string;

/**
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
 * the amount is the amount of shares to be withdrawn
 *
 * This struct represents amount of assets.
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
 * the controller is the address that can redeem the withdrawal after the lock period
 *
 * the owner is the address that owns the shares being withdrawn
 *
 * The proxy address that is being approved.
 *
 * The response to the `Assets` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `ConvertToAssets` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. This is just a wrapper around `Uint128`, so
 * that the schema can be generated.
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
 * The response to the `Shares` query. This is just a wrapper around `Uint128`, so that the
 * schema can be generated.
 *
 * The response to the `TotalAssets` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The response to the `TotalShares` query. This is just a wrapper around `Uint128`, so that
 * the schema can be generated.
 *
 * The `operator` that this vault is delegated to
 *
 * The `pauser` contract address
 *
 * The `vault-router` contract address
 *
 * The total assets under management
 *
 * The total shares in circulation
 */
type TotalSharesResponse = string;

export interface InstantiateMsg {
  /**
   * The denom supported by this vault.
   */
  denom: string;
  /**
   * The address of the `operator`. Each vault is delegated to an `operator`.
   */
  operator: string;
  /**
   * The address of the `pauser` contract. See [auth::set_pauser] for more information.
   */
  pauser: string;
  /**
   * The address of the `router` contract. See [auth::set_router] for more information.
   */
  router: string;
}

/**
 * Vault `ExecuteMsg`, to be implemented by the vault contract. Callable by any `sender`,
 * redeemable by any `recipient`. The `sender` can be the same as the `recipient` in some
 * cases.
 *
 * ExecuteMsg DepositFor assets into the vault. Sender must transfer the assets to the vault
 * contract (this is implementation agnostic). The vault contract must mint shares to the
 * `recipient`. Vault must be whitelisted in the `vault-router` to accept deposits.
 *
 * ExecuteMsg QueueWithdrawalTo assets from the vault. Sender must have enough shares to
 * queue the requested amount to the `controller`. Once the withdrawal is queued, the
 * `controller` can redeem the withdrawal after the lock period. Once the withdrawal is
 * locked, the `sender` cannot cancel the withdrawal. The time-lock is enforced by the vault
 * and cannot be changed retroactively.
 *
 * ### Lock Period Extension New withdrawals will extend the lock period of any existing
 * withdrawals. You can queue the withdrawal to a different `controller` than the `sender`
 * to avoid this.
 *
 * ExecuteMsg RedeemWithdrawalTo all queued shares into assets from the vault for
 * withdrawal. After the lock period, the `sender` (must be the `controller` of the original
 * withdrawal) can redeem the withdrawal to the `recipient`
 *
 * ExecuteMsg SlashLocked moves the assets from the vault to the `vault-router` contract for
 * custody. Part of the [https://build.satlayer.xyz/architecture/slashing](Programmable
 * Slashing) lifecycle. This function can only be called by `vault-router`, and takes an
 * absolute `amount` of assets to be moved. The amount is calculated and enforced by the
 * router. Further utility of the assets, post-locked, is implemented and enforced on the
 * router level.
 *
 * ExecuteMsg ApproveProxy allows the `proxy` to queue withdrawal and redeem withdrawal on
 * behalf of the `owner`.
 */
export interface ExecuteMsg {
  deposit_for?: RecipientAmount;
  queue_withdrawal_to?: QueueWithdrawalToParams;
  redeem_withdrawal_to?: RedeemWithdrawalToParams;
  slash_locked?: string;
  set_approve_proxy?: SetApproveProxyParams;
}

/**
 * This struct is used to represent the recipient and amount fields together.
 */
export interface RecipientAmount {
  amount: string;
  recipient: string;
}

/**
 * This struct is used to represent the controller and amount fields together.
 */
export interface QueueWithdrawalToParams {
  /**
   * the amount is the amount of shares to be withdrawn
   */
  amount: string;
  /**
   * the controller is the address that can redeem the withdrawal after the lock period
   */
  controller: string;
  /**
   * the owner is the address that owns the shares being withdrawn
   */
  owner: string;
}

/**
 * This struct is used to represent a recipient for RedeemWithdrawalTo.
 */
export interface RedeemWithdrawalToParams {
  controller: string;
  recipient: string;
}

export interface SetApproveProxyParams {
  /**
   * whether the proxy is approved or not.
   */
  approve: boolean;
  /**
   * The proxy address that is being approved.
   */
  proxy: string;
}

/**
 * QueryMsg Shares: get the shares of a staker.
 *
 * QueryMsg Assets: get the assets of a staker, converted from shares.
 *
 * QueryMsg ConvertToAssets: convert shares to assets.
 *
 * QueryMsg ConvertToShares: convert assets to shares.
 *
 * QueryMsg TotalShares: get the total shares in circulation.
 *
 * QueryMsg TotalAssets: get the total assets under vault.
 *
 * QueryMsg QueuedWithdrawal: get the queued withdrawal and unlock timestamp under vault.
 *
 * QueryMsg VaultInfo: get the vault information.
 */
export interface QueryMsg {
  shares?: Shares;
  assets?: Assets;
  convert_to_assets?: ConvertToAssets;
  convert_to_shares?: ConvertToShares;
  total_shares?: TotalShares;
  total_assets?: TotalAssets;
  queued_withdrawal?: QueuedWithdrawal;
  vault_info?: VaultInfo;
}

export interface Assets {
  staker: string;
}

export interface ConvertToAssets {
  shares: string;
}

export interface ConvertToShares {
  assets: string;
}

export interface QueuedWithdrawal {
  controller: string;
}

export interface Shares {
  staker: string;
}

export interface TotalAssets {}

export interface TotalShares {}

export interface VaultInfo {}

/**
 * The response to the `QueuedWithdrawal` query. This is just a wrapper around
 * `QueuedWithdrawalInfo`, so that the schema can be generated.
 */
export interface QueuedWithdrawalResponse {
  queued_shares: string;
  unlock_timestamp: string;
}

export interface VaultInfoResponse {
  /**
   * Asset identifier, using the CAIP-19 format.
   */
  asset_id: string;
  /**
   * The asset reference stores the cw20 contract address or the bank denom.
   */
  asset_reference: string;
  /**
   * The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
   */
  asset_type: AssetType;
  /**
   * The name of the vault contract, see [`cw2::set_contract_version`] for more information.
   */
  contract: string;
  /**
   * The `operator` that this vault is delegated to
   */
  operator: string;
  /**
   * The `pauser` contract address
   */
  pauser: string;
  /**
   * The `vault-router` contract address
   */
  router: string;
  /**
   * The total assets under management
   */
  total_assets: string;
  /**
   * The total shares in circulation
   */
  total_shares: string;
  /**
   * The version of the vault contract, see [`cw2::set_contract_version`] for more information.
   */
  version: string;
}

/**
 * The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
 */
export enum AssetType {
  Bank = "bank",
  Cw20 = "cw20",
}
