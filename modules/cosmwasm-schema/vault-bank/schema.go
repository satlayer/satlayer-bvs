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
//    assetsResponse, err := UnmarshalAssetsResponse(bytes)
//    bytes, err = assetsResponse.Marshal()
//
//    convertToAssetsResponse, err := UnmarshalConvertToAssetsResponse(bytes)
//    bytes, err = convertToAssetsResponse.Marshal()
//
//    convertToSharesResponse, err := UnmarshalConvertToSharesResponse(bytes)
//    bytes, err = convertToSharesResponse.Marshal()
//
//    sharesResponse, err := UnmarshalSharesResponse(bytes)
//    bytes, err = sharesResponse.Marshal()
//
//    totalAssetsResponse, err := UnmarshalTotalAssetsResponse(bytes)
//    bytes, err = totalAssetsResponse.Marshal()
//
//    totalSharesResponse, err := UnmarshalTotalSharesResponse(bytes)
//    bytes, err = totalSharesResponse.Marshal()
//
//    vaultInfoResponse, err := UnmarshalVaultInfoResponse(bytes)
//    bytes, err = vaultInfoResponse.Marshal()

package vaultbank

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

type AssetsResponse string

func UnmarshalAssetsResponse(data []byte) (AssetsResponse, error) {
	var r AssetsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *AssetsResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type ConvertToAssetsResponse string

func UnmarshalConvertToAssetsResponse(data []byte) (ConvertToAssetsResponse, error) {
	var r ConvertToAssetsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ConvertToAssetsResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type ConvertToSharesResponse string

func UnmarshalConvertToSharesResponse(data []byte) (ConvertToSharesResponse, error) {
	var r ConvertToSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ConvertToSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type SharesResponse string

func UnmarshalSharesResponse(data []byte) (SharesResponse, error) {
	var r SharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type TotalAssetsResponse string

func UnmarshalTotalAssetsResponse(data []byte) (TotalAssetsResponse, error) {
	var r TotalAssetsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TotalAssetsResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type TotalSharesResponse string

func UnmarshalTotalSharesResponse(data []byte) (TotalSharesResponse, error) {
	var r TotalSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TotalSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalVaultInfoResponse(data []byte) (VaultInfoResponse, error) {
	var r VaultInfoResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *VaultInfoResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

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
