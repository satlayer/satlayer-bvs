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
	SetVault                *SetVault             `json:"set_vault,omitempty"`
	SetWithdrawalLockPeriod *string               `json:"set_withdrawal_lock_period,omitempty"`
	TransferOwnership       *TransferOwnership    `json:"transfer_ownership,omitempty"`
	RequestSlashing         *RequestSlashingClass `json:"request_slashing,omitempty"`
}

type RequestSlashingClass struct {
	// The percentage of tokens to slash in basis points (1/100th of a percent). Max bips to
	// slash is set by the service slashing parameters at the timestamp and the operator must
	// have opted in.
	Bips int64 `json:"bips"`
	// Additional contextual information about the slashing request.
	Metadata RequestSlashingMetadata `json:"metadata"`
	// The operator address to slash. (service, operator) must have active registration at the
	// timestamp.
	Operator string `json:"operator"`
	// The timestamp at which the slashing condition occurred.
	Timestamp string `json:"timestamp"`
}

// Additional contextual information about the slashing request.
type RequestSlashingMetadata struct {
	// The reason for the slashing request. Must contain human-readable string. Max length of
	// 250 characters, empty string is allowed but not recommended.
	Reason string `json:"reason"`
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
// QueryMsg ListVaultsByOperator: returns a list of vaults managed by given operator. You
// can provide `limit` and `start_after` to paginate the results. The max `limit` is 100.
//
// QueryMsg WithdrawalLockPeriod: returns the withdrawal lock period.
type QueryMsg struct {
	IsWhitelisted        *IsWhitelisted        `json:"is_whitelisted,omitempty"`
	IsValidating         *IsValidating         `json:"is_validating,omitempty"`
	ListVaults           *ListVaults           `json:"list_vaults,omitempty"`
	ListVaultsByOperator *ListVaultsByOperator `json:"list_vaults_by_operator,omitempty"`
	WithdrawalLockPeriod *WithdrawalLockPeriod `json:"withdrawal_lock_period,omitempty"`
	SlashingRequestID    *SlashingRequestID    `json:"slashing_request_id,omitempty"`
	SlashingRequest      *string               `json:"slashing_request,omitempty"`
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

type SlashingRequestID struct {
	Operator string `json:"operator"`
	Service  string `json:"service"`
}

type WithdrawalLockPeriod struct {
}

// The response to the `ListVaults` query. For pagination, the `start_after` field is the
// last `vault` from the previous page.
type Vault struct {
	Vault       string `json:"vault"`
	Whitelisted bool   `json:"whitelisted"`
}

type SlashingRequest struct {
	// The core slashing request data including operator, bips, timestamp, and metadata.
	Request RequestClass `json:"request"`
	// The timestamp after which the request is no longer valid. This will be `request_time` +
	// `resolution_window` * 2 (as per current slashing parameters)
	RequestExpiry string `json:"request_expiry"`
	// The timestamp when the request was submitted.
	RequestTime string `json:"request_time"`
}

// The core slashing request data including operator, bips, timestamp, and metadata.
type RequestClass struct {
	// The percentage of tokens to slash in basis points (1/100th of a percent). Max bips to
	// slash is set by the service slashing parameters at the timestamp and the operator must
	// have opted in.
	Bips int64 `json:"bips"`
	// Additional contextual information about the slashing request.
	Metadata RequestMetadata `json:"metadata"`
	// The operator address to slash. (service, operator) must have active registration at the
	// timestamp.
	Operator string `json:"operator"`
	// The timestamp at which the slashing condition occurred.
	Timestamp string `json:"timestamp"`
}

// Additional contextual information about the slashing request.
type RequestMetadata struct {
	// The reason for the slashing request. Must contain human-readable string. Max length of
	// 250 characters, empty string is allowed but not recommended.
	Reason string `json:"reason"`
}
