package wasm

import (
	"encoding/base64"
	"fmt"
	"log/slog"
	"reflect"
	"strconv"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	junotypes "github.com/forbole/juno/v6/types"
	"github.com/ohler55/ojg/oj"

	eventutils "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/utils"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/utils"
)

var msgFilter = map[string]bool{
	types.MsgStoreCode:           true,
	types.MsgInstantiateContract: true,
	types.MsgExecuteContract:     true,
	types.MsgMigrateContract:     true,
	types.MsgUpdateAdmin:         true,
	types.MsgClearAdmin:          true,
}

// HandleMsg implements modules.MessageModule
func (m *Module) HandleMsg(index int, msg junotypes.Message, tx *junotypes.Transaction) error {
	if _, ok := msgFilter[msg.GetType()]; !ok {
		return nil
	}

	slog.Debug("Handle wasm message in wasm module", "tx hash", tx.TxHash, "block height", tx.Height,
		"message type", msg.GetType(), "index", msg.GetIndex())

	switch msg.GetType() {
	case types.MsgStoreCode:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgStoreCode{})
		return m.HandleMsgStoreCode(index, tx, cosmosMsg)

	case types.MsgInstantiateContract:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgInstantiateContract{})
		return m.HandleMsgInstantiateContract(index, tx, cosmosMsg)

	case types.MsgExecuteContract:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgExecuteContract{})
		return m.HandleMsgExecuteContract(index, tx, cosmosMsg)

	case types.MsgMigrateContract:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgMigrateContract{})
		return m.HandleMsgMigrateContract(index, tx, cosmosMsg)

	case types.MsgUpdateAdmin:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgUpdateAdmin{})
		return m.HandleMsgUpdateAdmin(cosmosMsg)

	case types.MsgClearAdmin:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgClearAdmin{})
		return m.HandleMsgClearAdmin(cosmosMsg)

	default:
		return fmt.Errorf("unknown msg type: %s", msg.GetType())
	}
}

// HandleMsgStoreCode allows to properly handle a MsgStoreCode
// The Store Code Event is to upload the contract code on the chain, where a Code ID is returned
func (m *Module) HandleMsgStoreCode(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgStoreCode) error {
	// Get store code event
	event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeStoreCode)

	if !success {
		slog.Error("Failed to search for EventTypeStoreCode", "tx hash", tx.TxHash)
		return fmt.Errorf("failed to search for EventTypeStoreCode in %s", tx.TxHash)
	}

	// Get code ID from store code event
	codeIDKey, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyCodeID)
	if err != nil {
		slog.Error("Failed to search for AttributeKeyCodeID", "error", err)
		return fmt.Errorf("failed to search for AttributeKeyCodeID: %s", err)
	}

	codeID, err := strconv.ParseUint(codeIDKey, 10, 64)
	if err != nil {
		slog.Error("Failed to parse code id to int64", "error", err)
		return fmt.Errorf("failed to parse code id to int64: %s", err)
	}

	return m.db.SaveWasmCode(
		types.NewWasmCode(
			msg.Sender, msg.WASMByteCode, msg.InstantiatePermission, codeID, int64(tx.Height),
		),
	)
}

// HandleMsgInstantiateContract allows to properly handle a MsgInstantiateContract
// Instantiate Contract Event instantiates an executable contract with the code previously stored with Store Code Event
func (m *Module) HandleMsgInstantiateContract(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgInstantiateContract) error {
	// Get instantiate contract event
	event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeInstantiate)

	if !success {
		slog.Error("Failed to search for EventTypeInstantiate", "tx hash", tx.TxHash)
		return fmt.Errorf("failed to search for EventTypeInstantiate in %s", tx.TxHash)
	}

	// Get contract address
	contractAddress, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyContractAddr)
	if err != nil {
		slog.Error("Failed to search for AttributeKeyContractAddr", "error", err)
		return fmt.Errorf("failed to search for AttributeKeyContractAddr: %s", err)
	}

	// Get result data
	resultData, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyResultDataHex)
	if err != nil {
		slog.Error("Failed to search for AttributeKeyResultDataHex", "error", err)
		resultData = ""
	}
	resultDataBz, err := base64.StdEncoding.DecodeString(resultData)
	if err != nil {
		slog.Error("Failed to decode result data", "error", err)
		return fmt.Errorf("failed to decode result data: %s", err)
	}

	// Get the contract info
	contractInfo, err := m.source.GetContractInfo(int64(tx.Height), contractAddress)
	if err != nil {
		slog.Error("Failed to get contract info", "block height", tx.Height,
			"contract address", contractAddress, "error", err)
		return fmt.Errorf("failed to get proposal: %s", err)
	}

	timestamp, err := time.Parse(time.RFC3339, tx.Timestamp)
	if err != nil {
		slog.Error("Failed to parse time", "error", err)
		return fmt.Errorf("failed to parse time: %s", err)
	}

	// Get contract info extension
	var contractInfoExt string
	if contractInfo.Extension != nil {
		var extentionI wasmtypes.ContractInfoExtension
		err = m.cdc.UnpackAny(contractInfo.Extension, &extentionI)
		if err != nil {
			slog.Error("Failed to get contract info extension", "error", err)
			return fmt.Errorf("failed to get contract info extension: %s", err)
		}
		contractInfoExt = extentionI.String()
	}

	// Get contract states
	contractStates, err := m.source.GetContractStates(int64(tx.Height), contractAddress)
	if err != nil {
		slog.Error("Failed to get contract states", "block height", tx.Height,
			"contract address", contractAddress, "error", err)
		return fmt.Errorf("failed to get contract states: %s", err)
	}

	contract := types.NewWasmContract(
		msg.Sender, msg.Admin, msg.CodeID, msg.Label, msg.Msg, msg.Funds,
		contractAddress, string(resultDataBz), timestamp,
		contractInfo.Creator, contractInfoExt, contractStates, int64(tx.Height),
	)
	return m.db.SaveWasmContracts(
		[]types.WasmContract{contract},
	)
}

