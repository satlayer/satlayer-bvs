// This file was automatically generated from vault-cw20-tokenized/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultcw20tokenized

type Binary string

type InstantiateMsg struct {
	// The address of the `operator`. Each vault is delegated to an `operator`.
	Operator string `json:"operator"`
	// The address of the `pauser` contract.
	Pauser string `json:"pauser"`
	// The vault itself is a CW20 token, which will serve as receipt cw20 token. With extended
	// functionality to be a vault. This field is the cw20 compliant `InstantiateMsg` for the
	// receipt cw20 token.
	ReceiptCw20InstantiateBase ReceiptCw20InstantiateBaseClass `json:"receipt_cw20_instantiate_base"`
	// The address of the `router` contract.
	Router string `json:"router"`
	// The address of the CW20 contract, underlying asset of the vault.
	//
	// ### CW20 Variant Warning
	//
	// Underlying assets that are not strictly CW20 compliant may cause unexpected behavior in
	// token balances. For example, any token with a fee-on-transfer mechanism is not
	// supported.
	//
	// Therefore, we do not support non-standard CW20 tokens. Vault deployed with such tokens
	// will be blacklisted in the vault-router.
	StakingCw20Contract string `json:"staking_cw20_contract"`
}

// The vault itself is a CW20 token, which will serve as receipt cw20 token. With extended
// functionality to be a vault. This field is the cw20 compliant `InstantiateMsg` for the
// receipt cw20 token.
type ReceiptCw20InstantiateBaseClass struct {
	Decimals        int64                     `json:"decimals"`
	InitialBalances []Cw20Coin                `json:"initial_balances"`
	Marketing       *InstantiateMarketingInfo `json:"marketing"`
	Mint            *MinterResponse           `json:"mint"`
	Name            string                    `json:"name"`
	Symbol          string                    `json:"symbol"`
}

type Cw20Coin struct {
	Address string `json:"address"`
	Amount  string `json:"amount"`
}

type InstantiateMarketingInfo struct {
	Description *string    `json:"description"`
	Logo        *LogoClass `json:"logo"`
	Marketing   *string    `json:"marketing"`
	Project     *string    `json:"project"`
}

// A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
//
// Logo content stored on the blockchain. Enforce maximum size of 5KB on all variants
type LogoClass struct {
	URL      *string           `json:"url,omitempty"`
	Embedded *LogoEmbeddedLogo `json:"embedded,omitempty"`
}

// This is used to store the logo on the blockchain in an accepted format. Enforce maximum
// size of 5KB on all variants.
//
// Store the Logo as an SVG file. The content must conform to the spec at
// https://en.wikipedia.org/wiki/Scalable_Vector_Graphics (The contract should do some
// light-weight sanity-check validation)
//
// Store the Logo as a PNG file. This will likely only support up to 64x64 or so within the
// 5KB limit.
type LogoEmbeddedLogo struct {
	SVG *string `json:"svg,omitempty"`
	PNG *string `json:"png,omitempty"`
}

type MinterResponse struct {
	// cap is a hard cap on total supply that can be achieved by minting. Note that this refers
	// to total_supply. If None, there is unlimited cap.
	Cap    *string `json:"cap"`
	Minter string  `json:"minter"`
}

// Supports the same [Cw20ExecuteMsg](cw20_base::msg::ExecuteMsg) as the `cw20-base`
// contract. Cw20 compliant messages are passed to the `cw20-base` contract.
//
// Supports the same [VaultExecuteMsg](bvs_vault_base::msg::VaultExecuteMsg) as the
// `bvs-vault-base` contract.
type ExecuteMsg struct {
	Base     *Cw20ExecuteMsg  `json:"base,omitempty"`
	Extended *VaultExecuteMsg `json:"extended,omitempty"`
}

