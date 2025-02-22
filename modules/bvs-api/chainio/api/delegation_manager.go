package api

import (
	"context"
	"encoding/base64"
	"encoding/json"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

const zeroValueAddr = "0"

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
	senderPublicKey cryptotypes.PubKey,
	deprecatedEarningsReceiver,
	delegationApprover,
	metadataURI string,
	stakerOptOutWindowBlocks int64,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		RegisterAsOperator: &delegationmanager.RegisterAsOperator{
			SenderPublicKey: base64.StdEncoding.EncodeToString(senderPublicKey.Bytes()),
			OperatorDetails: delegationmanager.ExecuteOperatorDetails{
				DeprecatedEarningsReceiver: deprecatedEarningsReceiver,
				DelegationApprover:         delegationApprover,
				StakerOptOutWindowBlocks:   stakerOptOutWindowBlocks,
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
	deprecatedEarningsReceiver, delegationApprover string,
	stakerOptOutWindowBlocks int64,
) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		ModifyOperatorDetails: &delegationmanager.ModifyOperatorDetails{
			NewOperatorDetails: delegationmanager.ExecuteOperatorDetails{
				DeprecatedEarningsReceiver: deprecatedEarningsReceiver,
				DelegationApprover:         delegationApprover,
				StakerOptOutWindowBlocks:   stakerOptOutWindowBlocks,
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

func (r *DelegationManager) DelegateTo(ctx context.Context, operator, approver, approverKeyName string, approverPublicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	stakerAccount, err := r.io.GetCurrentAccount()
	if err != nil {
		return nil, err
	}
	executeMsg := delegationmanager.ExecuteMsg{DelegateTo: &delegationmanager.DelegateTo{
		Params: delegationmanager.ExecuteDelegateParams{
			Staker:   stakerAccount.GetAddress().String(),
			Operator: operator,
		},
	}}
	if approver != zeroValueAddr && approverKeyName != "" && approverPublicKey != nil {
		nodeStatus, err := r.io.QueryNodeStatus(context.Background())
		if err != nil {
			return nil, err
		}
		expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000
		randomStr, err := utils.GenerateRandomString(16)
		if err != nil {
			return nil, err
		}
		salt := "salt" + randomStr
		digestHashParams := delegationmanager.QueryApproverDigestHashParams{
			Staker:            stakerAccount.GetAddress().String(),
			Operator:          operator,
			Approver:          approver,
			ApproverPublicKey: base64.StdEncoding.EncodeToString(approverPublicKey.Bytes()),
			ApproverSalt:      base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry:            expiry,
			ContractAddr:      r.ContractAddr,
		}
		hashBytes, err := r.DelegationApprovalDigestHash(digestHashParams)
		if err != nil {
			return nil, err
		}

		signature, err := r.io.GetSigner().SignByKeyName(hashBytes, approverKeyName)
		if err != nil {
			return nil, err
		}
		executeMsg.DelegateTo.Params.PublicKey = base64.StdEncoding.EncodeToString(approverPublicKey.Bytes())
		executeMsg.DelegateTo.Params.Salt = base64.StdEncoding.EncodeToString([]byte(salt))
		executeMsg.DelegateTo.ApproverSignatureAndExpiry = delegationmanager.ExecuteSignatureWithExpiry{
			Signature: signature,
			Expiry:    expiry,
		}
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}

	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "DelegateTo")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) DelegateToBySignature(
	ctx context.Context,
	operator, staker, stakerKeyName, approver, approverKeyName string,
	stakerPublicKey, approverPublicKey cryptotypes.PubKey,
) (*coretypes.ResultTx, error) {
	nodeStatus, err := r.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	stakerNonceResp, err := r.GetStakerNonce(staker)
	if err != nil {
		return nil, err
	}
	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000
	digestHashParams := delegationmanager.QueryStakerDigestHashParams{
		Staker:          staker,
		StakerNonce:     stakerNonceResp.Nonce,
		Operator:        operator,
		StakerPublicKey: base64.StdEncoding.EncodeToString(stakerPublicKey.Bytes()),
		Expiry:          expiry,
		ContractAddr:    r.ContractAddr,
	}
	stakerHashBytes, err := r.StakerDelegationDigestHash(digestHashParams)
	if err != nil {
		return nil, err
	}
	stakerSignature, err := r.io.GetSigner().SignByKeyName(stakerHashBytes, stakerKeyName)
	if err != nil {
		return nil, err
	}

	executeMsg := delegationmanager.ExecuteMsg{
		DelegateToBySignature: &delegationmanager.DelegateToBySignature{
			Params: delegationmanager.ExecuteDelegateParams{
				Staker:   staker,
				Operator: operator,
			},
			StakerPublicKey: base64.StdEncoding.EncodeToString(stakerPublicKey.Bytes()),
			StakerSignatureAndExpiry: delegationmanager.ExecuteSignatureWithExpiry{
				Signature: stakerSignature,
				Expiry:    expiry,
			},
		},
	}

	if approver != zeroValueAddr && approverKeyName != "" && approverPublicKey != nil {
		randomStr, err := utils.GenerateRandomString(16)
		if err != nil {
			return nil, err
		}
		salt := "salt" + randomStr
		approverDigestHashReq := delegationmanager.QueryApproverDigestHashParams{
			Staker:            staker,
			Operator:          operator,
			Approver:          approver,
			ApproverPublicKey: base64.StdEncoding.EncodeToString(approverPublicKey.Bytes()),
			ApproverSalt:      base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry:            expiry,
			ContractAddr:      r.ContractAddr,
		}
		approverHashBytes, err := r.DelegationApprovalDigestHash(approverDigestHashReq)
		if err != nil {
			return nil, err
		}
		approverSignature, err := r.io.GetSigner().SignByKeyName(approverHashBytes, approverKeyName)
		if err != nil {
			return nil, err
		}
		executeMsg.DelegateToBySignature.Params.PublicKey = base64.StdEncoding.EncodeToString(approverPublicKey.Bytes())
		executeMsg.DelegateToBySignature.Params.Salt = base64.StdEncoding.EncodeToString([]byte(salt))
		executeMsg.DelegateToBySignature.ApproverSignatureAndExpiry = delegationmanager.ExecuteSignatureWithExpiry{
			Signature: approverSignature,
			Expiry:    expiry,
		}
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "DelegateToBySignature")

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

func (r *DelegationManager) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		Pause: &delegationmanager.Pause{},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		Unpause: &delegationmanager.Unpause{},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetPauser: &delegationmanager.SetPauser{NewPauser: newPauser},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetUnpauser: &delegationmanager.SetUnpauser{NewUnpauser: newUnpauser},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *DelegationManager) SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error) {
	executeMsg := delegationmanager.ExecuteMsg{
		SetSlashManager: &delegationmanager.SetSlashManager{NewSlashManager: newSlashManager},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetSlashManager")

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

func (r *DelegationManager) DelegationApprover(operator string) (*delegationmanager.DelegationApproverResponse, error) {
	result := new(delegationmanager.DelegationApproverResponse)
	queryMsg := delegationmanager.QueryMsg{
		DelegationApprover: &delegationmanager.DelegationApprover{Operator: operator},
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

func (r *DelegationManager) CalculateCurrentStakerDelegationDigestHash(stakerDigestHashParams delegationmanager.QueryCurrentStakerDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := delegationmanager.QueryMsg{
		CalculateCurrentStakerDelegationDigestHash: &delegationmanager.CalculateCurrentStakerDelegationDigestHash{
			CurrentStakerDigestHashParams: stakerDigestHashParams,
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
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) StakerDelegationDigestHash(stakerDigestHashParams delegationmanager.QueryStakerDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := delegationmanager.QueryMsg{
		StakerDelegationDigestHash: &delegationmanager.StakerDelegationDigestHash{
			StakerDigestHashParams: stakerDigestHashParams,
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
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *DelegationManager) DelegationApprovalDigestHash(digestHashParams delegationmanager.QueryApproverDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := delegationmanager.QueryMsg{
		DelegationApprovalDigestHash: &delegationmanager.DelegationApprovalDigestHash{
			ApproverDigestHashParams: digestHashParams,
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
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, err
}

func (r *DelegationManager) GetStakerNonce(staker string) (*delegationmanager.StakerNonceResponse, error) {
	result := new(delegationmanager.StakerNonceResponse)
	queryMsg := delegationmanager.QueryMsg{
		GetStakerNonce: &delegationmanager.GetStakerNonce{Staker: staker},
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
