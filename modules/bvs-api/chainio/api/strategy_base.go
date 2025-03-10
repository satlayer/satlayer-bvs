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
	strategybase "github.com/satlayer/satlayer-bvs/cosmwasm-schema/strategy-base"
)

type StrategyBase struct {
	io             io.ChainIO
	ContractAddr   string
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

	r.ContractAddr = contractAddress
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

func (r *StrategyBase) Withdraw(ctx context.Context, recipient string, shares string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		Withdraw: &strategybase.Withdraw{
			Recipient: recipient,
			Shares:    shares,
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

func (r *StrategyBase) Shares(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		Shares: &strategybase.Shares{
			Staker: staker,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) Underlying(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		Underlying: &strategybase.Underlying{
			Staker: staker,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) SharesToUnderlying(shares string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		SharesToUnderlying: &strategybase.SharesToUnderlying{
			Shares: shares,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) UnderlyingToShares(amount string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UnderlyingToShares: &strategybase.UnderlyingToShares{
			Amount: amount,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBase) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybase.QueryMsg{
		UnderlyingToken: &strategybase.UnderlyingToken{},
	}
	return r.sendQuery(msg)
}

func (r *StrategyBase) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybase.ExecuteMsg{
		TransferOwnership: &strategybase.TransferOwnership{NewOwner: newOwner}}
	return r.execute(ctx, msg)
}
