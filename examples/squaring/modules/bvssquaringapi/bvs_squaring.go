package bvssquaringapi

import (
	"context"
	"encoding/json"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type GetLatestTaskIDReq struct {
	GetLatestTaskID struct{} `json:"get_latest_task_id"`
}

type BVSSquaring interface {
	BindClient(string)
	CreateNewTask(context.Context, int64) (*coretypes.ResultTx, error)
	RespondToTask(ctx context.Context, taskId int64, result int64, operators string) (*coretypes.ResultTx, error)
	GetTaskInput(int64) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetTaskResult(int64) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetLatestTaskID() (*wasmtypes.QuerySmartContractStateResponse, error)
}

type bvsSquaringImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
}

func (a *bvsSquaringImpl) BindClient(contractAddress string) {
	a.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: 1.2,
		GasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		Gas:           300000,
		Memo:          "test tx",
		Simulate:      true,
	}

	a.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (a *bvsSquaringImpl) CreateNewTask(ctx context.Context, input int64) (*coretypes.ResultTx, error) {
	msg := CreateNewTaskReq{
		CreateNewTask: CreateNewTask{
			Input: input,
		},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes
	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *bvsSquaringImpl) RespondToTask(ctx context.Context, taskId int64, result int64, operators string) (*coretypes.ResultTx, error) {
	msg := RespondToTaskReq{
		RespondToTask: RespondToTask{
			TaskID:    taskId,
			Result:    result,
			Operators: operators,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes

	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *bvsSquaringImpl) GetTaskInput(taskId int64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := GetTaskInputReq{
		GetTaskInput: GetTaskInput{
			TaskID: taskId,
		},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *bvsSquaringImpl) GetTaskResult(taskId int64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := GetTaskResultReq{
		GetTaskResult: GetTaskResult{
			TaskID: taskId,
		},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *bvsSquaringImpl) GetLatestTaskID() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := GetLatestTaskIDReq{
		GetLatestTaskID: struct{}{},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func NewBVSSquaring(chainIO io.ChainIO) BVSSquaring {
	return &bvsSquaringImpl{
		io: chainIO,
	}
}