// Transfer is a base message to move tokens to another account without triggering actions
//
// # Burn is a base message to destroy tokens forever
//
// Send is a base message to transfer tokens to a contract and trigger an action on the
// receiving contract.
//
// Only with "approval" extension. Allows spender to access an additional amount tokens from
// the owner's (env.sender) account. If expires is Some(), overwrites current allowance
// expiration with this one.
//
// Only with "approval" extension. Lowers the spender's access of tokens from the owner's
// (env.sender) account by amount. If expires is Some(), overwrites current allowance
// expiration with this one.
//
// Only with "approval" extension. Transfers amount tokens from owner -> recipient if
// `env.sender` has sufficient pre-approval.
//
// Only with "approval" extension. Sends amount tokens from owner -> contract if
// `env.sender` has sufficient pre-approval.
//
// Only with "approval" extension. Destroys tokens forever
//
// Only with the "mintable" extension. If authorized, creates amount new tokens and adds to
// the recipient balance.
//
// Only with the "mintable" extension. The current minter may set a new minter. Setting the
// minter to None will remove the token's minter forever.
//
// Only with the "marketing" extension. If authorized, updates marketing metadata. Setting
// None/null for any of these will leave it unchanged. Setting Some("") will clear this
// field on the contract storage
//
// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the
// token
type Cw20ExecuteMsg struct {
	Transfer          *Transfer          `json:"transfer,omitempty"`
	Burn              *Burn              `json:"burn,omitempty"`
	Send              *Send              `json:"send,omitempty"`
	IncreaseAllowance *IncreaseAllowance `json:"increase_allowance,omitempty"`
	DecreaseAllowance *DecreaseAllowance `json:"decrease_allowance,omitempty"`
	TransferFrom      *TransferFrom      `json:"transfer_from,omitempty"`
	SendFrom          *SendFrom          `json:"send_from,omitempty"`
	BurnFrom          *BurnFrom          `json:"burn_from,omitempty"`
	Mint              *Mint              `json:"mint,omitempty"`
	UpdateMinter      *UpdateMinter      `json:"update_minter,omitempty"`
	UpdateMarketing   *UpdateMarketing   `json:"update_marketing,omitempty"`
	UploadLogo        *Logo              `json:"upload_logo,omitempty"`
}

type Burn struct {
	Amount string `json:"amount"`
}

type BurnFrom struct {
	Amount string `json:"amount"`
	Owner  string `json:"owner"`
}

type DecreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type Expiration struct {
	AtHeight *int64  `json:"at_height,omitempty"`
	AtTime   *string `json:"at_time,omitempty"`
	Never    *Never  `json:"never,omitempty"`
}

type Never struct {
}

type IncreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

type Mint struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

type Send struct {
	Amount   string `json:"amount"`
	Contract string `json:"contract"`
	Msg      string `json:"msg"`
}

type SendFrom struct {
	Amount   string `json:"amount"`
	Contract string `json:"contract"`
	Msg      string `json:"msg"`
	Owner    string `json:"owner"`
}

type Transfer struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

type TransferFrom struct {
	Amount    string `json:"amount"`
	Owner     string `json:"owner"`
	Recipient string `json:"recipient"`
}

type UpdateMarketing struct {
	// A longer description of the token and it's utility. Designed for tooltips or such
	Description *string `json:"description"`
	// The address (if any) who can update this data structure
	Marketing *string `json:"marketing"`
	// A URL pointing to the project behind this token.
	Project *string `json:"project"`
}

type UpdateMinter struct {
	NewMinter *string `json:"new_minter"`
}

// This is used for uploading logo data, or setting it in InstantiateData
//
// A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
//
// Logo content stored on the blockchain. Enforce maximum size of 5KB on all variants
type Logo struct {
	URL      *string                `json:"url,omitempty"`
	Embedded *LogoEmbeddedLogoClass `json:"embedded,omitempty"`
}

// This is used to store the logo on the blockchain in an accepted format. Enforce maximum
// size of 5KB on all variants.
//
// Store the Logo as an SVG file. The content must conform to the spec at
// https://en.wikipedia.org/wiki/Scalable_Vector_Graphics (The contract should do some
// light-weight sanity-check validation)
//
// Store the Logo as a PNG file. This will likely only support up to 64x64 or so within the
// 5KB limit.
type LogoEmbeddedLogoClass struct {
	SVG *string `json:"svg,omitempty"`
	PNG *string `json:"png,omitempty"`
}

// Vault `ExecuteMsg`, to be implemented by the vault contract. Callable by any `sender`,
// redeemable by any `recipient`. The `sender` can be the same as the `recipient` in some
// cases.
//
// ExecuteMsg Deposit assets into the vault. Sender must transfer the assets to the vault
// contract (this is implementation agnostic). The vault contract must mint shares to the
// `recipient`. Vault must be whitelisted in the `vault-router` to accept deposits.
//
// ExecuteMsg Withdraw assets from the vault. Sender must have enough shares to withdraw the
// requested amount to the `recipient`. If the Vault is delegated to an `operator`,
// withdrawals must be queued. Operator must not be validating any services for instant
// withdrawals.
//
// ExecuteMsg QueueWithdrawal assets from the vault. Sender must have enough shares to queue
// the requested amount to the `recipient`. Once the withdrawal is queued, the `recipient`
// can redeem the withdrawal after the lock period. Once the withdrawal is locked, the
// `sender` cannot cancel the withdrawal. The time-lock is enforced by the vault and cannot
// be changed retroactively.
//
// ### Lock Period Extension New withdrawals will extend the lock period of any existing
// withdrawals. You can queue the withdrawal to a different `recipient` than the `sender` to
// avoid this.
//
// ExecuteMsg RedeemWithdrawal all queued shares into assets from the vault for withdrawal.
// After the lock period, the `sender` (must be the `recipient` of the original withdrawal)
// can redeem the withdrawal.
type VaultExecuteMsg struct {
	DepositFor         *RecipientAmount `json:"deposit_for,omitempty"`
	WithdrawTo         *RecipientAmount `json:"withdraw_to,omitempty"`
	QueueWithdrawalTo  *RecipientAmount `json:"queue_withdrawal_to,omitempty"`
	RedeemWithdrawalTo *string          `json:"redeem_withdrawal_to,omitempty"`
}

