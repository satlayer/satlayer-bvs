package squaringcontract

import (
	"context"
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type BVSSquaring struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
}

func (a *BVSSquaring) BindClient(contractAddress string) {
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

func (a *BVSSquaring) CreateNewTask(ctx context.Context, input int64) (*coretypes.ResultTx, error) {
	msg := ExecuteMsg{
		CreateNewTask: &CreateNewTask{
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

func (a *BVSSquaring) RespondToTask(ctx context.Context, taskId int64, result int64, operators string) (*coretypes.ResultTx, error) {
	msg := ExecuteMsg{
		RespondToTask: &RespondToTask{
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

func (a *BVSSquaring) GetTaskInput(taskId int64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := QueryMsg{
		GetTaskInput: &GetTaskInput{
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

func (a *BVSSquaring) GetTaskResult(taskId int64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := QueryMsg{
		GetTaskResult: &GetTaskResult{
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

func (a *BVSSquaring) GetLatestTaskID() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := QueryMsg{
		GetLatestTaskID: &GetLatestTaskID{},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func New(chainIO io.ChainIO) BVSSquaring {
	return BVSSquaring{
		io: chainIO,
	}
}
