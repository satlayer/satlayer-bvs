package bvssquaringapi

import (
	"context"
	"math/big"
	"strconv"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

type BVSSquaring interface {
	CreateNewTask(ctx context.Context, wallet types.ETHWallet, input int64) (*sdktypes.Receipt, error)
	RespondToTask(ctx context.Context, wallet types.ETHWallet, taskId uint64, result int64, operators string) (*sdktypes.Receipt, error)
	GetTaskInput(context.Context, int64) (int64, error)
	GetTaskResult(context.Context, int64) (int64, error)
	GetLatestTaskID(context.Context) (int64, error)
}

type bvsSquaringImpl struct {
	io           io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (a *bvsSquaringImpl) CreateNewTask(ctx context.Context, wallet types.ETHWallet, input int64) (*sdktypes.Receipt, error) {
	return a.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: a.contractAddr,
			ContractABI:  a.contractABI,
			Method:       "createNewTask",
			Args:         []interface{}{strconv.FormatInt(input, 10)},
		},
	})
}

func (a *bvsSquaringImpl) RespondToTask(ctx context.Context, wallet types.ETHWallet, taskId uint64, result int64, operators string) (*sdktypes.Receipt, error) {
	return a.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: a.contractAddr,
			ContractABI:  a.contractABI,
			Method:       "respondToTask",
			Args:         []interface{}{big.NewInt(int64(taskId)), big.NewInt(result), operators},
		},
	})
}

func (a *bvsSquaringImpl) GetTaskInput(ctx context.Context, taskId int64) (int64, error) {
	resp := new(string)
	if err := a.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: a.contractAddr,
		ContractABI:  a.contractABI,
		Method:       "getTaskInput",
		Args:         []interface{}{big.NewInt(taskId)},
	}, &resp); err != nil {
		return 0, err
	}
	result, err := strconv.ParseInt(*resp, 10, 64)
	if err != nil {
		return 0, err
	}
	return result, nil
}

func (a *bvsSquaringImpl) GetTaskResult(ctx context.Context, taskId int64) (int64, error) {
	resp := new(big.Int)
	if err := a.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: a.contractAddr,
		ContractABI:  a.contractABI,
		Method:       "getTaskResult",
		Args:         []interface{}{big.NewInt(taskId)},
	}, &resp); err != nil {
		return 0, err
	}
	return resp.Int64(), nil
}

func (a *bvsSquaringImpl) GetLatestTaskID(ctx context.Context) (int64, error) {
	resp := new(big.Int)
	if err := a.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: a.contractAddr,
		ContractABI:  a.contractABI,
		Method:       "getLatestTaskId",
		Args:         []interface{}{},
	}, &resp); err != nil {
		return 0, err
	}
	return resp.Int64(), nil
}

func NewBVSSquaringImpl(chainIO io.ETHChainIO, contractAddr common.Address, contractABI *abi.ABI) BVSSquaring {
	return &bvsSquaringImpl{
		io:           chainIO,
		contractABI:  contractABI,
		contractAddr: contractAddr,
	}
}
