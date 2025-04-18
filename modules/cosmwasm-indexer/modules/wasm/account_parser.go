package wasm

import (
	"fmt"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

func WasmMessagesParser(_ codec.Codec, cosmosMsg sdktypes.Msg) ([]string, error) {
	switch msg := cosmosMsg.(type) {
	case *wasmtypes.MsgStoreCode:
		return []string{msg.Sender}, nil

	case *wasmtypes.MsgInstantiateContract:
		return []string{msg.Sender}, nil

	case *wasmtypes.MsgExecuteContract:
		return []string{msg.Sender}, nil

	case *wasmtypes.MsgMigrateContract:
		return []string{msg.Sender}, nil

	case *wasmtypes.MsgUpdateAdmin:
		return []string{msg.Sender}, nil

	case *wasmtypes.MsgClearAdmin:
		return []string{msg.Sender}, nil
	}

	return nil, fmt.Errorf("Unknown msg type: %v", cosmosMsg)
}
