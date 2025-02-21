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
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewStrategyBase(chainIO io.ChainIO) *StrategyBase {
	return &StrategyBase{
		io:            chainIO,
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

func (r *StrategyBase) BindClient(contractAddress string) {
	r.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	r.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (r *StrategyBase) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *StrategyBase) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Deposit: &strategybase.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Withdraw: &strategybase.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) sendQuery(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *StrategyBase) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetShares: &strategybase.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		SharesToUnderlyingView: &strategybase.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UnderlyingToShareView: &strategybase.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UserUnderlyingView: &strategybase.UserUnderlyingView{
			User: user,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		GetUnderlyingToken: &strategybase.GetUnderlyingToken{},
	}
	return r.sendQuery(msg)
}

func (r *StrategyBase) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Pause: &strategybase.Pause{},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Unpause: &strategybase.Unpause{},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetPauser: &strategybase.SetPauser{NewPauser: newPauser},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetUnpauser: &strategybase.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		SetStrategyManager: &strategybase.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBase) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		TransferOwnership: &strategybase.TransferOwnership{NewOwner: newOwner}}
	return r.execute(ctx, msg)
}
