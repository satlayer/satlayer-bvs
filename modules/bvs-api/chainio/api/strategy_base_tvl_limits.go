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
	strategybasetvllimits "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base-tvl-limits"
)

type StrategyBaseTVLLimits interface {
	WithGasAdjustment(gasAdjustment float64) StrategyBaseTVLLimits
	WithGasPrice(gasPrice sdktypes.DecCoin) StrategyBaseTVLLimits
	WithGasLimit(gasLimit uint64) StrategyBaseTVLLimits

	BindClient(string)
	Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error)
	Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error)
	GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetTVLLimits() (*strategybasetvllimits.TVLLimitsResponse, error)
	GetTotalDeposits() (*strategybasetvllimits.TotalSharesResponse, error)
	Explanation() (*strategybasetvllimits.ExplanationResponse, error)
	GetStrategyState() (*strategybasetvllimits.StrategyState, error)
	SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error)
	UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	SetTVLLimits(ctx context.Context, maxPerDeposit string, maxTotalDeposits string) (*coretypes.ResultTx, error)
	UnderlyingToShares(amount_underlying string) (*strategybasetvllimits.UnderlyingToSharesResponse, error)
	GetStrategyManager() (*wasmtypes.QuerySmartContractStateResponse, error)
	SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error)
}

type strategyBaseTVLLimitsImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (a *strategyBaseTVLLimitsImpl) WithGasAdjustment(gasAdjustment float64) StrategyBaseTVLLimits {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *strategyBaseTVLLimitsImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StrategyBaseTVLLimits {
	a.gasPrice = gasPrice
	return a
}

func (a *strategyBaseTVLLimitsImpl) WithGasLimit(gasLimit uint64) StrategyBaseTVLLimits {
	a.gasLimit = gasLimit
	return a
}

func (a *strategyBaseTVLLimitsImpl) BindClient(contractAddress string) {
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

func (a *strategyBaseTVLLimitsImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes
	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *strategyBaseTVLLimitsImpl) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Deposit: &strategybasetvllimits.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Withdraw: &strategybasetvllimits.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) sendQuery(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *strategyBaseTVLLimitsImpl) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetShares: &strategybasetvllimits.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		SharesToUnderlyingView: &strategybasetvllimits.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShareView: &strategybasetvllimits.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UserUnderlyingView: &strategybasetvllimits.UserUnderlyingView{
			User: user,
		},
	}

	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) GetTVLLimits() (*strategybasetvllimits.TVLLimitsResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTVLLimits: &strategybasetvllimits.GetTVLLimits{},
	}
	resp, err := a.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.TVLLimitsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (a *strategyBaseTVLLimitsImpl) GetTotalDeposits() (*strategybasetvllimits.TotalSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTotalShares: &strategybasetvllimits.GetTotalShares{},
	}
	resp, err := a.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.TotalSharesResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (a *strategyBaseTVLLimitsImpl) Explanation() (*strategybasetvllimits.ExplanationResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		Explanation: &strategybasetvllimits.Explanation{},
	}
	resp, err := a.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.ExplanationResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (a *strategyBaseTVLLimitsImpl) GetStrategyState() (*strategybasetvllimits.StrategyState, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetStrategyState: &strategybasetvllimits.GetStrategyState{},
	}
	resp, err := a.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.StrategyState
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (a *strategyBaseTVLLimitsImpl) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetUnderlyingToken: &strategybasetvllimits.GetUnderlyingToken{},
	}
	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) UnderlyingToShares(amount_underlying string) (*strategybasetvllimits.UnderlyingToSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShares: &strategybasetvllimits.UnderlyingToShares{
			AmountUnderlying: amount_underlying,
		},
	}

	resp, err := a.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.UnderlyingToSharesResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (a *strategyBaseTVLLimitsImpl) GetStrategyManager() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetStrategyManager: &strategybasetvllimits.GetStrategyManager{},
	}
	return a.sendQuery(msg)
}

func (a *strategyBaseTVLLimitsImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Pause: &strategybasetvllimits.Pause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Unpause: &strategybasetvllimits.Unpause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetPauser: &strategybasetvllimits.SetPauser{NewPauser: newPauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetUnpauser: &strategybasetvllimits.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) SetTVLLimits(ctx context.Context, maxPerDeposit string, maxTotalDeposits string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetTVLLimits: &strategybasetvllimits.SetTVLLimits{
			MaxPerDeposit:    maxPerDeposit,
			MaxTotalDeposits: maxTotalDeposits,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetStrategyManager: &strategybasetvllimits.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return a.execute(ctx, msg)
}

func (a *strategyBaseTVLLimitsImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		TransferOwnership: &strategybasetvllimits.TransferOwnership{NewOwner: newOwner},
	}
	return a.execute(ctx, msg)
}

func NewStrategyBaseTVLLimits(chainIO io.ChainIO) StrategyBaseTVLLimits {
	return &strategyBaseTVLLimitsImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