// HandleMsgExecuteContract allows to properly handle a MsgExecuteContract
// Execute Event executes an instantiated contract
func (m *Module) HandleMsgExecuteContract(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgExecuteContract) error {
	slog.Debug("Handle MsgExecuteContract", "tx hash", tx.TxHash)

	// Parse the ExecuteContract message body
	msgJSON, err := oj.ParseString(string(msg.Msg))
	if err != nil {
		slog.Error("Failed to parse message JSON", "error", err)
		return fmt.Errorf("failed to parse message JSON: %s", err)
	}

	// Use reflection to get the message name by pulling the 1st field name from the JSON struct
	messageName := ""
	v := reflect.ValueOf(msgJSON)
	if v.Len() == 1 && len(v.MapKeys()) == 1 {
		messageName = v.MapKeys()[0].String()
	} else {
		slog.Warn("Unable to parse message name from JSON", "tx hash", tx.TxHash, "json message", string(msg.Msg))
	}

	// Skip some message types
	if messageName == "write_k_v" {
		slog.Debug("Skipping contract message", "tx hash", tx.TxHash, "message name", messageName)
		return nil
	}
	slog.Debug("Processing contract message", "block height", tx.Height, "tx hash", tx.TxHash, "index", index,
		"message name", messageName)

	// Check if events slice is not empty and index is within range
	if index >= len(tx.Events) {
		slog.Error("index out of range", "index", index, "events length", len(tx.Events))
		return fmt.Errorf("index out of range: %d, events length: %d", index, len(tx.Events))
	}

	event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeExecute)
	slog.Debug("Processing contract message", "block height", tx.Height, "tx hash", tx.TxHash, "index", index,
		"message name", messageName)

	if !success {
		slog.Error("Failed to search for EventTypeExecute", "error", err)
		return fmt.Errorf("failed to search for EventTypeExecute: %s", err)
	}

	// Get result data
	resultData, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyResultDataHex)
	if err != nil {
		resultData = ""
	}
	resultDataBz, err := base64.StdEncoding.DecodeString(resultData)
	if err != nil {
		slog.Error("Failed to decode result data", "error", err)
		return fmt.Errorf("failed to decode result data: %s", err)
	}

	timestamp, err := time.Parse(time.RFC3339, tx.Timestamp)
	if err != nil {
		slog.Error("Failed to parse time", "error", err)
		return fmt.Errorf("failed to parse time: %s", err)
	}

	contractExists, _ := m.db.GetWasmContractExists(msg.Contract)
	slog.Info("Print contract info", "contractExists", contractExists)

	execute := types.NewWasmExecuteContract(
		msg.Sender, msg.Contract, msg.Msg, msg.Funds,
		string(resultDataBz), timestamp, int64(tx.Height), tx.TxHash,
	)

	// save a record of the raw contract execution details
	if err = m.db.SaveWasmExecuteContract(execute); err != nil {
		slog.Error("Failed to save WasmExecuteContract", "error", err)
	}

	// save a row for each event in the contract execution
	if err = m.db.SaveWasmExecuteContractEvents(execute, tx); err != nil {
		slog.Error("Failed to save events for WasmExecuteContract", "error", err)
	}

	return nil
}

// HandleMsgMigrateContract allows to properly handle a MsgMigrateContract
// Migrate Contract Event upgrade the contract by updating code ID generated from new Store Code Event
func (m *Module) HandleMsgMigrateContract(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgMigrateContract) error {
	// Get Migrate Contract event
	event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeMigrate)

	if !success {
		slog.Error("Failed to search for EventTypeMigrate", "tx hash", tx.TxHash)
		return fmt.Errorf("failed to search for EventTypeMigrate in %s", tx.TxHash)
	}

	// Get result data
	resultData, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyResultDataHex)
	if err != nil {
		resultData = ""
	}
	resultDataBz, err := base64.StdEncoding.DecodeString(resultData)
	if err != nil {
		slog.Error("Failed to decode result data", "error", err)
		return fmt.Errorf("failed to decode result data: %s", err)
	}

	return m.db.UpdateContractWithMsgMigrateContract(msg.Sender, msg.Contract, msg.CodeID, msg.Msg, string(resultDataBz))
}

// HandleMsgUpdateAdmin allows to properly handle a MsgUpdateAdmin
// Update Admin Event updates the contract admin who can migrate the wasm contract
func (m *Module) HandleMsgUpdateAdmin(msg *wasmtypes.MsgUpdateAdmin) error {
	return m.db.UpdateContractAdmin(msg.Sender, msg.Contract, msg.NewAdmin)
}

// HandleMsgClearAdmin allows to properly handle a MsgClearAdmin
// Clear Admin Event clears the admin which make the contract no longer migratable
func (m *Module) HandleMsgClearAdmin(msg *wasmtypes.MsgClearAdmin) error {
	return m.db.UpdateContractAdmin(msg.Sender, msg.Contract, "")
}
