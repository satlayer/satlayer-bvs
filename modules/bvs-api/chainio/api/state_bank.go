package api

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"golang.org/x/time/rate"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	statebank "github.com/satlayer/satlayer-bvs/bvs-cw/state-bank"
)

var wasmUpdateState sync.Map

type StateBank interface {
	WithGasAdjustment(gasAdjustment float64) StateBank
	WithGasPrice(gasPrice sdktypes.DecCoin) StateBank
	WithGasLimit(gasLimit uint64) StateBank

	GetWasmUpdateState(key string) (string, error)
	GetStateMap() *sync.Map
	Indexer(ClientCtx client.Context, contractAddress string, bvsContractAddr string, startBlockHeight int64,
		eventTypes []string, rateLimit rate.Limit, maxRetries int) *indexer.EventIndexer
	EventHandler(ch chan *indexer.Event)
	SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error)
	Set(ctx context.Context, key string, value string) (*coretypes.ResultTx, error)
	BindClient(contractAddress string)
}

type stateBankImpl struct {
	registeredBVSContract string
	io                    io.ChainIO
	contractAddr          string
	gasAdjustment         float64
	gasPrice              sdktypes.DecCoin
	gasLimit              uint64
}

func NewStateBank(chainIO io.ChainIO) StateBank {
	return &stateBankImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *stateBankImpl) WithGasAdjustment(gasAdjustment float64) StateBank {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *stateBankImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StateBank {
	r.gasPrice = gasPrice
	return r
}

func (r *stateBankImpl) WithGasLimit(gasLimit uint64) StateBank {
	r.gasLimit = gasLimit
	return r
}

func (r *stateBankImpl) BindClient(contractAddress string) {
	r.contractAddr = contractAddress
}

func (r *stateBankImpl) GetWasmUpdateState(key string) (string, error) {
	value, exists := wasmUpdateState.Load(key)
	if !exists {
		return "", fmt.Errorf("does not exist: %s", key)
	}
	return value.(string), nil
}

func (r *stateBankImpl) GetStateMap() *sync.Map {
	return &wasmUpdateState
}

func (r *stateBankImpl) Indexer(clientCtx client.Context, contractAddress string, bvsContractAddr string, startBlockHeight int64,
	eventTypes []string, rateLimit rate.Limit, maxRetries int) *indexer.EventIndexer {
	r.registeredBVSContract = bvsContractAddr
	return indexer.NewEventIndexer(clientCtx, contractAddress, startBlockHeight, eventTypes, rateLimit, maxRetries)
}

func (r *stateBankImpl) EventHandler(ch chan *indexer.Event) {
	for event := range ch {
		if r.registeredBVSContract != event.AttrMap["sender"] {
			continue
		}

		key, ok := event.AttrMap["key"]
		if !ok {
			continue
		}
		val, ok := event.AttrMap["value"]
		if !ok {
			continue
		}
		wasmUpdateState.Store(key, val)
	}
}

func (r *stateBankImpl) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  r.contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (r *stateBankImpl) SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
	r.registeredBVSContract = addr

	msg := statebank.ExecuteMsg{
		AddRegisteredBvsContract: &statebank.AddRegisteredBvsContract{
			Address: addr,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	executeOptions := r.newExecuteOptions(msgBytes, "SetRegisteredBVSContract")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *stateBankImpl) Set(ctx context.Context, key string, value string) (*coretypes.ResultTx, error) {
	msg := statebank.ExecuteMsg{
		Set: &statebank.Set{
			Key:   key,
			Value: value,
		},
	}

	msgBytes, err := msg.Marshal()
	if err != nil {
		return nil, err
	}

	executeOptions := r.newExecuteOptions(msgBytes, "Set")
	return r.io.SendTransaction(ctx, executeOptions)
}
