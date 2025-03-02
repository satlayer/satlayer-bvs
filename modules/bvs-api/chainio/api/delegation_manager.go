package api

import (
	"context"
	"encoding/json"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

type DelegationManager struct {
	io            io.ChainIO
	ContractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewDelegationManager(chainIO io.ChainIO, contractAddr string) *DelegationManager {
	return &DelegationManager{
		io:            chainIO,
		ContractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *DelegationManager) WithGasAdjustment(gasAdjustment float64) *DelegationManager {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *DelegationManager) WithGasPrice(gasPrice sdktypes.DecCoin) *DelegationManager {
	r.gasPrice = gasPrice
	return r
}

func (r *DelegationManager) WithGasLimit(gasLimit uint64) *DelegationManager {
	r.gasLimit = gasLimit
	return r
}

func (r *DelegationManager) RegisterAsOperator(
	ctx context.Context,
	metadataURI string,
	stakerOptOutWindowBlocks int64,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		RegisterAsOperator: &delegationmanager.RegisterAsOperator{
			OperatorDetails: delegationmanager.NewOperatorDetailsClass{
				StakerOptOutWindowBlocks: stakerOptOutWindowBlocks,
			},
			MetadataURI: metadataURI,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "RegisterAsOperator")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) ModifyOperatorDetails(
	ctx context.Context,
	stakerOptOutWindowBlocks int64,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		ModifyOperatorDetails: &delegationmanager.ModifyOperatorDetails{
			NewOperatorDetails: delegationmanager.NewOperatorDetailsClass{
				StakerOptOutWindowBlocks: stakerOptOutWindowBlocks,
			},
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "ModifyOperatorDetails")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) UpdateOperatorMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		UpdateOperatorMetadataURI: &delegationmanager.UpdateOperatorMetadataURI{MetadataURI: metadataURI},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "UpdateOperatorMetadataURI")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) DelegateTo(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{DelegateTo: &delegationmanager.DelegateTo{
		Operator: operator,
	}}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}

	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "DelegateTo")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) UnDelegate(ctx context.Context, staker string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		Undelegate: &delegationmanager.Undelegate{Staker: staker},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "UnDelegate")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) QueueWithdrawals(ctx context.Context, withdrawalParams []delegationmanager.QueuedWithdrawalParams) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		QueueWithdrawals: &delegationmanager.QueueWithdrawals{
			QueuedWithdrawalParams: withdrawalParams,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "QueueWithdrawals")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) CompleteQueuedWithdrawal(
	ctx context.Context,
	withdrawal delegationmanager.WithdrawalElement,
	tokens []string,
	middlewareTimesIndex int64,
	receiveAsTokens bool,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		CompleteQueuedWithdrawal: &delegationmanager.CompleteQueuedWithdrawal{
			Withdrawal:           withdrawal,
			Tokens:               tokens,
			MiddlewareTimesIndex: middlewareTimesIndex,
			ReceiveAsTokens:      receiveAsTokens,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "CompleteQueuedWithdrawal")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) CompleteQueuedWithdrawals(
	ctx context.Context,
	withdrawals []delegationmanager.WithdrawalElement,
	tokens [][]string,
	middlewareTimesIndexes []int64,
	receiveAsTokens []bool,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		CompleteQueuedWithdrawals: &delegationmanager.CompleteQueuedWithdrawals{
			Withdrawals:            withdrawals,
			Tokens:                 tokens,
			MiddlewareTimesIndexes: middlewareTimesIndexes,
			ReceiveAsTokens:        receiveAsTokens,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "CompleteQueuedWithdrawals")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) IncreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		IncreaseDelegatedShares: &delegationmanager.IncreaseDelegatedShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   shares,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "IncreaseDelegatedShares")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) DecreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		DecreaseDelegatedShares: &delegationmanager.DecreaseDelegatedShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   shares,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "DecreaseDelegatedShares")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetMinWithdrawalDelayBlocks(ctx context.Context, newMinWithdrawalDelayBlocks int64) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetMinWithdrawalDelayBlocks: &delegationmanager.SetMinWithdrawalDelayBlocks{
			NewMinWithdrawalDelayBlocks: newMinWithdrawalDelayBlocks,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetMinWithdrawalDelayBlocks")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetStrategyWithdrawalDelayBlocks(ctx context.Context, strategies []string, withdrawalDelayBlocks []int64) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetStrategyWithdrawalDelayBlocks: &delegationmanager.SetStrategyWithdrawalDelayBlocks{
			Strategies:            strategies,
			WithdrawalDelayBlocks: withdrawalDelayBlocks,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetStrategyWithdrawalDelayBlocks")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		TransferOwnership: &delegationmanager.TransferOwnership{NewOwner: newOwner},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetRouting(ctx context.Context, strategyManager, slashManager string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetRouting: &delegationmanager.SetRouting{
			StrategyManager: strategyManager,
			SlashManager:    slashManager,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetRouting")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) IsDelegated(staker string) (*delegationmanager.DelegatedResponse, error) {
	result := new(delegationmanager.DelegatedResponse)
	queryMsg := delegationmanager.QueryMsg{
		IsDelegated: &delegationmanager.IsDelegated{Staker: staker},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) IsOperator(operator string) (*delegationmanager.OperatorResponse, error) {
	result := new(delegationmanager.OperatorResponse)
	queryMsg := delegationmanager.QueryMsg{
		IsOperator: &delegationmanager.IsOperator{Operator: operator},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) OperatorDetails(operator string) (*delegationmanager.OperatorDetailsResponse, error) {
	result := new(delegationmanager.OperatorDetailsResponse)
	queryMsg := delegationmanager.QueryMsg{
		OperatorDetails: &delegationmanager.OperatorDetails{Operator: operator},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) StakerOptOutWindowBlocks(operator string) (*delegationmanager.StakerOptOutWindowBlocksResponse, error) {
	result := new(delegationmanager.StakerOptOutWindowBlocksResponse)
	queryMsg := delegationmanager.QueryMsg{
		StakerOptOutWindowBlocks: &delegationmanager.StakerOptOutWindowBlocks{Operator: operator},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) GetOperatorShares(operator string, strategies []string) (*delegationmanager.OperatorSharesResponse, error) {
	result := new(delegationmanager.OperatorSharesResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetOperatorShares: &delegationmanager.GetOperatorShares{
			Operator:   operator,
			Strategies: strategies,
		},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) GetOperatorStakers(operator string) (*delegationmanager.OperatorStakersResponse, error) {
	result := new(delegationmanager.OperatorStakersResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetOperatorStakers: &delegationmanager.GetOperatorStakers{Operator: operator},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) GetDelegatableShares(staker string) (*delegationmanager.DelegatableSharesResponse, error) {
	result := new(delegationmanager.DelegatableSharesResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetDelegatableShares: &delegationmanager.GetDelegatableShares{Staker: staker},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) GetWithdrawalDelay(strategies []string) (*delegationmanager.WithdrawalDelayResponse, error) {
	result := new(delegationmanager.WithdrawalDelayResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetWithdrawalDelay: &delegationmanager.GetWithdrawalDelay{Strategies: strategies},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) CalculateWithdrawalRoot(withdrawal delegationmanager.CalculateWithdrawalRootWithdrawal) ([]byte, error) {
	var result []byte
	queryMsg := delegationmanager.QueryMsg{
		CalculateWithdrawalRoot: &delegationmanager.CalculateWithdrawalRoot{Withdrawal: withdrawal},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) GetCumulativeWithdrawalsQueuedNonce(staker string) (*delegationmanager.CumulativeWithdrawalsQueuedResponse, error) {
	result := new(delegationmanager.CumulativeWithdrawalsQueuedResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetCumulativeWithdrawalsQueued: &delegationmanager.GetCumulativeWithdrawalsQueued{Staker: staker},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (r *DelegationManager) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}