// This struct is used to represent the recipient and amount fields together.
type RecipientAmount struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

// Supports the same [VaultQueryMsg](bvs_vault_base::msg::VaultQueryMsg) as the
// `bvs-vault-base` contract.
type QueryMsg struct {
	Base     *QueryMsgClass `json:"base,omitempty"`
	Extended *VaultQueryMsg `json:"extended,omitempty"`
}

// Returns the current balance of the given address, 0 if unset.
//
// Returns metadata on the contract - name, decimals, supply, etc.
//
// Only with "mintable" extension. Returns who can mint and the hard cap on maximum tokens
// after minting.
//
// Only with "allowance" extension. Returns how much spender can use from owner account, 0
// if unset.
//
// Only with "enumerable" extension (and "allowances") Returns all allowances this owner has
// approved. Supports pagination.
//
// Only with "enumerable" extension (and "allowances") Returns all allowances this spender
// has been granted. Supports pagination.
//
// Only with "enumerable" extension Returns all accounts that have balances. Supports
// pagination.
//
// Only with "marketing" extension Returns more metadata on the contract to display in the
// client: - description, logo, project url, etc.
//
// Only with "marketing" extension Downloads the embedded logo data (if stored on chain).
// Errors if no logo data is stored for this contract.
type QueryMsgClass struct {
	Balance              *Balance              `json:"balance,omitempty"`
	TokenInfo            *TokenInfo            `json:"token_info,omitempty"`
	Minter               *Minter               `json:"minter,omitempty"`
	Allowance            *Allowance            `json:"allowance,omitempty"`
	AllAllowances        *AllAllowances        `json:"all_allowances,omitempty"`
	AllSpenderAllowances *AllSpenderAllowances `json:"all_spender_allowances,omitempty"`
	AllAccounts          *AllAccounts          `json:"all_accounts,omitempty"`
	MarketingInfo        *MarketingInfo        `json:"marketing_info,omitempty"`
	DownloadLogo         *DownloadLogo         `json:"download_logo,omitempty"`
}

type AllAccounts struct {
	Limit      *int64  `json:"limit"`
	StartAfter *string `json:"start_after"`
}

type AllAllowances struct {
	Limit      *int64  `json:"limit"`
	Owner      string  `json:"owner"`
	StartAfter *string `json:"start_after"`
}

type AllSpenderAllowances struct {
	Limit      *int64  `json:"limit"`
	Spender    string  `json:"spender"`
	StartAfter *string `json:"start_after"`
}

type Allowance struct {
	Owner   string `json:"owner"`
	Spender string `json:"spender"`
}

type Balance struct {
	Address string `json:"address"`
}

type DownloadLogo struct {
}

type MarketingInfo struct {
}

type Minter struct {
}

type TokenInfo struct {
}

// QueryMsg Shares: get the shares of a staker.
//
// QueryMsg Assets: get the assets of a staker, converted from shares.
//
// QueryMsg ConvertToAssets: convert shares to assets.
//
// QueryMsg ConvertToShares: convert assets to shares.
//
// QueryMsg TotalShares: get the total shares in circulation.
//
// QueryMsg TotalAssets: get the total assets under vault.
//
// QueryMsg QueuedWithdrawal: get the queued withdrawal and unlock timestamp under vault.
//
// QueryMsg VaultInfo: get the vault information.
type VaultQueryMsg struct {
	Shares           *Shares           `json:"shares,omitempty"`
	Assets           *Assets           `json:"assets,omitempty"`
	ConvertToAssets  *ConvertToAssets  `json:"convert_to_assets,omitempty"`
	ConvertToShares  *ConvertToShares  `json:"convert_to_shares,omitempty"`
	TotalShares      *TotalShares      `json:"total_shares,omitempty"`
	TotalAssets      *TotalAssets      `json:"total_assets,omitempty"`
	QueuedWithdrawal *QueuedWithdrawal `json:"queued_withdrawal,omitempty"`
	VaultInfo        *VaultInfo        `json:"vault_info,omitempty"`
}

type Assets struct {
	Staker string `json:"staker"`
}

type ConvertToAssets struct {
	Shares string `json:"shares"`
}

type ConvertToShares struct {
	Assets string `json:"assets"`
}

type QueuedWithdrawal struct {
	Staker string `json:"staker"`
}

type Shares struct {
	Staker string `json:"staker"`
}

type TotalAssets struct {
}

type TotalShares struct {
}

type VaultInfo struct {
}
