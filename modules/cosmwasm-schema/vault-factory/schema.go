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
// ExecuteMsg DeployCw20Tokenized Deploy a Cw20 tokenized vault contract, the operator will
// be the sender of this message. The `symbol` is the symbol for the receipt token. Must
// start with sat and conform the Bank symbol rules. The `name` is the cw20 compliant name
// for the receipt token.
//
// ExecuteMsg DeployBank Deploy a Bank vault contract, the operator will be the sender of
// this message. The `denom` is the denomination of the native token, e.g. "ubbn" for
// Babylon native token.
//
// ExecuteMsg DeployBankTokenized Deploy a Bank tokenized vault contract, the operator will
// be the sender of this message. The `denom` is the denomination of the native token, e.g.
// "ubbn" for Babylon native token. The `decimals` is the number of decimals for the receipt
// token The `symbol` is the symbol for the receipt token. Must start with sat and conform
// the Bank symbol rules. The `name` is the cw20 compliant name for the receipt token.
//
// ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
// information on this field Only the `owner` can call this message.
//
// ExecuteMsg SetCodeId Set the code id for a vault type, allowing the factory to deploy
// vaults of that type. Only the `owner` can call this message.
//
// ExecuteMsg MigrateVault Migrate an existing vault to a new code id. The `vault` is the
// address of the vault to migrate. The `vault_type` is the type of the vault to migrate.
// Note that this execute message assume setCodeId message has been called prior with new
// code id for the vault type.
type ExecuteMsg struct {
	DeployCw20          *DeployCw20          `json:"deploy_cw20,omitempty"`
	DeployCw20Tokenized *DeployCw20Tokenized `json:"deploy_cw20_tokenized,omitempty"`
	DeployBank          *DeployBank          `json:"deploy_bank,omitempty"`
	DeployBankTokenized *DeployBankTokenized `json:"deploy_bank_tokenized,omitempty"`
	TransferOwnership   *TransferOwnership   `json:"transfer_ownership,omitempty"`
	SetCodeID           *SetCodeID           `json:"set_code_id,omitempty"`
	MigrateVault        *MigrateVault        `json:"migrate_vault,omitempty"`
}

type DeployBank struct {
	Denom string `json:"denom"`
}

type DeployBankTokenized struct {
	Decimals int64  `json:"decimals"`
	Denom    string `json:"denom"`
	Name     string `json:"name"`
	Symbol   string `json:"symbol"`
}

type DeployCw20 struct {
	Cw20 string `json:"cw20"`
}

type DeployCw20Tokenized struct {
	Cw20   string `json:"cw20"`
	Name   string `json:"name"`
	Symbol string `json:"symbol"`
}

type MigrateVault struct {
	MigrateMsg   string    `json:"migrate_msg"`
	VaultAddress string    `json:"vault_address"`
	VaultType    VaultType `json:"vault_type"`
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
	Bank          VaultType = "bank"
	BankTokenized VaultType = "bank_tokenized"
	Cw20          VaultType = "cw20"
	Cw20Tokenized VaultType = "cw20_tokenized"
)
