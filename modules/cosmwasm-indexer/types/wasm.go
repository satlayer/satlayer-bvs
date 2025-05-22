package types

import (
	"encoding/json"
	"strings"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
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

// WASMExecuteContract represents the CosmWasm execute contract in x/wasm module
type WASMExecuteContract struct {
	Sender                 string
	ContractAddress        string
	ExecuteContractMessage []byte
	MessageType            string
	WASMEvent              []byte
	CustomWASMEvent        []byte
	ExecutedAt             time.Time
	Height                 int64
	TxHash                 string
}

// GetWasmExecuteContractMessageType gets the name of the contract execution message type. It will create a comma
// separated string if there are multiple keys in the root of the message type. If message is empty, or an array, or
// a single value, the return value will be an empty string.
func GetWasmExecuteContractMessageType(rawContractMsg []byte) string {
	// the default return is an empty string
	messageType := ""

	if rawContractMsg != nil {
		var msg map[string]any
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
	executeContractMessage, _ := rawMsg.MarshalJSON()

	messageType := GetWasmExecuteContractMessageType(rawMsg)

	return WASMExecuteContract{
		Sender:                 sender,
		ContractAddress:        contractAddress,
		ExecuteContractMessage: executeContractMessage,
		MessageType:            messageType,
		WASMEvent:              wasmEvent,
		CustomWASMEvent:        customWASMEvent,
		ExecutedAt:             executedAt,
		Height:                 height,
		TxHash:                 txHash,
	}
}
