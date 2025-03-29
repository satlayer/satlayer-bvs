// This file was automatically generated from vault-factory/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultfactory

type CodeIDResponse int64

type InstantiateMsg struct {
	Owner    string `json:"owner"`
	Pauser   string `json:"pauser"`
	Registry string `json:"registry"`
	Router   string `json:"router"`
}

// ExecuteMsg DeployCw20 Deploy a CW20 vault contract, the operator will be the sender of
// this message. The `cw20` is the address of the CW20 contract.
//
// ExecuteMsg DeployBank Deploy a Bank vault contract, the operator will be the sender of
// this message. The `denom` is the denomination of the native token, e.g. "ubbn" for
// Babylon native token.
//
// ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
// information on this field Only the `owner` can call this message.
//
// ExecuteMsg SetCodeId Set the code id for a vault type, allowing the factory to deploy
// vaults of that type. Only the `owner` can call this message.
type ExecuteMsg struct {
	DeployCw20        *DeployCw20        `json:"deploy_cw20,omitempty"`
	DeployBank        *DeployBank        `json:"deploy_bank,omitempty"`
	TransferOwnership *TransferOwnership `json:"transfer_ownership,omitempty"`
	SetCodeID         *SetCodeID         `json:"set_code_id,omitempty"`
}

type DeployBank struct {
	Denom string `json:"denom"`
}

type DeployCw20 struct {
	Cw20 string `json:"cw20"`
}

type SetCodeID struct {
	CodeID    int64     `json:"code_id"`
	VaultType VaultType `json:"vault_type"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	CodeID CodeID `json:"code_id"`
}

type CodeID struct {
	VaultType VaultType `json:"vault_type"`
}

type VaultType string

const (
	Bank VaultType = "bank"
	Cw20 VaultType = "cw20"
)
