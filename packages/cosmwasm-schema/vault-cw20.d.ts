// This file was automatically generated from vault-cw20/schema.json.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
 * The response to the `Assets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `ConvertToShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `Shares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalAssets` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
 *
 * The response to the `TotalShares` query. Not exported. This is just a wrapper around
 * `Uint128`, so that the schema can be generated.
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
   * The address of the CW20 contract, underlying asset of the vault.
   */
  cw20_contract: string;
  /**
   * The address of the `operator`. Each vault is delegated to an `operator`.
   */
  operator: string;
  /**
   * The address of the `pauser` contract.
   */
  pauser: string;
  /**
   * The address of the `router` contract.
   */
  router: string;
}

/**
 * Vault `ExecuteMsg`, to be implemented by the vault contract. Callable by any `sender`,
 * redeemable by any `recipient`. The `sender` can be the same as the `recipient` in some
 * cases.
 *
 * ExecuteMsg Deposit assets into the vault. Sender must transfer the assets to the vault
 * contract (this is implementation agnostic). The vault contract must mint shares to the
 * `recipient`. Vault must be whitelisted in the `vault-router` to accept deposits.
 *
 * ExecuteMsg Withdraw assets from the vault. Sender must have enough shares to withdraw the
 * requested amount to the `recipient`. If the Vault is delegated to an `operator`,
 * withdrawals must be queued. Operator must not be validating any services for instant
 * withdrawals.
 */
export interface ExecuteMsg {
  deposit_for?: RecipientAmount;
  withdraw_to?: RecipientAmount;
}

/**
 * This struct is used to represent the recipient and amount fields together.
 */
export interface RecipientAmount {
  amount: string;
  recipient: string;
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
 * QueryMsg VaultInfo: get the vault information.
 */
export interface QueryMsg {
  shares?: Shares;
  assets?: Assets;
  convert_to_assets?: ConvertToAssets;
  convert_to_shares?: ConvertToShares;
  total_shares?: TotalShares;
  total_assets?: TotalAssets;
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

export interface Shares {
  staker: string;
}

export interface TotalAssets {}

export interface TotalShares {}

export interface VaultInfo {}

export interface VaultInfoResponse {
  /**
   * Asset identifier, using the CAIP-19 format.
   */
  asset_id: string;
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
   * Whether the vault has enabled slashing
   */
  slashing: boolean;
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
