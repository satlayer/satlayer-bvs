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

type StrategyBase interface {
	WithGasAdjustment(gasAdjustment float64) StrategyBase
	WithGasPrice(gasPrice sdktypes.DecCoin) StrategyBase
	WithGasLimit(gasLimit uint64) StrategyBase

	BindClient(string)
	Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error)
	Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error)
	GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error)
}

type strategyBaseImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (a *strategyBaseImpl) WithGasAdjustment(gasAdjustment float64) StrategyBase {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *strategyBaseImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StrategyBase {
	a.gasPrice = gasPrice
	return a
}

func (a *strategyBaseImpl) WithGasLimit(gasLimit uint64) StrategyBase {
	a.gasLimit = gasLimit
	return a
}

func (a *strategyBaseImpl) BindClient(contractAddress string) {
	a.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: a.gasAdjustment,
		GasPrice:      a.gasPrice,
		Gas:           a.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	a.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (a *strategyBaseImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes
	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *strategyBaseImpl) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Deposit: &strategybase.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Withdraw: &strategybase.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) sendQuery(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *strategyBaseImpl) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetShares: &strategybase.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseImpl) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		SharesToUnderlyingView: &strategybase.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseImpl) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UnderlyingToShareView: &strategybase.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseImpl) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UserUnderlyingView: &strategybase.UserUnderlyingView{
			User: user,
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseImpl) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetUnderlyingToken: &strategybase.GetUnderlyingToken{},
	}
	return a.sendQuery(msg)
}

func (a *strategyBaseImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Pause: &strategybase.Pause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Unpause: &strategybase.Unpause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetPauser: &strategybase.SetPauser{NewPauser: newPauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetUnpauser: &strategybase.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetStrategyManager: &strategybase.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		TransferOwnership: &strategybase.TransferOwnership{NewOwner: newOwner}}
	return a.execute(ctx, msg)
}

func NewStrategyBase(chainIO io.ChainIO) StrategyBase {
	return &strategyBaseImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
