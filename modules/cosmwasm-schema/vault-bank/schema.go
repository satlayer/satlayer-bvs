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
type ExecuteMsg struct {
	DepositFor *RecipientAmount `json:"deposit_for,omitempty"`
	WithdrawTo *RecipientAmount `json:"withdraw_to,omitempty"`
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
// QueryMsg VaultInfo: get the vault information.
type QueryMsg struct {
	Shares          *Shares          `json:"shares,omitempty"`
	Assets          *Assets          `json:"assets,omitempty"`
	ConvertToAssets *ConvertToAssets `json:"convert_to_assets,omitempty"`
	ConvertToShares *ConvertToShares `json:"convert_to_shares,omitempty"`
	TotalShares     *TotalShares     `json:"total_shares,omitempty"`
	TotalAssets     *TotalAssets     `json:"total_assets,omitempty"`
	VaultInfo       *VaultInfo       `json:"vault_info,omitempty"`
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

type Shares struct {
	Staker string `json:"staker"`
}

type TotalAssets struct {
}

type TotalShares struct {
}

type VaultInfo struct {
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
