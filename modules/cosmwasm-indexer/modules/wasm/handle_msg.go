package wasm

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"reflect"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	junotypes "github.com/forbole/juno/v6/types"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/utils"
)

var (
	emptyJSONBytes = []byte("{}")

	msgFilter = map[string]bool{
		types.MsgStoreCode:            false,
		types.MsgInstantiateContract:  false,
		types.MsgInstantiateContract2: false,
		types.MsgExecuteContract:      true,
		types.MsgMigrateContract:      false,
		types.MsgUpdateAdmin:          false,
		types.MsgClearAdmin:           false,
	}
)

// HandleMsg implements modules.MessageModule
func (m *Module) HandleMsg(index int, msg junotypes.Message, tx *junotypes.Transaction) error {
	if _, ok := msgFilter[msg.GetType()]; !ok {
		return nil
	}

	slog.Info("Handle CosmWasm message in wasm module", "tx hash", tx.TxHash, "block height", tx.Height,
		"message type", msg.GetType(), "index", msg.GetIndex())

	switch msg.GetType() {
	case types.MsgExecuteContract:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgExecuteContract{})
		return m.HandleMsgExecuteContract(index, tx, cosmosMsg)
	default:
		return fmt.Errorf("unknown msg type: %s", msg.GetType())
	}
}

// HandleMsgExecuteContract allows to properly handle a MsgExecuteContract
// Execute Event executes an instantiated contract
func (m *Module) HandleMsgExecuteContract(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgExecuteContract) error {
	// Only record the specified contract addresses in config
	labelName, ok := m.cfg.Contracts[msg.Contract]
	if !ok {
		slog.Debug("Not found specified contractAddress in HandleMsgExecuteContract")
		return nil
	}

	slog.Debug("Handle MsgExecuteContract", "tx hash", tx.TxHash, "contract address", msg.Contract,
		"contract label name", labelName, "index", index)

	// Parse the ExecuteContract message body
	var msgJSON map[string]interface{}
	err := json.Unmarshal(msg.Msg, &msgJSON)
	if err != nil {
		slog.Error("Failed to parse JSON message", "error", err)
		return err
	}

	// Use reflection to get the message name by pulling the 1st field name from the JSON struct
	messageName := ""
	v1 := reflect.ValueOf(msgJSON)
	if v1.Len() == 1 && len(v1.MapKeys()) == 1 {
		messageName = v1.MapKeys()[0].String()
	} else {
		slog.Warn("Unable to parse message name from JSON string", "tx hash", tx.TxHash, "json message", string(msg.Msg))
	}

	slog.Debug("Processing contract message", "block height", tx.Height, "tx hash", tx.TxHash, "index", index,
		"message name", messageName, "Msg", string(msg.Msg), slog.Any("msgJSON", msgJSON))

	// Check if events slice is not empty and index is within range
	if index >= len(tx.Events) {
		slog.Error("Index out of range", "index", index, "events length", len(tx.Events))
		return fmt.Errorf("index out of range: %d, events length: %d", index, len(tx.Events))
	}

	txEvents := sdktypes.StringifyEvents(tx.Events)

	wasmEvent, found := utils.FindEventByType(txEvents, wasmtypes.WasmModuleEventType)
	wasmByteEvent := emptyJSONBytes
	if found {
		if wasmByteEvent, err = utils.ExtractStringEvent(wasmEvent); err != nil {
			slog.Error("Failed to extract WASM event", "error", err)
			wasmByteEvent = emptyJSONBytes
		}
	} else {
		slog.Warn("Not found WASM event in execute events")
	}

	customWASMEvent, found := utils.FindCustomWASMEvent(txEvents)
	customWASMByteEvent := emptyJSONBytes
	if found {
		if customWASMByteEvent, err = utils.ExtractStringEvent(customWASMEvent); err != nil {
			slog.Error("Failed to extract custom WASM event", "error", err)
			customWASMByteEvent = emptyJSONBytes
		}
	} else {
		slog.Warn("Not found custom WASM event in execute events")
	}

	slog.Info("Execute events", slog.Any("all events", txEvents), slog.Any("wasmEvent", wasmEvent),
		slog.Any("customWASMEvent", customWASMEvent))

	timestamp, err := time.Parse(time.RFC3339, tx.Timestamp)
	if err != nil {
		slog.Error("Failed to parse time", "error", err)
		return err
	}

	execute := types.NewWASMExecuteContract(msg.Sender, msg.Contract, msg.Msg, wasmByteEvent, customWASMByteEvent,
		timestamp, int64(tx.Height), tx.TxHash)

	if err = m.db.SaveWASMExecuteContract(execute); err != nil {
		slog.Error("Failed to save WASMExecuteContract", "error", err)
	}

	return nil
}
