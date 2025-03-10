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
	strategymanager "github.com/satlayer/satlayer-bvs/cosmwasm-schema/strategy-manager"
)

type StrategyManager struct {
	io             io.ChainIO
	ContractAddr   string
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewStrategyManager(chainIO io.ChainIO) *StrategyManager {
	return &StrategyManager{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *StrategyManager) WithGasAdjustment(gasAdjustment float64) *StrategyManager {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *StrategyManager) WithGasPrice(gasPrice sdktypes.DecCoin) *StrategyManager {
	r.gasPrice = gasPrice
	return r
}

func (r *StrategyManager) WithGasLimit(gasLimit uint64) *StrategyManager {
	r.gasLimit = gasLimit
	return r
}

func (r *StrategyManager) BindClient(contractAddress string) {
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

func (r *StrategyManager) UpdateStrategy(ctx context.Context, strategy string, whitelisted bool) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		UpdateStrategy: &strategymanager.UpdateStrategy{
			Strategy:    strategy,
			Whitelisted: whitelisted,
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) DepositIntoStrategy(ctx context.Context, strategy string, token string, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		DepositIntoStrategy: &strategymanager.DepositIntoStrategy{
			Strategy: strategy,
			Token:    token,
			Amount:   fmt.Sprintf("%d", amount),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) RemoveShares(ctx context.Context, staker string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		RemoveShares: &strategymanager.RemoveShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) WithdrawSharesAsTokens(ctx context.Context, recipient string, strategy string, shares string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		WithdrawSharesAsTokens: &strategymanager.WithdrawSharesAsTokens{
			Recipient: recipient,
			Strategy:  strategy,
			Shares:    shares,
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) AddShares(ctx context.Context, staker string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		AddShares: &strategymanager.AddShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) SetRouting(ctx context.Context, delegationManager, slashManager string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetRouting: &strategymanager.SetRouting{
			DelegationManager: delegationManager,
			SlashManager:      slashManager,
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyManager) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		TransferOwnership: &strategymanager.TransferOwnership{NewOwner: newOwner},
	}
	return r.execute(ctx, msg)
}

func (r *StrategyManager) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *StrategyManager) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *StrategyManager) StakerDepositList(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		StakerDepositList: &strategymanager.StakerDepositList{
			Staker: staker,
		},
	}

	return r.query(msg)
}

func (r *StrategyManager) StakerStrategyShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		StakerStrategyShares: &strategymanager.StakerStrategyShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return r.query(msg)
}

func (r *StrategyManager) StakerStrategyList(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		StakerStrategyList: &strategymanager.StakerStrategyList{
			Staker: staker,
		},
	}

	return r.query(msg)
}

func (r *StrategyManager) IsStrategyWhitelisted(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		IsStrategyWhitelisted: &strategy,
	}

	return r.query(msg)
}
