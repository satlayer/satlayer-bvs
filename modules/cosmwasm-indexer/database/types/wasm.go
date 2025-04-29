package types

import (
	"database/sql/driver"
	"fmt"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

// DBAccessConfig represents the information stored inside the database about a single access_config
type DBAccessConfig struct {
	Permission int      `db:"permission"`
	Address    []string `db:"address"`
}

// NewDBAccessConfig builds DBAccessConfig starting from CosmWasm type AccessConfig
func NewDBAccessConfig(accessCfg *wasmtypes.AccessConfig) DBAccessConfig {
	return DBAccessConfig{
		Permission: int(accessCfg.Permission),
		Address:    accessCfg.Addresses,
	}
}

// Value implements driver.Valuer
func (cfg *DBAccessConfig) Value() (driver.Value, error) {
	if cfg != nil {
		return fmt.Sprintf("(%d,%s)", cfg.Permission, cfg.Address), nil
	}

	return fmt.Sprintf("(%d,%s)", wasmtypes.AccessTypeUnspecified, ""), nil
}

// Equal tells whether a and b represent the same access_config
func (cfg *DBAccessConfig) Equal(b *DBAccessConfig) bool {
	return cfg.Permission == b.Permission
}

// WasmParams represents the CosmWasm code in x/wasm module
type WasmParams struct {
	CodeUploadAccess             *DBAccessConfig `db:"code_upload_access"`
	InstantiateDefaultPermission int32           `db:"instantiate_default_permission"`
	Height                       int64           `db:"height"`
}

// NewWASMParams allows to build a new x/wasm params instance
func NewWASMParams(
	codeUploadAccess *DBAccessConfig, instantiateDefaultPermission int32, height int64,
) WasmParams {
	return WasmParams{
		CodeUploadAccess:             codeUploadAccess,
		InstantiateDefaultPermission: instantiateDefaultPermission,
		Height:                       height,
	}
}

// WASMCodeRow represents a single row inside the "wasm_code" table
type WASMCodeRow struct {
	Sender                string          `db:"sender"`
	WASMByteCode          string          `db:"wasm_byte_code"`
	InstantiatePermission *DBAccessConfig `db:"instantiate_permission"`
	CodeID                int64           `db:"code_id"`
	Height                int64           `db:"height"`
}

// NewWASMCodeRow allows to easily create a new NewWASMCodeRow
func NewWASMCodeRow(
	sender string,
	wasmByteCode string,
	instantiatePermission *DBAccessConfig,
	codeID int64,
	height int64,
) WASMCodeRow {
	return WASMCodeRow{
		Sender:                sender,
		WASMByteCode:          wasmByteCode,
		InstantiatePermission: instantiatePermission,
		CodeID:                codeID,
		Height:                height,
	}
}

// Equals return true if one WasmCodeRow representing the same row as the original one
func (a WASMCodeRow) Equals(b WASMCodeRow) bool {
	return a.Sender == b.Sender &&
		a.WASMByteCode == b.WASMByteCode &&
		a.InstantiatePermission.Equal(b.InstantiatePermission) &&
		a.CodeID == b.CodeID &&
		a.Height == b.Height
}

// WASMInstantiateContractRow represents a single row inside the "wasm_instantiate_contract" table
type WASMInstantiateContractRow struct {
	Sender                string    `db:"sender"`
	Creator               string    `db:"creator"`
	Admin                 string    `db:"admin"`
	CodeID                int64     `db:"code_id"`
	Label                 string    `db:"label"`
	RawContractMessage    string    `db:"raw_contract_message"`
	Funds                 *DBCoins  `db:"funds"`
	ContractAddress       string    `db:"contract_address"`
	Data                  string    `db:"data"`
	InstantiatedAt        time.Time `db:"instantiated_at"`
	ContractInfoExtension string    `db:"contract_info_extension"`
	ContractStates        string    `db:"contract_states"`
	Height                int64     `db:"height"`
}

// NewWASMInstantiateContractRow allows to easily create a new WasmContractRow
func NewWASMInstantiateContractRow(
	sender string,
	admin string,
	codeID int64,
	label string,
	rawContractMessage string,
	funds *DBCoins,
	contractAddress string,
	data string,
	instantiatedAt time.Time,
	creator string,
	contractInfoExtension string,
	height int64,
) WASMInstantiateContractRow {
	return WASMInstantiateContractRow{
		Sender:                sender,
		Admin:                 admin,
		CodeID:                codeID,
		Label:                 label,
		RawContractMessage:    rawContractMessage,
		Funds:                 funds,
		ContractAddress:       contractAddress,
		Data:                  data,
		InstantiatedAt:        instantiatedAt,
		Creator:               creator,
		ContractInfoExtension: contractInfoExtension,
		Height:                height,
	}
}

// WASMExecuteContractRow represents a single row inside the "wasm_execute_contract" table
type WASMExecuteContractRow struct {
	Sender             string    `db:"sender"`
	ContractAddress    string    `db:"contract_address"`
	RawContractMessage string    `db:"raw_contract_message"`
	Funds              *DBCoins  `db:"funds"`
	Data               string    `db:"data"`
	ExecutedAt         time.Time `db:"executed_at"`
	Height             int64     `db:"height"`
	Hash               string    `db:"hash"`
}

// NewWASMExecuteContractRow allows to easily create a new WASMExecuteContractRow
func NewWASMExecuteContractRow(
	sender string,
	contractAddress string,
	rawContractMessage string,
	funds *DBCoins,
	data string,
	executedAt time.Time,
	height int64,
	hash string,
) WASMExecuteContractRow {
	return WASMExecuteContractRow{
		Sender:             sender,
		RawContractMessage: rawContractMessage,
		Funds:              funds,
		ContractAddress:    contractAddress,
		Data:               data,
		ExecutedAt:         executedAt,
		Height:             height,
		Hash:               hash,
	}
}

// WASMExecuteContractEventRow represents a single row inside the "wasm_execute_contract" table
type WASMExecuteContractEventRow struct {
	Sender          string    `db:"sender"`
	ContractAddress string    `db:"contract_address"`
	EventType       string    `db:"event_type"`
	Attributes      string    `db:"attributes"`
	ExecutedAt      time.Time `db:"executed_at"`
	Height          int64     `db:"height"`
	Hash            string    `db:"hash"`
}

// NewWASMExecuteContractEventRow allows to easily create a new WasmExecuteContractEventRow
func NewWASMExecuteContractEventRow(
	sender string,
	contractAddress string,
	eventType string,
	attributes string,
	executedAt time.Time,
	height int64,
	hash string,
) WASMExecuteContractEventRow {
	return WASMExecuteContractEventRow{
		Sender:          sender,
		ContractAddress: contractAddress,
		EventType:       eventType,
		Attributes:      attributes,
		ExecutedAt:      executedAt,
		Height:          height,
		Hash:            hash,
	}
}
