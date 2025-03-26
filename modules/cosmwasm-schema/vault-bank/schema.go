// This file was automatically generated from vault-bank/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultbank

type AssetsResponse string

type ConvertToAssetsResponse string

type ConvertToSharesResponse string

type SharesResponse string

type TotalAssetsResponse string

type TotalSharesResponse string

type InstantiateMsg struct {
	// The denom supported by this vault.
	Denom string `json:"denom"`
	// The address of the `operator`. Each vault is delegated to an `operator`.
	Operator string `json:"operator"`
	// The address of the `pauser` contract. See [auth::set_pauser] for more information.
	Pauser string `json:"pauser"`
	// The address of the `router` contract. See [auth::set_router] for more information.
	Router string `json:"router"`
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
type ExecuteMsg struct {
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
type QueryMsg struct {
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

// The response to the `QueuedWithdrawal` query. Not exported. This is just a wrapper around
// `QueuedWithdrawalInfo`, so that the schema can be generated.
type QueuedWithdrawalResponse struct {
	QueuedShares    string `json:"queued_shares"`
	UnlockTimestamp string `json:"unlock_timestamp"`
}

type VaultInfoResponse struct {
	// Asset identifier, using the CAIP-19 format.
	AssetID string `json:"asset_id"`
	// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
	Contract string `json:"contract"`
	// The `operator` that this vault is delegated to
	Operator string `json:"operator"`
	// The `pauser` contract address
	Pauser string `json:"pauser"`
	// The `vault-router` contract address
	Router string `json:"router"`
	// Whether the vault has enabled slashing
	Slashing bool `json:"slashing"`
	// The total assets under management
	TotalAssets string `json:"total_assets"`
	// The total shares in circulation
	TotalShares string `json:"total_shares"`
	// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
	Version string `json:"version"`
}
