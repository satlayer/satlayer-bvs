// This file was automatically generated from vault-cw20-tokenized/schema.json.
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
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also
 * adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See
 * also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
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
 * This struct is used to represent a recipient for RedeemWithdrawalTo.
 */
type Binary = string;

export interface InstantiateMsg {
  /**
   * The address of the `operator`. Each vault is delegated to an `operator`.
   */
  operator: string;
  /**
   * The address of the `pauser` contract.
   */
  pauser: string;
  /**
   * The vault itself is a CW20 token, which will serve as receipt cw20 token. With extended
   * functionality to be a vault. This field is the cw20 compliant `InstantiateMsg` for the
   * receipt cw20 token.
   */
  receipt_cw20_instantiate_base: ReceiptCw20InstantiateBaseClass;
  /**
   * The address of the `router` contract.
   */
  router: string;
  /**
   * The address of the CW20 contract, underlying asset of the vault.
   *
   * ### CW20 Variant Warning
   *
   * Underlying assets that are not strictly CW20 compliant may cause unexpected behavior in
   * token balances. For example, any token with a fee-on-transfer mechanism is not
   * supported.
   *
   * Therefore, we do not support non-standard CW20 tokens. Vault deployed with such tokens
   * will be blacklisted in the vault-router.
   */
  staking_cw20_contract: string;
}

/**
 * The vault itself is a CW20 token, which will serve as receipt cw20 token. With extended
 * functionality to be a vault. This field is the cw20 compliant `InstantiateMsg` for the
 * receipt cw20 token.
 */
export interface ReceiptCw20InstantiateBaseClass {
  decimals: number;
  initial_balances: Cw20Coin[];
  marketing?: InstantiateMarketingInfo | null;
  mint?: MinterResponse | null;
  name: string;
  symbol: string;
}

export interface Cw20Coin {
  address: string;
  amount: string;
}

export interface InstantiateMarketingInfo {
  description?: null | string;
  logo?: LogoClass | null;
  marketing?: null | string;
  project?: null | string;
}

/**
 * A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
 *
 * Logo content stored on the blockchain. Enforce maximum size of 5KB on all variants
 */
export interface LogoClass {
  url?: string;
  embedded?: LogoEmbeddedLogo;
}

/**
 * This is used to store the logo on the blockchain in an accepted format. Enforce maximum
 * size of 5KB on all variants.
 *
 * Store the Logo as an SVG file. The content must conform to the spec at
 * https://en.wikipedia.org/wiki/Scalable_Vector_Graphics (The contract should do some
 * light-weight sanity-check validation)
 *
 * Store the Logo as a PNG file. This will likely only support up to 64x64 or so within the
 * 5KB limit.
 */
export interface LogoEmbeddedLogo {
  svg?: string;
  png?: string;
}

export interface MinterResponse {
  /**
   * cap is a hard cap on total supply that can be achieved by minting. Note that this refers
   * to total_supply. If None, there is unlimited cap.
   */
  cap?: null | string;
  minter: string;
}

/**
 * Supports the same [Cw20ExecuteMsg](cw20_base::msg::ExecuteMsg) as the `cw20-base`
 * contract. Cw20 compliant messages are passed to the `cw20-base` contract.
 *
 * Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the
 * `bvs-vault-base` contract.
 */
export interface ExecuteMsg {
  base?: Cw20ExecuteMsg;
  extended?: VaultExecuteMsg;
}

/**
 * Transfer is a base message to move tokens to another account without triggering actions
 *
 * Burn is a base message to destroy tokens forever
 *
 * Send is a base message to transfer tokens to a contract and trigger an action on the
 * receiving contract.
 *
 * Only with "approval" extension. Allows spender to access an additional amount tokens from
 * the owner's (env.sender) account. If expires is Some(), overwrites current allowance
 * expiration with this one.
 *
 * Only with "approval" extension. Lowers the spender's access of tokens from the owner's
 * (env.sender) account by amount. If expires is Some(), overwrites current allowance
 * expiration with this one.
 *
 * Only with "approval" extension. Transfers amount tokens from owner -> recipient if
 * `env.sender` has sufficient pre-approval.
 *
 * Only with "approval" extension. Sends amount tokens from owner -> contract if
 * `env.sender` has sufficient pre-approval.
 *
 * Only with "approval" extension. Destroys tokens forever
 *
 * Only with the "mintable" extension. If authorized, creates amount new tokens and adds to
 * the recipient balance.
 *
 * Only with the "mintable" extension. The current minter may set a new minter. Setting the
 * minter to None will remove the token's minter forever.
 *
 * Only with the "marketing" extension. If authorized, updates marketing metadata. Setting
 * None/null for any of these will leave it unchanged. Setting Some("") will clear this
 * field on the contract storage
 *
 * If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the
 * token
 */
export interface Cw20ExecuteMsg {
  transfer?: Transfer;
  burn?: Burn;
  send?: Send;
  increase_allowance?: IncreaseAllowance;
  decrease_allowance?: DecreaseAllowance;
  transfer_from?: TransferFrom;
  send_from?: SendFrom;
  burn_from?: BurnFrom;
  mint?: Mint;
  update_minter?: UpdateMinter;
  update_marketing?: UpdateMarketing;
  upload_logo?: Logo;
}

export interface Burn {
  amount: string;
}

export interface BurnFrom {
  amount: string;
  owner: string;
}

export interface DecreaseAllowance {
  amount: string;
  expires?: Expiration | null;
  spender: string;
}

