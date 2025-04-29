package types

import (
	"encoding/hex"
	"encoding/json"
	"strings"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// CosmWasm message type
const (
	MsgStoreCode            = "/cosmwasm.wasm.v1.MsgStoreCode"
	MsgInstantiateContract  = "/cosmwasm.wasm.v1.MsgInstantiateContract"
	MsgInstantiateContract2 = "/cosmwasm.wasm.v1.MsgInstantiateContract2"
	MsgExecuteContract      = "/cosmwasm.wasm.v1.MsgExecuteContract"
	MsgMigrateContract      = "/cosmwasm.wasm.v1.MsgMigrateContract"
	MsgUpdateAdmin          = "/cosmwasm.wasm.v1.MsgUpdateAdmin"
	MsgClearAdmin           = "/cosmwasm.wasm.v1.MsgClearAdmin"
)

// WASMParams represents the CosmWasm code in x/wasm module
type WASMParams struct {
	CodeUploadAccess             *wasmtypes.AccessConfig
	InstantiateDefaultPermission int32
	Height                       int64
}

// NewWASMParams allows to build a new x/wasm params instance
func NewWASMParams(
	codeUploadAccess *wasmtypes.AccessConfig, instantiateDefaultPermission int32, height int64,
) WASMParams {
	return WASMParams{
		CodeUploadAccess:             codeUploadAccess,
		InstantiateDefaultPermission: instantiateDefaultPermission,
		Height:                       height,
	}
}

// WASMCode represents the CosmWasm code in x/wasm module
type WASMCode struct {
	Sender                string
	WASMByteCode          []byte
	InstantiatePermission *wasmtypes.AccessConfig
	CodeID                uint64
	Height                int64
}

// NewWASMCode allows to build a new x/wasm code instance
func NewWASMCode(
	sender string, wasmByteCode []byte, initPermission *wasmtypes.AccessConfig, codeID uint64, height int64,
) WASMCode {
	return WASMCode{
		Sender:                sender,
		WASMByteCode:          wasmByteCode,
		InstantiatePermission: initPermission,
		CodeID:                codeID,
		Height:                height,
	}
}

// WASMInstantiateContract represents the CosmWasm instantiate contract in x/wasm module
type WASMInstantiateContract struct {
	Sender                 string
	Creator                string
	Admin                  string
	CodeID                 uint64
	Label                  string
	InstantiateContractMsg wasmtypes.RawContractMessage
	ContractAddress        string
	WASMEvent              []byte
	CustomWASMEvent        []byte
	ContractInfoExtension  string
	ContractStates         []byte
	Funds                  sdk.Coins
	InstantiatedAt         time.Time
	Height                 int64
	TxHash                 string
}

// NewInstantiateWASMContract allows to build a new x/wasm contract instance.
func NewInstantiateWASMContract(
	sender string, creator string, admin string, codeID uint64, label string, rawMsg wasmtypes.RawContractMessage,
	contractAddress string, wasmEvent []byte, customWASMEvent []byte, contractInfoExtension string, states []wasmtypes.Model,
	funds sdk.Coins, instantiatedAt time.Time, height int64, txHash string,
) WASMInstantiateContract {
	instantiateContractMsg, _ := rawMsg.MarshalJSON()
	contractStateInfo := ConvertContractStates(states)

	return WASMInstantiateContract{
		Sender:                 sender,
		Creator:                creator,
		Admin:                  admin,
		CodeID:                 codeID,
		Label:                  label,
		InstantiateContractMsg: instantiateContractMsg,
		ContractAddress:        contractAddress,
		WASMEvent:              wasmEvent,
		CustomWASMEvent:        customWASMEvent,
		ContractInfoExtension:  contractInfoExtension,
		ContractStates:         contractStateInfo,
		Funds:                  funds,
		InstantiatedAt:         instantiatedAt,
		Height:                 height,
		TxHash:                 txHash,
	}
}

// ConvertContractStates removes unaccepted hex characters for PostgreSQL from the state key
func ConvertContractStates(states []wasmtypes.Model) []byte {
	jsonStates := make(map[string]interface{})

	hexZero, _ := hex.DecodeString("00")
	for _, state := range states {
		key := state.Key
		// Remove initial 2 hex characters if the first is \x00
		if string(state.Key[:1]) == string(hexZero) {
			key = state.Key[2:]
		}

		// Remove \x00 hex characters in the middle
		for i := 0; i < len(key); i++ {
			if string(key[i]) == string(hexZero) {
				key = append(key[:i], key[i+1:]...)
				i--
			}
		}

		// Decode hex value
		keyBz, _ := hex.DecodeString(key.String())

		jsonStates[string(keyBz)] = string(state.Value)
	}
	jsonStatesBz, _ := json.Marshal(&jsonStates)

	return jsonStatesBz
}

// WASMExecuteContract represents the CosmWasm execute contract in x/wasm module
type WASMExecuteContract struct {
	Sender             string
	ContractAddress    string
	ExecuteContractMsg []byte
	MessageType        string
	WASMEvent          []byte
	CustomWASMEvent    []byte
	ExecutedAt         time.Time
	Height             int64
	TxHash             string
}

// GetWasmExecuteContractMessageType gets the name of the contract execution message type. It will create a comma
// separated string if there are multiple keys in the root of the message type. If message is empty, or an array, or
// a single value, the return value will be an empty string.
func GetWasmExecuteContractMessageType(rawContractMsg []byte) string {
	// the default return is an empty string
	messageType := ""

	if rawContractMsg != nil {
		var msg map[string]interface{}
		_ = json.Unmarshal(rawContractMsg, &msg)

		if len(msg) > 0 {

			// get the keys in the root of the message
			keys := make([]string, 0, len(msg))
			for k := range msg {
				keys = append(keys, k)
			}

			// create string of key names, so `{ a: 1 }` will be "a", and `{ b: 2, c: { ... } }` will be "b,c"
			messageType = strings.Join(keys, ",")
		}
	}

	return messageType
}

// NewWASMExecuteContract allows to build a new x/wasm execute contract instance
func NewWASMExecuteContract(
	sender string, contractAddress string, rawMsg wasmtypes.RawContractMessage, wasmEvent []byte, customWASMEvent []byte,
	executedAt time.Time, height int64, txHash string,
) WASMExecuteContract {
	executeContractMsg, _ := rawMsg.MarshalJSON()

	messageType := GetWasmExecuteContractMessageType(rawMsg)

	return WASMExecuteContract{
		Sender:             sender,
		ContractAddress:    contractAddress,
		ExecuteContractMsg: executeContractMsg,
		MessageType:        messageType,
		WASMEvent:          wasmEvent,
		CustomWASMEvent:    customWASMEvent,
		ExecutedAt:         executedAt,
		Height:             height,
		TxHash:             txHash,
	}
}

// WASMMigrateContract represents the CosmWasm migrate contract in x/wasm module.
type WASMMigrateContract struct {
	Sender             string
	CodeID             uint64
	ContractAddress    string
	MigrateContractMsg wasmtypes.RawContractMessage
	WASMEvent          []byte
	CustomWASMEvent    []byte
	MigratedAt         time.Time
	Height             int64
	TxHash             string
}

// NewWasmMigrateContract allows to migrate one contract to a new contract instance.
func NewWASMMigrateContract(
	sender string, codeID uint64, contractAddress string, rawMsg wasmtypes.RawContractMessage, wasmEvent []byte,
	customWASMEvent []byte, migratedAt time.Time, height int64, txHash string,
) WASMMigrateContract {
	rawContractMsg, _ := rawMsg.MarshalJSON()

	return WASMMigrateContract{
		Sender:             sender,
		CodeID:             codeID,
		ContractAddress:    contractAddress,
		MigrateContractMsg: rawContractMsg,
		WASMEvent:          wasmEvent,
		CustomWASMEvent:    customWASMEvent,
		MigratedAt:         migratedAt,
		Height:             height,
		TxHash:             txHash,
	}
}
