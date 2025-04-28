package wasm

import (
	"fmt"
	"log/slog"
	"reflect"
	"slices"
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
	types.MsgStoreCode:            true,
	types.MsgInstantiateContract:  true,
	types.MsgInstantiateContract2: true,
	types.MsgExecuteContract:      true,
	types.MsgMigrateContract:      true,
	types.MsgUpdateAdmin:          true,
	types.MsgClearAdmin:           true,
}

// HandleMsg implements modules.MessageModule
func (m *Module) HandleMsg(index int, msg junotypes.Message, tx *junotypes.Transaction) error {
	if _, ok := msgFilter[msg.GetType()]; !ok {
		return nil
	}

	slog.Info("Handle wasm message in wasm module", "tx hash", tx.TxHash, "block height", tx.Height,
		"message type", msg.GetType(), "index", msg.GetIndex())

	switch msg.GetType() {
	case types.MsgStoreCode:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgStoreCode{})
		return m.HandleMsgStoreCode(index, tx, cosmosMsg)

	case types.MsgInstantiateContract, types.MsgInstantiateContract2:
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
		return m.HandleMsgUpdateAdmin(tx, cosmosMsg)
	case types.MsgClearAdmin:
		cosmosMsg := utils.UnpackMessage(m.cdc, msg.GetBytes(), &wasmtypes.MsgClearAdmin{})
		return m.HandleMsgClearAdmin(tx, cosmosMsg)

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
		slog.Error("Failed to search for code_id attribute in event", "error", err)
		return err
	}

	codeID, err := strconv.ParseUint(codeIDKey, 10, 64)
	if err != nil {
		slog.Error("Failed to parse code id to uint64", "error", err)
		return err
	}

	if _, found := slices.BinarySearch(m.cfg.CodeIDs, codeID); !found {
		slog.Debug("Not found specified code id in HandleMsgStoreCode", "code id", codeID)
		return nil
	}

	slog.Debug("Handle MsgStoreCode", "tx hash", tx.TxHash, "code id", codeID, "index", index)

	return m.db.SaveWasmCode(
		types.NewWasmCode(
			msg.Sender, msg.WASMByteCode, msg.InstantiatePermission, codeID, int64(tx.Height),
		),
	)
}

