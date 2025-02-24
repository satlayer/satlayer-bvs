package api

import (
	"context"
	"encoding/json"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	strategyfactory "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-factory"
)

type StrategyFactory struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewStrategyFactory(chainIO io.ChainIO, contractAddr string) *StrategyFactory {
	return &StrategyFactory{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}

func (r *StrategyFactory) WithGasAdjustment(gasAdjustment float64) *StrategyFactory {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *StrategyFactory) WithGasPrice(gasPrice sdktypes.DecCoin) *StrategyFactory {
	r.gasPrice = gasPrice
	return r
}

func (r *StrategyFactory) WithGasLimit(gasLimit uint64) *StrategyFactory {
	r.gasLimit = gasLimit
	return r
}

// Execute Functions

func (r *StrategyFactory) DeployNewStrategy(ctx context.Context, token, pauser, unpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		DeployNewStrategy: &strategyfactory.DeployNewStrategy{
			Token:    token,
			Pauser:   pauser,
			Unpauser: unpauser,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "DeployNewStrategy")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) UpdateConfig(ctx context.Context, newOwner string, strategyCodeId int64) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		UpdateConfig: &strategyfactory.UpdateConfig{
			NewOwner:       newOwner,
			StrategyCodeID: strategyCodeId,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "UpdateConfig")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) BlacklistTokens(ctx context.Context, tokens []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		BlacklistTokens: &strategyfactory.BlacklistTokens{
			Tokens: tokens,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "BlacklistTokens")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategyfactory.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "RemoveStrategiesFromWhitelist")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) SetThirdPartyTransfersForbidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetThirdPartyTransfersForbidden: &strategyfactory.SetThirdPartyTransfersForbidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetThirdPartyTransfersForbidden")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) WhitelistStrategies(ctx context.Context, strategies []string, forbiddenValues []bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		WhitelistStrategies: &strategyfactory.WhitelistStrategies{
			StrategiesToWhitelist:              strategies,
			ThirdPartyTransfersForbiddenValues: forbiddenValues,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "WhitelistStrategies")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetStrategyManager: &strategyfactory.SetStrategyManager{
			NewStrategyManager: newStrategyManager,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		TransferOwnership: &strategyfactory.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Pause: &strategyfactory.Pause{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Unpause: &strategyfactory.Unpause{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetPauser: &strategyfactory.SetPauser{
			NewPauser: newPauser,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyFactory) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetUnpauser: &strategyfactory.SetUnpauser{
			NewUnpauser: newUnpauser,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

// Query Functions

func (r *StrategyFactory) GetStrategy(token string) (*strategyfactory.StrategyResponse, error) {
	queryMsg := strategyfactory.QueryMsg{
		GetStrategy: &strategyfactory.GetStrategy{
			Token: token,
		},
	}

	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	res, err := strategyfactory.UnmarshalStrategyResponse(resp.Data)
	return &res, err
}

func (r *StrategyFactory) IsTokenBlacklisted(token string) (*strategyfactory.BlacklistStatusResponse, error) {
	queryMsg := strategyfactory.QueryMsg{
		IsTokenBlacklisted: &strategyfactory.IsTokenBlacklisted{
			Token: token,
		},
	}

	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	res, err := strategyfactory.UnmarshalBlacklistStatusResponse(resp.Data)
	return &res, err
}

func (r *StrategyFactory) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *StrategyFactory) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
