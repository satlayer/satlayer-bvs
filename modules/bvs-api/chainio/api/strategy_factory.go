package api

import (
	"context"
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	strategyfactory "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-factory"
)

type StrategyFactory struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewStrategyFactory(chainIO io.ChainIO) *StrategyFactory {
	return &StrategyFactory{
		io:            chainIO,
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

func (r *StrategyFactory) BindClient(contractAddress string) {
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

func (r *StrategyFactory) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *StrategyFactory) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
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

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) UpdateConfig(ctx context.Context, newOwner string, strategyCodeId int64) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		UpdateConfig: &strategyfactory.UpdateConfig{
			NewOwner:       newOwner,
			StrategyCodeID: strategyCodeId,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) BlacklistTokens(ctx context.Context, tokens []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		BlacklistTokens: &strategyfactory.BlacklistTokens{
			Tokens: tokens,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategyfactory.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) SetThirdPartyTransfersForbidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetThirdPartyTransfersForbidden: &strategyfactory.SetThirdPartyTransfersForbidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) WhitelistStrategies(ctx context.Context, strategies []string, forbiddenValues []bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		WhitelistStrategies: &strategyfactory.WhitelistStrategies{
			StrategiesToWhitelist:              strategies,
			ThirdPartyTransfersForbiddenValues: forbiddenValues,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetStrategyManager: &strategyfactory.SetStrategyManager{
			NewStrategyManager: newStrategyManager,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		TransferOwnership: &strategyfactory.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Pause: &strategyfactory.Pause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Unpause: &strategyfactory.Unpause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetPauser: &strategyfactory.SetPauser{
			NewPauser: newPauser,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *StrategyFactory) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetUnpauser: &strategyfactory.SetUnpauser{
			NewUnpauser: newUnpauser,
		},
	}

	return r.execute(ctx, executeMsg)
}

// Query Functions

func (r *StrategyFactory) GetStrategy(token string) (*strategyfactory.StrategyResponse, error) {
	queryMsg := strategyfactory.QueryMsg{
		GetStrategy: &strategyfactory.GetStrategy{
			Token: token,
		},
	}

	resp, err := r.query(queryMsg)
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

	resp, err := r.query(queryMsg)
	if err != nil {
		return nil, err
	}

	res, err := strategyfactory.UnmarshalBlacklistStatusResponse(resp.Data)
	return &res, err
}
