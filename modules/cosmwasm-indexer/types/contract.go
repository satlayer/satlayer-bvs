package types

import (
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

// Contract type
type Contract struct {
	Address     string
	CodeID      uint64
	Creator     string
	Admin       string
	Label       string
	CreatedTime string
}

// NewContract instance
func NewContract(
	info *wasmtypes.ContractInfo,
	address string,
	createdTime string,
) *Contract {
	return &Contract{
		Address:     address,
		CodeID:      info.CodeID,
		Creator:     info.Creator,
		Admin:       info.Admin,
		Label:       info.Label,
		CreatedTime: createdTime,
	}
}