// HandleMsgInstantiateContract allows to properly handle a MsgInstantiateContract
// Instantiate Contract Event instantiates an executable contract with the code previously stored with Store Code Event
func (m *Module) HandleMsgInstantiateContract(index int, tx *junotypes.Transaction, msg *wasmtypes.MsgInstantiateContract) error {
	if _, found := slices.BinarySearch(m.cfg.CodeIDs, msg.CodeID); !found {
		slog.Debug("Not found specified code id in HandleMsgInstantiateContract", "code id", msg.CodeID)
		return nil
	}

	// Get instantiate contract event
	event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeInstantiate)

	if !success {
		slog.Error("Failed to search for instantiate attribute in events", "tx hash", tx.TxHash)
		return fmt.Errorf("failed to search for EventTypeInstantiate in %s", tx.TxHash)
	}

	// Get contract address
	contractAddress, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyContractAddr)
	if err != nil {
		slog.Error("Failed to search for AttributeKeyContractAddr", "error", err)
		return err
	}

	// Only record the specified contract addresses in config
	labelName, ok := m.cfg.Contracts[contractAddress]
	if !ok {
		slog.Debug("Not found specified contract address in HandleMsgInstantiateContract",
			"contract address", contractAddress)
		return nil
	}

	slog.Debug("Handle MsgMigrateContract", "tx hash", tx.TxHash, "contract address", contractAddress,
		"contract label name", labelName, "index", index)

	// Get the contract info
	contractInfo, err := m.source.GetContractInfo(int64(tx.Height), contractAddress)
	if err != nil {
		slog.Error("Failed to get contract info", "block height", tx.Height,
			"contract address", contractAddress, "error", err)
		return err
	}

	timestamp, err := time.Parse(time.RFC3339, tx.Timestamp)
	if err != nil {
		slog.Error("Failed to parse time", "error", err)
		return err
	}

	// Get contract info extension
	var contractInfoExt string
	if contractInfo.Extension != nil {
		var extension wasmtypes.ContractInfoExtension
		err = m.cdc.UnpackAny(contractInfo.Extension, &extension)
		if err != nil {
			slog.Error("Failed to get contract info extension", "error", err)
			return err
		}
		contractInfoExt = extension.String()
	}

	// Get contract states
	contractStates, err := m.source.GetContractStates(int64(tx.Height), contractAddress)
	if err != nil {
		slog.Error("Failed to get contract states", "block height", tx.Height,
			"contract address", contractAddress, "error", err)
		return err
	}

	contract := types.NewWasmContract(
		msg.Sender, msg.Admin, msg.CodeID, msg.Label, msg.Msg, msg.Funds,
		contractAddress, string("TODO"), timestamp,
		contractInfo.Creator, contractInfoExt, contractStates, int64(tx.Height),
	)
	return m.db.SaveWasmContracts(
		[]types.WasmContract{contract},
	)
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
	msgJSON, err := oj.ParseString(string(msg.Msg))
	if err != nil {
		slog.Error("Failed to parse message JSON", "error", err)
		return err
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
		slog.Error("Index out of range", "index", index, "events length", len(tx.Events))
		return fmt.Errorf("index out of range: %d, events length: %d", index, len(tx.Events))
	}

	txEvents := sdktypes.StringifyEvents(tx.Events)
	slog.Info("wasm attribute", slog.Any("all events", txEvents))

	// wasmAttr, success := eventutils.FindEventByType(txEvents, wasmtypes.WasmModuleEventType)
	// if !success {
	// 	slog.Error("Failed to search for wasm attribute in event", "error", err)
	// 	return err
	// }
	// slog.Info("wasm attribute", "wasm detail", wasmAttr)

	timestamp, err := time.Parse(time.RFC3339, tx.Timestamp)
	if err != nil {
		slog.Error("Failed to parse time", "error", err)
		return err
	}

	contractExists, _ := m.db.GetWasmContractExists(msg.Contract)
	if !contractExists {
		slog.Info("Contract doesn't exist in db", "contract address", msg.Contract)

		contractAddress := msg.Contract

		// default values
		contractInfoCreator := "unknown"
		contractInfoAdmin := "unknown"
		contractInfoCodeID := uint64(0)
		contractInfoLabel := ""

		// Check if there is a record of the contract, otherwise look it up
		contractInfo, err := m.source.GetContractInfo(int64(tx.Height), contractAddress)
		if err != nil {
			slog.Error("Failed to get contract info", "error", err, "contract address", msg.Contract,
				"block height", tx.Height)
		} else {
			contractInfoCreator = contractInfo.Creator
			contractInfoAdmin = contractInfo.Admin
			contractInfoCodeID = contractInfo.CodeID
			contractInfoLabel = contractInfo.Label
		}

		createdBlockHeight := int64(0)
		if contractInfo != nil && contractInfo.Created != nil {
			createdBlockHeight = int64(contractInfo.Created.BlockHeight)
		}

		emptyBytes := make([]byte, 0)
		var initPermission wasmtypes.AccessConfig
		newCode := types.NewWasmCode(
			contractInfoCreator, emptyBytes, &initPermission, contractInfoCodeID, createdBlockHeight,
		)

		err = m.db.SaveWasmCode(newCode)
		if err != nil {
			slog.Error("Failed to save contract code into db", "error", err)
			return fmt.Errorf("failed to save contract code: %s", err)
		}

		// Get contract info extension
		contractInfoExt := ""
		if contractInfo != nil && contractInfo.Extension != nil {
			var extension wasmtypes.ContractInfoExtension
			err = m.cdc.UnpackAny(contractInfo.Extension, &extension)
			if err != nil {
				return fmt.Errorf("failed to get contract info extension: %s", err)
			}
			contractInfoExt = extension.String()
		}

		// Set to default values, that will hopefully be overwritten during the next migration of this contract
		emptyRawMessage := []byte("{}")
		emptyFunds := sdktypes.Coins{sdktypes.Coin{}}

		var contractStates []wasmtypes.Model

		contract := types.NewWasmContract(
			msg.Sender, contractInfoAdmin, contractInfoCodeID, contractInfoLabel,
			emptyRawMessage, emptyFunds,
			contractAddress, string("TODO"), timestamp,
			contractInfoCreator, contractInfoExt, contractStates, createdBlockHeight,
		)

		err = m.db.SaveWasmContracts(
			[]types.WasmContract{contract},
		)
		if err != nil {
			return fmt.Errorf("failed to save contract info: %s", err)
		}
	}

	execute := types.NewWasmExecuteContract(
		msg.Sender, msg.Contract, msg.Msg, msg.Funds,
		string("TODO"), timestamp, int64(tx.Height), tx.TxHash,
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
	// Only record the specified contract addresses in config
	labelName, ok := m.cfg.Contracts[msg.Contract]
	if !ok {
		slog.Debug("Not found specified contractAddress in HandleMsgMigrateContract")
		return nil
	}

	slog.Debug("Handle MsgMigrateContract", "tx hash", tx.TxHash, "contract address", msg.Contract,
		"contract label name", labelName, "index", index, "new code id", msg.CodeID)

	// // Get Migrate Contract event
	// event, success := eventutils.FindEventByType(sdktypes.StringifyEvents(tx.Events), wasmtypes.EventTypeMigrate)
	//
	// if !success {
	// 	slog.Error("Failed to search for EventTypeMigrate", "tx hash", tx.TxHash)
	// 	return fmt.Errorf("failed to search for EventTypeMigrate in %s", tx.TxHash)
	// }

	return m.db.UpdateContractWithMsgMigrateContract(msg.Sender, msg.Contract, msg.CodeID, msg.Msg, string("TODO"))
}

// HandleMsgUpdateAdmin allows to properly handle a MsgUpdateAdmin
// Update Admin Event updates the contract admin who can migrate the wasm contract
func (m *Module) HandleMsgUpdateAdmin(tx *junotypes.Transaction, msg *wasmtypes.MsgUpdateAdmin) error {
	// Only record the specified contract addresses in config
	labelName, ok := m.cfg.Contracts[msg.Contract]
	if !ok {
		slog.Debug("Not found specified contractAddress in HandleMsgUpdateAdmin")
		return nil
	}

	slog.Debug("Handle MsgUpdateAdmin", "tx hash", tx.TxHash, "contract address", msg.Contract,
		"contract label name", labelName)

	return m.db.UpdateContractAdmin(msg.Sender, msg.Contract, msg.NewAdmin)
}

// HandleMsgClearAdmin allows to properly handle a MsgClearAdmin
// Clear Admin Event clears the admin which make the contract no longer migratable
func (m *Module) HandleMsgClearAdmin(tx *junotypes.Transaction, msg *wasmtypes.MsgClearAdmin) error {
	// Only record the specified contract addresses in config
	labelName, ok := m.cfg.Contracts[msg.Contract]
	if !ok {
		slog.Debug("Not found specified contractAddress in HandleMsgClearAdmin")
		return nil
	}

	slog.Debug("Handle MsgClearAdmin", "tx hash", tx.TxHash, "contract address", msg.Contract,
		"contract label name", labelName)

	return m.db.UpdateContractAdmin(msg.Sender, msg.Contract, "")
}