/**
 * AtHeight will expire when `env.block.height` >= height
 *
 * AtTime will expire when `env.block.time` >= time
 *
 * Never will never expire. Used to express the empty variant
 */
export interface Expiration {
  at_height?: number;
  at_time?: string;
  never?: Never;
}

export interface Never {}

export interface IncreaseAllowance {
  amount: string;
  expires?: Expiration | null;
  spender: string;
}

export interface Mint {
  amount: string;
  recipient: string;
}

export interface Send {
  amount: string;
  contract: string;
  msg: string;
}

export interface SendFrom {
  amount: string;
  contract: string;
  msg: string;
  owner: string;
}

export interface Transfer {
  amount: string;
  recipient: string;
}

export interface TransferFrom {
  amount: string;
  owner: string;
  recipient: string;
}

export interface UpdateMarketing {
  /**
   * A longer description of the token and it's utility. Designed for tooltips or such
   */
  description?: null | string;
  /**
   * The address (if any) who can update this data structure
   */
  marketing?: null | string;
  /**
   * A URL pointing to the project behind this token.
   */
  project?: null | string;
}

export interface UpdateMinter {
  new_minter?: null | string;
}

/**
 * This is used for uploading logo data, or setting it in InstantiateData
 *
 * A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
 *
 * Logo content stored on the blockchain. Enforce maximum size of 5KB on all variants
 */
export interface Logo {
  url?: string;
  embedded?: LogoEmbeddedLogoClass;
}

/**
 * This is used to store the logo on the blockchain in an accepted format. Enforce maximum
 * size of 5KB on all variants.
 *
 * Store the Logo as an SVG file. The content must conform to the spec at
 * https://en.wikipedia.org/wiki/Scalable_Vector_Graphics (The contract should do some
 * light-weight sanity-check validation)
 *
 * Store the Logo as a PNG file. This will likely only support up to 64x64 or so within the
 * 5KB limit.
 */
export interface LogoEmbeddedLogoClass {
  svg?: string;
  png?: string;
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
 *
 * ExecuteMsg QueueWithdrawal assets from the vault. Sender must have enough shares to queue
 * the requested amount to the `recipient`. Once the withdrawal is queued, the `recipient`
 * can redeem the withdrawal after the lock period. Once the withdrawal is locked, the
 * `sender` cannot cancel the withdrawal. The time-lock is enforced by the vault and cannot
 * be changed retroactively.
 *
 * ### Lock Period Extension New withdrawals will extend the lock period of any existing
 * withdrawals. You can queue the withdrawal to a different `recipient` than the `sender` to
 * avoid this.
 *
 * ExecuteMsg RedeemWithdrawal all queued shares into assets from the vault for withdrawal.
 * After the lock period, the `sender` (must be the `recipient` of the original withdrawal)
 * can redeem the withdrawal.
 */
export interface VaultExecuteMsg {
  deposit_for?: RecipientAmount;
  withdraw_to?: RecipientAmount;
  queue_withdrawal_to?: RecipientAmount;
  redeem_withdrawal_to?: string;
}

/**
 * This struct is used to represent the recipient and amount fields together.
 */
export interface RecipientAmount {
  amount: string;
  recipient: string;
}

/**
 * Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the
 * `bvs-vault-base` contract.
 */
export interface QueryMsg {
  base?: QueryMsgClass;
  extended?: VaultQueryMsg;
}

/**
 * Returns the current balance of the given address, 0 if unset.
 *
 * Returns metadata on the contract - name, decimals, supply, etc.
 *
 * Only with "mintable" extension. Returns who can mint and the hard cap on maximum tokens
 * after minting.
 *
 * Only with "allowance" extension. Returns how much spender can use from owner account, 0
 * if unset.
 *
 * Only with "enumerable" extension (and "allowances") Returns all allowances this owner has
 * approved. Supports pagination.
 *
 * Only with "enumerable" extension (and "allowances") Returns all allowances this spender
 * has been granted. Supports pagination.
 *
 * Only with "enumerable" extension Returns all accounts that have balances. Supports
 * pagination.
 *
 * Only with "marketing" extension Returns more metadata on the contract to display in the
 * client: - description, logo, project url, etc.
 *
 * Only with "marketing" extension Downloads the embedded logo data (if stored on chain).
 * Errors if no logo data is stored for this contract.
 */
export interface QueryMsgClass {
  balance?: Balance;
  token_info?: TokenInfo;
  minter?: Minter;
  allowance?: Allowance;
  all_allowances?: AllAllowances;
  all_spender_allowances?: AllSpenderAllowances;
  all_accounts?: AllAccounts;
  marketing_info?: MarketingInfo;
  download_logo?: DownloadLogo;
}

export interface AllAccounts {
  limit?: number | null;
  start_after?: null | string;
}

export interface AllAllowances {
  limit?: number | null;
  owner: string;
  start_after?: null | string;
}

export interface AllSpenderAllowances {
  limit?: number | null;
  spender: string;
  start_after?: null | string;
}

export interface Allowance {
  owner: string;
  spender: string;
}

export interface Balance {
  address: string;
}

export interface DownloadLogo {}

export interface MarketingInfo {}

export interface Minter {}

export interface TokenInfo {}

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
export interface VaultQueryMsg {
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
  staker: string;
}

export interface Shares {
  staker: string;
}

export interface TotalAssets {}

export interface TotalShares {}

export interface VaultInfo {}
