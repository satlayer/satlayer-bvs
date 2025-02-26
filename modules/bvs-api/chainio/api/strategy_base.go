package api

import (
	"context"
	"encoding/json"
	"fmt"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	strategybase "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base"
)

type StrategyBase struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewStrategyBase(chainIO io.ChainIO, contractAddr string) *StrategyBase {
	return &StrategyBase{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *StrategyBase) WithGasAdjustment(gasAdjustment float64) *StrategyBase {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *StrategyBase) WithGasPrice(gasPrice sdktypes.DecCoin) *StrategyBase {
	r.gasPrice = gasPrice
	return r
}

func (r *StrategyBase) WithGasLimit(gasLimit uint64) *StrategyBase {
	r.gasLimit = gasLimit
	return r
}

func (r *StrategyBase) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Deposit: &strategybase.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Deposit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Withdraw: &strategybase.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Withdraw")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) PauseAll(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategybase.ExecuteMsg{PauseAll: &strategybase.PauseAll{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "PauseAll")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) UnpauseAll(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategybase.ExecuteMsg{UnpauseAll: &strategybase.UnpauseAll{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "UnpauseAll")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) PauseBit(ctx context.Context, index uint8) (*coretypes.ResultTx, error) {
	executeMsg := strategybase.ExecuteMsg{PauseBit: &strategybase.PauseBit{Index: index}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "PauseBit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) UnpauseBit(ctx context.Context, index uint8) (*coretypes.ResultTx, error) {
	executeMsg := strategybase.ExecuteMsg{UnpauseBit: &strategybase.UnpauseBit{Index: index}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "UnpauseBit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetPauser: &strategybase.SetPauser{NewPauser: newPauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetUnpauser: &strategybase.SetUnpauser{NewUnpauser: newUnpauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetStrategyManager: &strategybase.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		TransferOwnership: &strategybase.TransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBase) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetShares: &strategybase.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBase) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		SharesToUnderlyingView: &strategybase.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBase) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UnderlyingToShareView: &strategybase.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBase) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UserUnderlyingView: &strategybase.UserUnderlyingView{
			User: user,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBase) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetUnderlyingToken: &strategybase.GetUnderlyingToken{},
	}
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBase) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *StrategyBase) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
