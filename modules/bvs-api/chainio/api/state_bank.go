package api

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"sync"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"golang.org/x/time/rate"

	"github.com/satlayer/satlayer-api/chainio/indexer"
	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
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
	Set(ctx context.Context, key string, value int64) (*coretypes.ResultTx, error)
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

func (s *stateBankImpl) WithGasAdjustment(gasAdjustment float64) StateBank {
	s.gasAdjustment = gasAdjustment
	return s
}

func (s *stateBankImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StateBank {
	s.gasPrice = gasPrice
	return s
}

func (s *stateBankImpl) WithGasLimit(gasLimit uint64) StateBank {
	s.gasLimit = gasLimit
	return s
}

func (s *stateBankImpl) BindClient(contractAddress string) {
	s.contractAddr = contractAddress
}

func (s *stateBankImpl) GetWasmUpdateState(key string) (string, error) {
	value, exists := wasmUpdateState.Load(key)
	if !exists {
		return "", fmt.Errorf("does not exist: %s", key)
	}
	return value.(string), nil
}

func (s *stateBankImpl) GetStateMap() *sync.Map {
	return &wasmUpdateState
}

func (s *stateBankImpl) Indexer(clientCtx client.Context, contractAddress string, bvsContractAddr string, startBlockHeight int64,
	eventTypes []string, rateLimit rate.Limit, maxRetries int) *indexer.EventIndexer {
	s.registeredBVSContract = bvsContractAddr
	return indexer.NewEventIndexer(clientCtx, contractAddress, startBlockHeight, eventTypes, rateLimit, maxRetries)
}

func (s *stateBankImpl) EventHandler(ch chan *indexer.Event) {
	for event := range ch {
		sender, ok := event.AttrMap["sender"].(string)
		if !ok {
			continue
		}
		if strings.ToLower(s.registeredBVSContract) != strings.ToLower(sender) {
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

func (s *stateBankImpl) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  s.contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: s.gasAdjustment,
		GasPrice:      s.gasPrice,
		Gas:           s.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (s *stateBankImpl) SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
	s.registeredBVSContract = addr

	msg := types.AddRegisteredBVSContractReq{
		AddRegisteredBVSContract: types.AddRegisteredBVSContract{
			Address: addr,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	executeOptions := s.newExecuteOptions(msgBytes, "SetRegisteredBVSContract")
	return s.io.SendTransaction(ctx, executeOptions)
}

func (s *stateBankImpl) Set(ctx context.Context, key string, value int64) (*coretypes.ResultTx, error) {
	msg := types.SetReq{
		Set: types.Set{
			Key:   key,
			Value: value,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	executeOptions := s.newExecuteOptions(msgBytes, "Set")
	return s.io.SendTransaction(ctx, executeOptions)
}

func NewStateBankImpl(chainIO io.ChainIO) StateBank {
	return &stateBankImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
