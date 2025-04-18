package wasm

import (
	"encoding/json"
	"fmt"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	junotypes "github.com/forbole/juno/v6/types"
)

// MessageWrapper wraps wasm types to implement junotypes.Message
type MessageWrapper struct {
	msg   interface{}
	index int
}

// NewMessageWrapper creates a new MessageWrapper
func NewMessageWrapper(msg interface{}, index int) *MessageWrapper {
	return &MessageWrapper{msg: msg, index: index}
}

// GetBytes implements junotypes.Message
func (w *MessageWrapper) GetBytes() json.RawMessage {
	bytes, err := json.Marshal(w.msg)
	if err != nil {
		return nil
	}
	return json.RawMessage(bytes)
}

// GetIndex implements junotypes.Message
func (w *MessageWrapper) GetIndex() int {
	return w.index
}

// GetType implements junotypes.Message
func (w *MessageWrapper) GetType() string {
	switch w.msg.(type) {
	case *wasmtypes.MsgStoreCode:
		return "/cosmwasm.wasm.v1.MsgStoreCode"
	case *wasmtypes.MsgInstantiateContract:
		return "/cosmwasm.wasm.v1.MsgInstantiateContract"
	case *wasmtypes.MsgInstantiateContract2:
		return "/cosmwasm.wasm.v1.MsgInstantiateContract2"
	case *wasmtypes.MsgMigrateContract:
		return "/cosmwasm.wasm.v1.MsgMigrateContract"
	case *wasmtypes.MsgClearAdmin:
		return "/cosmwasm.wasm.v1.MsgClearAdmin"
	case *wasmtypes.MsgUpdateAdmin:
		return "/cosmwasm.wasm.v1.MsgUpdateAdmin"
	default:
		return fmt.Sprintf("%T", w.msg)
	}
}

// WrapMsg wraps a wasm message to implement junotypes.Message
func WrapMsg(msg interface{}, index int) junotypes.Message {
	return NewMessageWrapper(msg, index)
}

// UnwrapMsg unwraps a wrapped message
func UnwrapMsg(msg junotypes.Message) interface{} {
	if wrapper, ok := msg.(*MessageWrapper); ok {
		return wrapper.msg
	}
	return msg
}
