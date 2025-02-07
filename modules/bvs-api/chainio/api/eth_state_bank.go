package api

import (
	"context"
	"fmt"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"golang.org/x/time/rate"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

var ethUpdateState sync.Map

type ETHStateBank interface {
	GetEthUpdateState(key string) (string, error)
	GetStateMap() *sync.Map

	IsBVSContractRegistered(ctx context.Context, addr common.Address) (bool, error)
	Get(ctx context.Context, key string) (string, error)
	Owner(ctx context.Context) (common.Address, error)
	PendingOwner(ctx context.Context) (common.Address, error)

	SetRegisteredBVSContract(ctx context.Context, wallet types.ETHWallet, contractAddr common.Address) (*sdktypes.Receipt, error)
	Set(ctx context.Context, wallet types.ETHWallet, key string, value string) (*sdktypes.Receipt, error)
	AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error)
	Initialize(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)

	Indexer(ethClient *ethclient.Client, bvsContractAddr string, startBlockHeight uint64, eventTypes []common.Hash, rateLimit rate.Limit, maxRetries int) *indexer.ETHIndexer
	EventHandler(ch chan *indexer.Event)
}

type ethStateBankImpl struct {
	io                    io.ETHChainIO
	registeredBVSContract string
	contractAddr          common.Address
	contractABI           *abi.ABI
}

func (e *ethStateBankImpl) GetEthUpdateState(key string) (string, error) {
	value, exists := ethUpdateState.Load(key)
	if !exists {
		return "", fmt.Errorf("does not exist: %s", key)
	}
	return value.(string), nil
}

func (e *ethStateBankImpl) GetStateMap() *sync.Map {
	return &ethUpdateState
}

func (e *ethStateBankImpl) IsBVSContractRegistered(ctx context.Context, addr common.Address) (bool, error) {
	var isRegistered bool
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "isBvsContractRegistered",
		Args:         []interface{}{addr},
	}, &isRegistered); err != nil {
		return false, err
	}
	return isRegistered, nil
}

func (e *ethStateBankImpl) Get(ctx context.Context, key string) (string, error) {
	var value string
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "get",
		Args:         []interface{}{key},
	}, &value); err != nil {
		return "", err
	}
	return value, nil
}

func (e *ethStateBankImpl) Owner(ctx context.Context) (common.Address, error) {
	var addr common.Address
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "owner",
		Args:         []interface{}{},
	}, &addr); err != nil {
		return common.Address{}, err
	}
	return addr, nil
}
func (e *ethStateBankImpl) PendingOwner(ctx context.Context) (common.Address, error) {
	var addr common.Address
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "pendingOwner",
		Args:         []interface{}{},
	}, &addr); err != nil {
		return common.Address{}, err
	}
	return addr, nil
}

func (e *ethStateBankImpl) SetRegisteredBVSContract(ctx context.Context, wallet types.ETHWallet, contractAddr common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "addRegisteredBvsContract",
			Args:         []interface{}{contractAddr},
		},
	})
}

func (e *ethStateBankImpl) Set(ctx context.Context, wallet types.ETHWallet, key string, value string) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "set",
			Args:         []interface{}{key, value},
		},
	})
}

func (e *ethStateBankImpl) AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "acceptOwnership",
			Args:         []interface{}{},
		},
	})
}

func (e *ethStateBankImpl) RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "renounceOwnership",
			Args:         []interface{}{},
		},
	})
}

func (e *ethStateBankImpl) TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "transferOwnership",
			Args:         []interface{}{newOwner},
		},
	})
}

func (e *ethStateBankImpl) Initialize(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "initialize",
			Args:         []interface{}{},
		},
	})
}

func (e *ethStateBankImpl) Indexer(ethClient *ethclient.Client, bvsContractAddr string, startBlockHeight uint64, eventTypes []common.Hash, rateLimit rate.Limit, maxRetries int) *indexer.ETHIndexer {
	e.registeredBVSContract = bvsContractAddr
	return indexer.NewETHIndexer(ethClient, e.contractABI, e.contractAddr, startBlockHeight, eventTypes, rateLimit, maxRetries)
}

func (e *ethStateBankImpl) EventHandler(ch chan *indexer.Event) {
	for event := range ch {
		var sender string
		switch senderAttr := event.AttrMap["sender"].(type) {
		case common.Address:
			sender = senderAttr.Hex()
		case string:
			sender = senderAttr
		default:
			continue
		}

		if strings.ToLower(e.registeredBVSContract) != strings.ToLower(sender) {
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
		ethUpdateState.Store(key, val)
	}
}

func NewETHStateBankImpl(chainIO io.ETHChainIO, contractAddr common.Address, contractABI *abi.ABI) ETHStateBank {
	return &ethStateBankImpl{
		io:           chainIO,
		contractABI:  contractABI,
		contractAddr: contractAddr,
	}
}
