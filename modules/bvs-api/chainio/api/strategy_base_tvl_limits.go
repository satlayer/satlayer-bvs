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

type StrategyBaseTvlLimits struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewStrategyBaseTVLLimits(chainIO io.ChainIO) *StrategyBaseTvlLimits {
	return &StrategyBaseTvlLimits{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *StrategyBaseTvlLimits) WithGasAdjustment(gasAdjustment float64) *StrategyBaseTvlLimits {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *StrategyBaseTvlLimits) WithGasPrice(gasPrice sdktypes.DecCoin) *StrategyBaseTvlLimits {
	r.gasPrice = gasPrice
	return r
}

func (r *StrategyBaseTvlLimits) WithGasLimit(gasLimit uint64) *StrategyBaseTvlLimits {
	r.gasLimit = gasLimit
	return r
}

func (r *StrategyBaseTvlLimits) BindClient(contractAddress string) {
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

func (r *StrategyBaseTvlLimits) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *StrategyBaseTvlLimits) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Deposit: &strategybasetvllimits.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Withdraw: &strategybasetvllimits.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) sendQuery(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *StrategyBaseTvlLimits) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetShares: &strategybasetvllimits.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		SharesToUnderlyingView: &strategybasetvllimits.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShareView: &strategybasetvllimits.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UserUnderlyingView: &strategybasetvllimits.UserUnderlyingView{
			User: user,
		},
	}

	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) GetTVLLimits() (*strategybasetvllimits.TVLLimitsResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTVLLimits: &strategybasetvllimits.GetTVLLimits{},
	}
	resp, err := r.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.TVLLimitsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) GetTotalDeposits() (*strategybasetvllimits.TotalSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTotalShares: &strategybasetvllimits.GetTotalShares{},
	}
	resp, err := r.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.TotalSharesResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) Explanation() (*strategybasetvllimits.ExplanationResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		Explanation: &strategybasetvllimits.Explanation{},
	}
	resp, err := r.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.ExplanationResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) GetStrategyState() (*strategybasetvllimits.StrategyState, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetStrategyState: &strategybasetvllimits.GetStrategyState{},
	}
	resp, err := r.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.StrategyState
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetUnderlyingToken: &strategybasetvllimits.GetUnderlyingToken{},
	}
	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) UnderlyingToShares(amount_underlying string) (*strategybasetvllimits.UnderlyingToSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShares: &strategybasetvllimits.UnderlyingToShares{
			AmountUnderlying: amount_underlying,
		},
	}

	resp, err := r.sendQuery(msg)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.UnderlyingToSharesResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) GetStrategyManager() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetStrategyManager: &strategybasetvllimits.GetStrategyManager{},
	}
	return r.sendQuery(msg)
}

func (r *StrategyBaseTvlLimits) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Pause: &strategybasetvllimits.Pause{},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Unpause: &strategybasetvllimits.Unpause{},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetPauser: &strategybasetvllimits.SetPauser{NewPauser: newPauser},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetUnpauser: &strategybasetvllimits.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) SetTVLLimits(ctx context.Context, maxPerDeposit string, maxTotalDeposits string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetTVLLimits: &strategybasetvllimits.SetTVLLimits{
			MaxPerDeposit:    maxPerDeposit,
			MaxTotalDeposits: maxTotalDeposits,
		},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetStrategyManager: &strategybasetvllimits.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return r.execute(ctx, msg)
}

func (r *StrategyBaseTvlLimits) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		TransferOwnership: &strategybasetvllimits.TransferOwnership{NewOwner: newOwner},
	}
	return r.execute(ctx, msg)
}
