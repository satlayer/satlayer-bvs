package api

import (
	"context"
	"encoding/json"

	strategyfactory "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-factory"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
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

	GetStrategy(token string) (*types.GetStrategyResponse, error)
	IsTokenBlacklisted(token string) (*types.BlacklistStatusResponse, error)
}

type strategyFactoryImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (s *strategyFactoryImpl) WithGasAdjustment(gasAdjustment float64) StrategyFactory {
	s.gasAdjustment = gasAdjustment
	return s
}

func (s *strategyFactoryImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StrategyFactory {
	s.gasPrice = gasPrice
	return s
}

func (s *strategyFactoryImpl) WithGasLimit(gasLimit uint64) StrategyFactory {
	s.gasLimit = gasLimit
	return s
}

func (s *strategyFactoryImpl) BindClient(contractAddress string) {
	s.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: s.gasAdjustment,
		GasPrice:      s.gasPrice,
		Gas:           s.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	s.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (s *strategyFactoryImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*s.executeOptions).ExecuteMsg = msgBytes
	return s.io.SendTransaction(ctx, *s.executeOptions)
}

func (s *strategyFactoryImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	(*s.queryOptions).QueryMsg = msgBytes
	return s.io.QueryContract(*s.queryOptions)
}

// Execute Functions

func (s *strategyFactoryImpl) DeployNewStrategy(ctx context.Context, token, pauser, unpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		DeployNewStrategy: &strategyfactory.DeployNewStrategy{
			Token:    token,
			Pauser:   pauser,
			Unpauser: unpauser,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) UpdateConfig(ctx context.Context, newOwner string, strategyCodeId int64) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		UpdateConfig: &strategyfactory.UpdateConfig{
			NewOwner:       newOwner,
			StrategyCodeID: strategyCodeId,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) BlacklistTokens(ctx context.Context, tokens []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		BlacklistTokens: &strategyfactory.BlacklistTokens{
			Tokens: tokens,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategyfactory.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) SetThirdPartyTransfersForBidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetThirdPartyTransfersForBidden: &strategyfactory.SetThirdPartyTransfersForBidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) WhitelistStrategies(ctx context.Context, strategies []string, forbiddenValues []bool) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		WhitelistStrategies: &strategyfactory.WhitelistStrategies{
			StrategiesToWhitelist:              strategies,
			ThirdPartyTransfersForbiddenValues: forbiddenValues,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetStrategyManager: &strategyfactory.SetStrategyManager{
			NewStrategyManager: newStrategyManager,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		TransferOwnership: &strategyfactory.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Pause: &strategyfactory.Pause{},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		Unpause: &strategyfactory.Unpause{},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetPauser: &strategyfactory.SetPauser{
			NewPauser: newPauser,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *strategyFactoryImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := strategyfactory.ExecuteMsg{
		SetUnpauser: &strategyfactory.SetUnpauser{
			NewUnpauser: newUnpauser,
		},
	}

	return s.execute(ctx, executeMsg)
}

// Query Functions

func (s *strategyFactoryImpl) GetStrategy(token string) (*types.GetStrategyResponse, error) {
	queryMsg := types.GetStrategyReq{
		GetStrategy: types.GetStrategy{
			Token: token,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.GetStrategyResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *strategyFactoryImpl) IsTokenBlacklisted(token string) (*types.BlacklistStatusResponse, error) {
	queryMsg := strategyfactory.QueryMsg{
		IsTokenBlacklisted: &strategyfactory.IsTokenBlacklisted{
			Token: token,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.BlacklistStatusResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func NewStrategyFactoryImpl(chainIO io.ChainIO, contractAddr string) StrategyFactory {
	return &strategyFactoryImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}
