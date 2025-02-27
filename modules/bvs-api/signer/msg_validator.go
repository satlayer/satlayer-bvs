package signer

import (
	"encoding/json"
	"errors"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

// MsgValidator defines the interface for message validation
type MsgValidator interface {
	ValidateMsg(msg sdktypes.Msg) error
}

// DefaultMsgValidator implements basic message validation
type DefaultMsgValidator struct{}

func (v *DefaultMsgValidator) ValidateMsg(msg sdktypes.Msg) error {
	// Check if message is nil
	if msg == nil {
		return errors.New("nil message")
	}

	// Validate JSON in execute contract message
	if execMsg, ok := msg.(*wasmtypes.MsgExecuteContract); ok {
		if !json.Valid(execMsg.Msg) {
			return errors.New("invalid JSON in execute message")
		}
	}

	// Validate JSON in query contract message
	if queryMsg, ok := msg.(*wasmtypes.QuerySmartContractStateRequest); ok {
		if !json.Valid(queryMsg.QueryData) {
			return errors.New("invalid JSON in query message")
		}
	}

	return nil
}
