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
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewStrategyBaseTVLLimits(chainIO io.ChainIO, contractAddr string) *StrategyBaseTvlLimits {
	return &StrategyBaseTvlLimits{
		io:            chainIO,
		contractAddr:  contractAddr,
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

func (r *StrategyBaseTvlLimits) Deposit(ctx context.Context, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Deposit: &strategybasetvllimits.Deposit{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Deposit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) Withdraw(ctx context.Context, recipient string, amountShares uint64) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		Withdraw: &strategybasetvllimits.Withdraw{
			Recipient:    recipient,
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Withdraw")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) PauseAll(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategybasetvllimits.ExecuteMsg{PauseAll: &strategybasetvllimits.PauseAll{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "PauseAll")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) UnpauseAll(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategybasetvllimits.ExecuteMsg{UnpauseAll: &strategybasetvllimits.UnpauseAll{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "UnpauseAll")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) PauseBit(ctx context.Context, index uint8) (*coretypes.ResultTx, error) {
	executeMsg := strategybasetvllimits.ExecuteMsg{PauseBit: &strategybasetvllimits.PauseBit{Index: index}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "PauseBit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) UnpauseBit(ctx context.Context, index uint8) (*coretypes.ResultTx, error) {
	executeMsg := strategybasetvllimits.ExecuteMsg{UnpauseBit: &strategybasetvllimits.UnpauseBit{Index: index}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "UnpauseBit")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetPauser: &strategybasetvllimits.SetPauser{NewPauser: newPauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetUnpauser: &strategybasetvllimits.SetUnpauser{NewUnpauser: newUnpauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) SetTvlLimits(ctx context.Context, maxPerDeposit string, maxTotalDeposits string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetTvlLimits: &strategybasetvllimits.SetTvlLimits{
			MaxPerDeposit:    maxPerDeposit,
			MaxTotalDeposits: maxTotalDeposits,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetTvlLimits")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		SetStrategyManager: &strategybasetvllimits.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategybasetvllimits.ExecuteMsg{
		TransferOwnership: &strategybasetvllimits.TransferOwnership{NewOwner: newOwner},
	}
	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyBaseTvlLimits) GetTvlLimits() (*strategybasetvllimits.TvlLimitsResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTvlLimits: &strategybasetvllimits.GetTvlLimits{},
	}
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.TvlLimitsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) GetTotalDeposits() (*strategybasetvllimits.TotalSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetTotalShares: &strategybasetvllimits.GetTotalShares{},
	}
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
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
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
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
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.StrategyState
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) UnderlyingToShares(amount_underlying string) (*strategybasetvllimits.UnderlyingToSharesResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShares: &strategybasetvllimits.UnderlyingToShares{
			AmountUnderlying: amount_underlying,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result strategybasetvllimits.UnderlyingToSharesResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *StrategyBaseTvlLimits) GetShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetShares: &strategybasetvllimits.GetShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) SharesToUnderlyingView(amountShares uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		SharesToUnderlyingView: &strategybasetvllimits.SharesToUnderlyingView{
			AmountShares: fmt.Sprintf("%d", amountShares),
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) UnderlyingToShareView(amount uint64) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UnderlyingToShareView: &strategybasetvllimits.UnderlyingToShareView{
			Amount: fmt.Sprintf("%d", amount),
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) UnderlyingView(user string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		UserUnderlyingView: &strategybasetvllimits.UserUnderlyingView{
			User: user,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) UnderlyingToken() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetUnderlyingToken: &strategybasetvllimits.GetUnderlyingToken{},
	}
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) GetStrategyManager() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategybasetvllimits.QueryMsg{
		GetStrategyManager: &strategybasetvllimits.GetStrategyManager{},
	}
	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyBaseTvlLimits) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *StrategyBaseTvlLimits) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
