// This file was automatically generated from vault-cw20/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultcw20

type AssetsResponse string

type ConvertToAssetsResponse string

type ConvertToSharesResponse string

type SharesResponse string

type TotalAssetsResponse string

type TotalSharesResponse string

type InstantiateMsg struct {
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
	Cw20Contract string `json:"cw20_contract"`
	// The address of the `operator`. Each vault is delegated to an `operator`.
	Operator string `json:"operator"`
	// The address of the `pauser` contract.
	Pauser string `json:"pauser"`
	// The address of the `router` contract.
	Router string `json:"router"`
}

// Vault `ExecuteMsg`, to be implemented by the vault contract. Callable by any `sender`,
// redeemable by any `recipient`. The `sender` can be the same as the `recipient` in some
// cases.
//
// ExecuteMsg DepositFor assets into the vault. Sender must transfer the assets to the vault
// contract (this is implementation agnostic). The vault contract must mint shares to the
// `recipient`. Vault must be whitelisted in the `vault-router` to accept deposits.
//
// ExecuteMsg QueueWithdrawalTo assets from the vault. Sender must have enough shares to
// queue the requested amount to the `controller`. Once the withdrawal is queued, the
// `controller` can redeem the withdrawal after the lock period. Once the withdrawal is
// locked, the `sender` cannot cancel the withdrawal. The time-lock is enforced by the vault
// and cannot be changed retroactively.
//
// ### Lock Period Extension New withdrawals will extend the lock period of any existing
// withdrawals. You can queue the withdrawal to a different `controller` than the `sender`
// to avoid this.
//
// ExecuteMsg RedeemWithdrawalTo all queued shares into assets from the vault for
// withdrawal. After the lock period, the `sender` (must be the `controller` of the original
// withdrawal) can redeem the withdrawal to the `recipient`
//
// ExecuteMsg SlashLocked moves the assets from the vault to the `vault-router` contract for
// custody. Part of the [https://build.satlayer.xyz/getting-started/slashing](Programmable
// Slashing) lifecycle. This function can only be called by `vault-router`, and takes an
// absolute `amount` of assets to be moved. The amount is calculated and enforced by the
// router. Further utility of the assets, post-locked, is implemented and enforced on the
// router level.
//
// ExecuteMsg ApproveProxy allows the `proxy` to queue withdrawal and redeem withdrawal on
// behalf of the `owner`.
type ExecuteMsg struct {
	DepositFor         *RecipientAmount          `json:"deposit_for,omitempty"`
	QueueWithdrawalTo  *QueueWithdrawalToParams  `json:"queue_withdrawal_to,omitempty"`
	RedeemWithdrawalTo *RedeemWithdrawalToParams `json:"redeem_withdrawal_to,omitempty"`
	SlashLocked        *string                   `json:"slash_locked,omitempty"`
	SetApproveProxy    *SetApproveProxyParams    `json:"set_approve_proxy,omitempty"`
}

// This struct is used to represent the recipient and amount fields together.
type RecipientAmount struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

// This struct is used to represent the controller and amount fields together.
type QueueWithdrawalToParams struct {
	// the amount is the amount of shares to be withdrawn
	Amount string `json:"amount"`
	// the controller is the address that can redeem the withdrawal after the lock period
	Controller string `json:"controller"`
	// the owner is the address that owns the shares being withdrawn
	Owner string `json:"owner"`
}

// This struct is used to represent a recipient for RedeemWithdrawalTo.
type RedeemWithdrawalToParams struct {
	Controller string `json:"controller"`
	Recipient  string `json:"recipient"`
}

type SetApproveProxyParams struct {
	// whether the proxy is approved or not.
	Approve bool `json:"approve"`
	// The proxy address that is being approved.
	Proxy string `json:"proxy"`
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
	Controller string `json:"controller"`
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

// The response to the `QueuedWithdrawal` query. This is just a wrapper around
// `QueuedWithdrawalInfo`, so that the schema can be generated.
type QueuedWithdrawalResponse struct {
	QueuedShares    string `json:"queued_shares"`
	UnlockTimestamp string `json:"unlock_timestamp"`
}

type VaultInfoResponse struct {
	// Asset identifier, using the CAIP-19 format.
	AssetID string `json:"asset_id"`
	// The asset reference stores the cw20 contract address or the bank denom.
	AssetReference string `json:"asset_reference"`
	// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
	AssetType AssetType `json:"asset_type"`
	// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
	Contract string `json:"contract"`
	// The `operator` that this vault is delegated to
	Operator string `json:"operator"`
	// The `pauser` contract address
	Pauser string `json:"pauser"`
	// The `vault-router` contract address
	Router string `json:"router"`
	// The total assets under management
	TotalAssets string `json:"total_assets"`
	// The total shares in circulation
	TotalShares string `json:"total_shares"`
	// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
	Version string `json:"version"`
}

// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
type AssetType string

const (
	Bank AssetType = "bank"
	Cw20 AssetType = "cw20"
)
