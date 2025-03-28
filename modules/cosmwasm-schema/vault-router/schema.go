// This file was automatically generated from vault-router/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultrouter

type IsValidatingResponse bool

type IsWhitelistedResponse bool

type VaultListResponse []Vault

type WithdrawalLockPeriodResponse string

type InstantiateMsg struct {
	Owner    string `json:"owner"`
	Pauser   string `json:"pauser"`
	Registry string `json:"registry"`
}

// ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it. Only
// the `owner` can call this message.
//
// ExecuteMsg SetWithdrawalLockPeriod the lock period for withdrawal. Only the `owner` can
// call this message.
//
// ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
// information on this field
type ExecuteMsg struct {
	SetVault                *SetVault          `json:"set_vault,omitempty"`
	SetWithdrawalLockPeriod *string            `json:"set_withdrawal_lock_period,omitempty"`
	TransferOwnership       *TransferOwnership `json:"transfer_ownership,omitempty"`
}

type SetVault struct {
	Vault       string `json:"vault"`
	Whitelisted bool   `json:"whitelisted"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

// QueryMsg IsWhitelisted: returns true if the vault is whitelisted. See
// [`ExecuteMsg::SetVault`]
//
// QueryMsg IsValidating: returns true if the operator is validating services. See BVS
// Registry for more information.
//
// QueryMsg ListVaults: returns a list of vaults. You can provide `limit` and `start_after`
// to paginate the results. The max `limit` is 100.
//
// QueryMsg WithdrawalLockPeriod: returns the withdrawal lock period.
type QueryMsg struct {
	IsWhitelisted        *IsWhitelisted        `json:"is_whitelisted,omitempty"`
	IsValidating         *IsValidating         `json:"is_validating,omitempty"`
	ListVaults           *ListVaults           `json:"list_vaults,omitempty"`
	ListVaultsByOperator *ListVaultsByOperator `json:"list_vaults_by_operator,omitempty"`
	WithdrawalLockPeriod *WithdrawalLockPeriod `json:"withdrawal_lock_period,omitempty"`
}

type IsValidating struct {
	Operator string `json:"operator"`
}

type IsWhitelisted struct {
	Vault string `json:"vault"`
}

type ListVaults struct {
	Limit      *int64  `json:"limit"`
	StartAfter *string `json:"start_after"`
}

type ListVaultsByOperator struct {
	Limit      *int64  `json:"limit"`
	Operator   string  `json:"operator"`
	StartAfter *string `json:"start_after"`
}

type WithdrawalLockPeriod struct {
}

// The response to the `ListVaults` query. For pagination, the `start_after` field is the
// last `vault` from the previous page.
type Vault struct {
	Vault       string `json:"vault"`
	Whitelisted bool   `json:"whitelisted"`
}
