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

type StrategyFactory interface {
	WithGasAdjustment(gasAdjustment float64) StrategyFactory
	WithGasPrice(gasPrice sdktypes.DecCoin) StrategyFactory
	WithGasLimit(gasLimit uint64) StrategyFactory

	BindClient(string)

	DeployNewStrategy(ctx context.Context, token, pauser, unpauser string) (*coretypes.ResultTx, error)
	UpdateConfig(ctx context.Context, newOwner string, strategyCodeId int64) (*coretypes.ResultTx, error)
	BlacklistTokens(ctx context.Context, tokens []string) (*coretypes.ResultTx, error)
	RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error)
	SetThirdPartyTransfersForBidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error)
	WhitelistStrategies(ctx context.Context, strategies []string, forbiddenValues []bool) (*coretypes.ResultTx, error)
	SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)

	GetStrategy(token string) (*strategyfactory.StrategyResponse, error)
	IsTokenBlacklisted(token string) (*strategyfactory.BlacklistStatusResponse, error)
}

type strategyFactoryImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewStrategyFactory(chainIO io.ChainIO) StrategyFactory {
	return &strategyFactoryImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}

func (r *strategyFactoryImpl) WithGasAdjustment(gasAdjustment float64) StrategyFactory {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *strategyFactoryImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StrategyFactory {
	r.gasPrice = gasPrice
	return r
}

func (r *strategyFactoryImpl) WithGasLimit(gasLimit uint64) StrategyFactory {
	r.gasLimit = gasLimit
	return r
}

func (r *strategyFactoryImpl) BindClient(contractAddress string) {
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

func (r *strategyFactoryImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *strategyFactoryImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

// Execute Functions

func (r *strategyFactoryImpl) DeployNewStrategy(ctx context.Context, token, pauser, unpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		DeployNewStrategy: &strategyfactory.DeployNewStrategy{
			Token:    token,
			Pauser:   pauser,
			Unpauser: unpauser,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) UpdateConfig(ctx context.Context, newOwner string, strategyCodeId int64) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		UpdateConfig: &strategyfactory.UpdateConfig{
			NewOwner:       newOwner,
			StrategyCodeID: strategyCodeId,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) BlacklistTokens(ctx context.Context, tokens []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		BlacklistTokens: &strategyfactory.BlacklistTokens{
			Tokens: tokens,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategyfactory.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) SetThirdPartyTransfersForBidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetThirdPartyTransfersForBidden: &strategyfactory.SetThirdPartyTransfersForBidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) WhitelistStrategies(ctx context.Context, strategies []string, forbiddenValues []bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		WhitelistStrategies: &strategyfactory.WhitelistStrategies{
			StrategiesToWhitelist:              strategies,
			ThirdPartyTransfersForbiddenValues: forbiddenValues,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetStrategyManager: &strategyfactory.SetStrategyManager{
			NewStrategyManager: newStrategyManager,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		TransferOwnership: &strategyfactory.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Pause: &strategyfactory.Pause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Unpause: &strategyfactory.Unpause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetPauser: &strategyfactory.SetPauser{
			NewPauser: newPauser,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *strategyFactoryImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetUnpauser: &strategyfactory.SetUnpauser{
			NewUnpauser: newUnpauser,
		},
	}

	return r.execute(ctx, executeMsg)
}

// Query Functions

func (r *strategyFactoryImpl) GetStrategy(token string) (*strategyfactory.StrategyResponse, error) {
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

func (r *strategyFactoryImpl) IsTokenBlacklisted(token string) (*strategyfactory.BlacklistStatusResponse, error) {
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
