// This file was generated from JSON Schema using quicktype, do not modify it directly.
// To parse and unparse this JSON data, add this code to your project and do:
//
//    instantiateMsg, err := UnmarshalInstantiateMsg(bytes)
//    bytes, err = instantiateMsg.Marshal()
//
//    executeMsg, err := UnmarshalExecuteMsg(bytes)
//    bytes, err = executeMsg.Marshal()
//
//    queryMsg, err := UnmarshalQueryMsg(bytes)
//    bytes, err = queryMsg.Marshal()
//
//    isValidatingResponse, err := UnmarshalIsValidatingResponse(bytes)
//    bytes, err = isValidatingResponse.Marshal()
//
//    isWhitelistedResponse, err := UnmarshalIsWhitelistedResponse(bytes)
//    bytes, err = isWhitelistedResponse.Marshal()
//
//    vaultListResponse, err := UnmarshalVaultListResponse(bytes)
//    bytes, err = vaultListResponse.Marshal()

package vaultrouter

import "encoding/json"

func UnmarshalInstantiateMsg(data []byte) (InstantiateMsg, error) {
	var r InstantiateMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *InstantiateMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalExecuteMsg(data []byte) (ExecuteMsg, error) {
	var r ExecuteMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ExecuteMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalQueryMsg(data []byte) (QueryMsg, error) {
	var r QueryMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *QueryMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type IsValidatingResponse bool

func UnmarshalIsValidatingResponse(data []byte) (IsValidatingResponse, error) {
	var r IsValidatingResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsValidatingResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type IsWhitelistedResponse bool

func UnmarshalIsWhitelistedResponse(data []byte) (IsWhitelistedResponse, error) {
	var r IsWhitelistedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsWhitelistedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type VaultListResponse []Vault

func UnmarshalVaultListResponse(data []byte) (VaultListResponse, error) {
	var r VaultListResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *VaultListResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Owner  string `json:"owner"`
	Pauser string `json:"pauser"`
}

// ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it. Only
// the `owner` can call this message.
//
// ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
// information on this field
type ExecuteMsg struct {
	SetVault          *SetVault          `json:"set_vault,omitempty"`
	TransferOwnership *TransferOwnership `json:"transfer_ownership,omitempty"`
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
type QueryMsg struct {
	IsWhitelisted *IsWhitelisted `json:"is_whitelisted,omitempty"`
	IsValidating  *IsValidating  `json:"is_validating,omitempty"`
	ListVaults    *ListVaults    `json:"list_vaults,omitempty"`
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

// The response to the `ListVaults` query. For pagination, the `start_after` field is the
// last `vault` from the previous page.
type Vault struct {
	Vault       string `json:"vault"`
	Whitelisted bool   `json:"whitelisted"`
}
