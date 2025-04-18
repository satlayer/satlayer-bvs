package wasm

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	junotypes "github.com/forbole/juno/v6/types"
)

// HandleMsg implements modules.MessageModule
func (m *Module) HandleMsg(index int, msg junotypes.Message, tx *junotypes.Transaction) error {
	if len(tx.Logs) == 0 {
		return nil
	}

	unwrappedMsg := UnwrapMsg(msg)
	switch cosmosMsg := unwrappedMsg.(type) {
	case *wasmtypes.MsgStoreCode:
		return m.handleMsgStoreCode(tx, index, cosmosMsg)
	case *wasmtypes.MsgInstantiateContract:
		return m.handleMsgInstantiateContract(tx, index)
	case *wasmtypes.MsgInstantiateContract2:
		return m.handleMsgInstantiateContract(tx, index)
	case *wasmtypes.MsgMigrateContract:
		return m.handleMsgMigrateContract(tx, index, cosmosMsg)
	case *wasmtypes.MsgClearAdmin:
		return m.handleMsgClearAdmin(tx, index, cosmosMsg)
	case *wasmtypes.MsgUpdateAdmin:
		return m.handleMsgUpdateAdmin(tx, index, cosmosMsg)
	default:
		return fmt.Errorf("Unknown msg type: %v", cosmosMsg)
	}
}

func (m *Module) handleMsgStoreCode(tx *junotypes.Transaction, index int, msg *wasmtypes.MsgStoreCode) error {
	event, err := tx.FindEventByType(index, wasmtypes.EventTypeStoreCode)
	if err != nil {
		return err
	}

	codeID, err := tx.FindAttributeByKey(event, wasmtypes.AttributeKeyCodeID)
	if err != nil {
		return err
	}

	code := types.NewCode(codeID, msg.Sender, tx.Timestamp, tx.Height)

	return m.db.SaveCode(code)
}

func (m *Module) handleMsgInstantiateContract(tx *junotypes.Transaction, index int) error {
	contracts, err := GetAllContracts(tx, index, wasmtypes.EventTypeInstantiate)
	if err != nil {
		return err
	}

	if len(contracts) == 0 {
		return fmt.Errorf("No contract address found")
	}

	createdAt := &wasmtypes.AbsoluteTxPosition{
		BlockHeight: tx.Height,
		TxIndex:     uint64(index),
	}
	ctx := context.Background()
	for _, contractAddress := range contracts {
		response, err := m.client.ContractInfo(ctx, &wasmtypes.QueryContractInfoRequest{
			Address: contractAddress,
		})
		if err != nil {
			return err
		}

		creator, _ := sdktypes.AccAddressFromBech32(response.Creator)
		var admin sdktypes.AccAddress
		if response.Admin != "" {
			admin, _ = sdktypes.AccAddressFromBech32(response.Admin)
		}

		contractInfo := wasmtypes.NewContractInfo(response.CodeID, creator, admin, response.Label, createdAt)
		contract := types.NewContract(&contractInfo, contractAddress, tx.Timestamp)

		if err = m.db.SaveContract(contract); err != nil {
			return err
		}
	}

	return nil
}

func (m *Module) handleMsgMigrateContract(tx *junotypes.Transaction, index int, msg *wasmtypes.MsgMigrateContract) error {
	return m.db.SaveContractCodeID(msg.Contract, msg.CodeID)
}

func (m *Module) handleMsgClearAdmin(tx *junotypes.Transaction, index int, msg *wasmtypes.MsgClearAdmin) error {
	return m.db.UpdateContractAdmin(msg.Contract, "")
}

func (m *Module) handleMsgUpdateAdmin(tx *junotypes.Transaction, index int, msg *wasmtypes.MsgUpdateAdmin) error {
	return m.db.UpdateContractAdmin(msg.Contract, msg.NewAdmin)
}

func GetAllContracts(tx *junotypes.Transaction, index int, eventType string) ([]string, error) {
	contracts := []string{}
	event, err := tx.FindEventByType(index, eventType)
	if err != nil {
		return contracts, err
	}

	for _, attr := range event.Attributes {
		if attr.Key == wasmtypes.AttributeKeyContractAddr {
			contracts = append(contracts, attr.Value)
		}
	}

	return contracts, nil
}
