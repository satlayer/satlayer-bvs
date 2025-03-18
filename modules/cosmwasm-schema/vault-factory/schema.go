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
//    vaultCodeIDSResponse, err := UnmarshalVaultCodeIDSResponse(bytes)
//    bytes, err = vaultCodeIDSResponse.Marshal()

package vaultfactory

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

func UnmarshalVaultCodeIDSResponse(data []byte) (VaultCodeIDSResponse, error) {
	var r VaultCodeIDSResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *VaultCodeIDSResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Owner    string `json:"owner"`
	Pauser   string `json:"pauser"`
	Registry string `json:"registry"`
	Router   string `json:"router"`
}

// ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
// information on this field Only the `owner` can call this message.
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
	VaultCodeIDS VaultCodeIDS `json:"vault_code_ids"`
}

type VaultCodeIDS struct {
}

type VaultCodeIDSResponse struct {
	CodeIDS CodeIDS `json:"code_ids"`
}

type CodeIDS struct {
}

type VaultType string

const (
	BankVault VaultType = "bank_vault"
	Cw20Vault VaultType = "cw20_vault"
)
