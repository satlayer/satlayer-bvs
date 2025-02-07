package api

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"strconv"
	"time"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"golang.org/x/exp/rand"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

const zeroValueAddr = "0"

type Delegation interface {
	WithGasAdjustment(gasAdjustment float64) Delegation
	WithGasPrice(gasPrice sdktypes.DecCoin) Delegation
	WithGasLimit(gasLimit uint64) Delegation

	RegisterAsOperator(ctx context.Context, senderPublicKey cryptotypes.PubKey, deprecatedEarningsReceiver, delegationApprover,
		metadataURI string, stakerOptOutWindowBlocks uint64) (*coretypes.ResultTx, error)
	ModifyOperatorDetails(ctx context.Context, deprecatedEarningsReceiver, delegationApprover string, stakerOptOutWindowBlocks uint64) (*coretypes.ResultTx, error)
	UpdateOperatorMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error)
	DelegateTo(ctx context.Context, operator, approver, approverKeyName string, approverPublicKey cryptotypes.PubKey) (*coretypes.ResultTx, error)
	DelegateToBySignature(ctx context.Context, operator, staker, stakerKeyName, approver, approverKeyName string, stakerPublicKey,
		approverPublicKey cryptotypes.PubKey) (*coretypes.ResultTx, error)
	UnDelegate(ctx context.Context, staker string) (*coretypes.ResultTx, error)
	QueueWithdrawals(ctx context.Context, queuedWithdrawalParams []types.QueuedWithdrawalParams) (*coretypes.ResultTx, error)
	CompleteQueuedWithdrawal(ctx context.Context, withdrawal types.Withdrawal, tokens []string, middlewareTimesIndex uint64, receiveAsTokens bool) (*coretypes.ResultTx, error)
	CompleteQueuedWithdrawals(ctx context.Context, withdrawals []types.Withdrawal, tokens [][]string, middlewareTimesIndexes []uint64,
		receiveAsTokens []bool) (*coretypes.ResultTx, error)
	IncreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error)
	DecreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error)
	SetMinWithdrawalDelayBlocks(ctx context.Context, newMinWithdrawalDelayBlocks uint64) (*coretypes.ResultTx, error)
	SetStrategyWithdrawalDelayBlocks(ctx context.Context, strategies []string, withdrawalDelayBlocks []uint64) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error)

	IsDelegated(staker string) (*types.IsDelegatedResp, error)
	IsOperator(operator string) (*types.IsOperatorResp, error)
	OperatorDetails(operator string) (*types.OperatorDetailsResp, error)
	DelegationApprover(operator string) (*types.DelegationApproverResp, error)
	StakerOptOutWindowBlocks(operator string) (*types.StakerOptOutWindowBlocksResp, error)
	GetOperatorShares(operator string, strategies []string) (*types.GetOperatorSharesResp, error)
	GetOperatorStakers(operator string) (*types.GetOperatorStakersResp, error)
	GetDelegatableShares(staker string) (*types.GetDelegatableSharesResp, error)
	GetWithdrawalDelay(strategies []string) (*types.GetWithdrawalDelayResp, error)
	CalculateWithdrawalRoot(withdrawal types.Withdrawal) ([]byte, error)
	CalculateCurrentStakerDelegationDigestHash(currentStakerDigestHashParams types.CurrentStakerDigestHashParams) ([]byte, error)
	StakerDelegationDigestHash(stakerDigestHashParams types.StakerDigestHashParams) ([]byte, error)
	DelegationApprovalDigestHash(approverDigestHashParams types.ApproverDigestHashParams) ([]byte, error)
	GetStakerNonce(staker string) (*types.GetStakerNonceResp, error)
	GetCumulativeWithdrawalsQueuedNonce(staker string) (*types.GetCumulativeWithdrawalsQueuedNonceResp, error)
}

type delegationImpl struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func (d *delegationImpl) WithGasAdjustment(gasAdjustment float64) Delegation {
	d.gasAdjustment = gasAdjustment
	return d
}

func (d *delegationImpl) WithGasPrice(gasPrice sdktypes.DecCoin) Delegation {
	d.gasPrice = gasPrice
	return d
}

func (d *delegationImpl) WithGasLimit(gasLimit uint64) Delegation {
	d.gasLimit = gasLimit
	return d
}

func (d *delegationImpl) RegisterAsOperator(
	ctx context.Context,
	senderPublicKey cryptotypes.PubKey,
	deprecatedEarningsReceiver,
	delegationApprover,
	metadataURI string,
	stakerOptOutWindowBlocks uint64,
) (*coretypes.ResultTx, error) {
	executeMsg := types.RegisterAsOperatorReq{RegisterAsOperator: types.RegisterAsOperator{
		SenderPublicKey: base64.StdEncoding.EncodeToString(senderPublicKey.Bytes()),
		OperatorDetails: types.OperatorDetails{
			DeprecatedEarningsReceiver: deprecatedEarningsReceiver,
			DelegationApprover:         delegationApprover,
			StakerOptOutWindowBlocks:   stakerOptOutWindowBlocks,
		},
		MetadataURI: metadataURI,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "RegisterAsOperator")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) ModifyOperatorDetails(
	ctx context.Context,
	deprecatedEarningsReceiver, delegationApprover string,
	stakerOptOutWindowBlocks uint64,
) (*coretypes.ResultTx, error) {
	executeMsg := types.ModifyOperatorDetailsReq{
		ModifyOperatorDetails: types.ModifyOperatorDetails{
			NewOperatorDetails: types.OperatorDetails{
				DeprecatedEarningsReceiver: deprecatedEarningsReceiver,
				DelegationApprover:         delegationApprover,
				StakerOptOutWindowBlocks:   stakerOptOutWindowBlocks,
			}}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "ModifyOperatorDetails")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) UpdateOperatorMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := types.UpdateOperatorMetadataURIReq{UpdateOperatorMetadataURI: types.UpdateOperatorMetadataURI{MetadataURI: metadataURI}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "UpdateOperatorMetadataURI")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) DelegateTo(ctx context.Context, operator, approver, approverKeyName string, approverPublicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	stakerAccount, err := d.io.GetCurrentAccount()
	if err != nil {
		return nil, err
	}
	executeMsg := types.DelegateToReq{DelegateTo: types.DelegateTo{
		Params: types.DelegateParams{
			Staker:   stakerAccount.GetAddress().String(),
			Operator: operator,
		},
	}}
	if approver != zeroValueAddr && approverKeyName != "" && approverPublicKey != nil {
		nodeStatus, err := d.io.QueryNodeStatus(context.Background())
		if err != nil {
			return nil, err
		}
		expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
		salt := "salt" + strconv.FormatUint(rand.New(rand.NewSource(uint64(time.Now().Unix()))).Uint64(), 10)
		approverDigestHashReq := types.ApproverDigestHashParams{
			Staker:            stakerAccount.GetAddress().String(),
			Operator:          operator,
			Approver:          approver,
			ApproverPublicKey: base64.StdEncoding.EncodeToString(approverPublicKey.Bytes()),
			ApproverSalt:      base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry:            expiry,
			ContractAddr:      d.contractAddr,
		}
		hashBytes, err := d.DelegationApprovalDigestHash(approverDigestHashReq)
		if err != nil {
			return nil, err
		}

		signature, err := d.io.GetSigner().SignByKeyName(hashBytes, approverKeyName)
		if err != nil {
			return nil, err
		}
		executeMsg.DelegateTo.Params.PublicKey = base64.StdEncoding.EncodeToString(approverPublicKey.Bytes())
		executeMsg.DelegateTo.Params.Salt = base64.StdEncoding.EncodeToString([]byte(salt))
		executeMsg.DelegateTo.ApproverSignatureAndExpiry = types.SignatureWithExpiry{
			Signature: signature,
			Expiry:    expiry,
		}
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}

	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "DelegateTo")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) DelegateToBySignature(
	ctx context.Context,
	operator, staker, stakerKeyName, approver, approverKeyName string,
	stakerPublicKey, approverPublicKey cryptotypes.PubKey,
) (*coretypes.ResultTx, error) {
	nodeStatus, err := d.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	stakerNonceResp, err := d.GetStakerNonce(staker)
	if err != nil {
		return nil, err
	}
	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
	stakerDigestHashReq := types.StakerDigestHashParams{
		Staker:          staker,
		StakerNonce:     stakerNonceResp.Nonce,
		Operator:        operator,
		StakerPublicKey: base64.StdEncoding.EncodeToString(stakerPublicKey.Bytes()),
		Expiry:          expiry,
		ContractAddr:    d.contractAddr,
	}
	stakerHashBytes, err := d.StakerDelegationDigestHash(stakerDigestHashReq)
	if err != nil {
		return nil, err
	}
	stakerSignature, err := d.io.GetSigner().SignByKeyName(stakerHashBytes, stakerKeyName)
	if err != nil {
		return nil, err
	}

	executeMsg := types.DelegateToBySignatureReq{DelegateToBySignature: types.DelegateToBySignature{
		Params: types.DelegateParams{
			Staker:   staker,
			Operator: operator,
		},
		StakerPublicKey: base64.StdEncoding.EncodeToString(stakerPublicKey.Bytes()),
		StakerSignatureAndExpiry: types.SignatureWithExpiry{
			Signature: stakerSignature,
			Expiry:    expiry,
		},
	}}

	if approver != zeroValueAddr && approverKeyName != "" && approverPublicKey != nil {
		salt := "salt" + strconv.FormatUint(rand.New(rand.NewSource(uint64(time.Now().Unix()))).Uint64(), 10)
		approverDigestHashReq := types.ApproverDigestHashParams{
			Staker:            staker,
			Operator:          operator,
			Approver:          approver,
			ApproverPublicKey: base64.StdEncoding.EncodeToString(approverPublicKey.Bytes()),
			ApproverSalt:      base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry:            expiry,
			ContractAddr:      d.contractAddr,
		}
		approverHashBytes, err := d.DelegationApprovalDigestHash(approverDigestHashReq)
		if err != nil {
			return nil, err
		}
		approverSignature, err := d.io.GetSigner().SignByKeyName(approverHashBytes, approverKeyName)
		if err != nil {
			return nil, err
		}
		executeMsg.DelegateToBySignature.Params.PublicKey = base64.StdEncoding.EncodeToString(approverPublicKey.Bytes())
		executeMsg.DelegateToBySignature.Params.Salt = base64.StdEncoding.EncodeToString([]byte(salt))
		executeMsg.DelegateToBySignature.ApproverSignatureAndExpiry = types.SignatureWithExpiry{
			Signature: approverSignature,
			Expiry:    expiry,
		}
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "DelegateToBySignature")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) UnDelegate(ctx context.Context, staker string) (*coretypes.ResultTx, error) {
	executeMsg := types.UnDelegateReq{UnDelegate: types.UnDelegate{Staker: staker}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "UnDelegate")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) QueueWithdrawals(ctx context.Context, queuedWithdrawalParams []types.QueuedWithdrawalParams) (*coretypes.ResultTx, error) {
	executeMsg := types.QueueWithdrawalsReq{QueueWithdrawals: types.QueueWithdrawals{QueuedWithdrawalParams: queuedWithdrawalParams}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "QueueWithdrawals")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) CompleteQueuedWithdrawal(
	ctx context.Context,
	withdrawal types.Withdrawal,
	tokens []string,
	middlewareTimesIndex uint64,
	receiveAsTokens bool,
) (*coretypes.ResultTx, error) {
	executeMsg := types.CompleteQueuedWithdrawalReq{CompleteQueuedWithdrawal: types.CompleteQueuedWithdrawal{
		Withdrawal:           withdrawal,
		Tokens:               tokens,
		MiddlewareTimesIndex: middlewareTimesIndex,
		ReceiveAsTokens:      receiveAsTokens,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "CompleteQueuedWithdrawal")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) CompleteQueuedWithdrawals(
	ctx context.Context,
	withdrawals []types.Withdrawal,
	tokens [][]string,
	middlewareTimesIndexes []uint64,
	receiveAsTokens []bool,
) (*coretypes.ResultTx, error) {
	executeMsg := types.CompleteQueuedWithdrawalsReq{CompleteQueuedWithdrawals: types.CompleteQueuedWithdrawals{
		Withdrawals:            withdrawals,
		Tokens:                 tokens,
		MiddlewareTimesIndexes: middlewareTimesIndexes,
		ReceiveAsTokens:        receiveAsTokens,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "CompleteQueuedWithdrawals")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) IncreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error) {
	executeMsg := types.IncreaseDelegatedSharesReq{IncreaseDelegatedShares: types.DelegatedShares{
		Staker:   staker,
		Strategy: strategy,
		Shares:   shares,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "IncreaseDelegatedShares")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) DecreaseDelegatedShares(ctx context.Context, staker, strategy, shares string) (*coretypes.ResultTx, error) {
	executeMsg := types.DecreaseDelegatedSharesReq{DecreaseDelegatedShares: types.DelegatedShares{
		Staker:   staker,
		Strategy: strategy,
		Shares:   shares,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "DecreaseDelegatedShares")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) SetMinWithdrawalDelayBlocks(ctx context.Context, newMinWithdrawalDelayBlocks uint64) (*coretypes.ResultTx, error) {
	executeMsg := types.SetMinWithdrawalDelayBlocksReq{SetMinWithdrawalDelayBlocks: types.SetMinWithdrawalDelayBlocks{
		NewMinWithdrawalDelayBlocks: newMinWithdrawalDelayBlocks,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "SetMinWithdrawalDelayBlocks")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) SetStrategyWithdrawalDelayBlocks(ctx context.Context, strategies []string, withdrawalDelayBlocks []uint64) (*coretypes.ResultTx, error) {
	executeMsg := types.SetStrategyWithdrawalDelayBlocksReq{SetStrategyWithdrawalDelayBlocks: types.SetStrategyWithdrawalDelayBlocks{
		Strategies:            strategies,
		WithdrawalDelayBlocks: withdrawalDelayBlocks,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "SetStrategyWithdrawalDelayBlocks")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegateTransferOwnershipReq{DelegateTransferOwnership: types.DelegateTransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "TransferOwnership")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegationPauseReq{}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "Pause")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegationUnpauseReq{}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "Unpause")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegationSetPauserReq{SetPauser: types.DelegationSetPauser{NewPauser: newPauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "SetPauser")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegationSetUnpauserReq{SetUnpauser: types.DelegationSetUnpauser{NewUnpauser: newUnpauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "SetUnpauser")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error) {
	executeMsg := types.DelegationSetSlashManagerReq{SetSlashManager: types.DelegationSetSlashManager{NewSlashManager: newSlashManager}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := d.newExecuteOptions(d.contractAddr, executeMsgBytes, "SetSlashManager")

	return d.io.SendTransaction(ctx, executeOptions)
}

func (d *delegationImpl) IsDelegated(staker string) (*types.IsDelegatedResp, error) {
	result := new(types.IsDelegatedResp)
	queryMsg := types.IsDelegatedReq{IsDelegated: types.IsDelegated{Staker: staker}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) IsOperator(operator string) (*types.IsOperatorResp, error) {
	result := new(types.IsOperatorResp)
	queryMsg := types.IsOperatorReq{IsOperator: types.IsOperator{Operator: operator}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) OperatorDetails(operator string) (*types.OperatorDetailsResp, error) {
	result := new(types.OperatorDetailsResp)
	queryMsg := types.OperatorDetailsReq{QueryOperatorDetails: types.QueryOperatorDetails{Operator: operator}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) DelegationApprover(operator string) (*types.DelegationApproverResp, error) {
	result := new(types.DelegationApproverResp)
	queryMsg := types.DelegationApproverReq{DelegationApprover: types.DelegationApprover{Operator: operator}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) StakerOptOutWindowBlocks(operator string) (*types.StakerOptOutWindowBlocksResp, error) {
	result := new(types.StakerOptOutWindowBlocksResp)
	queryMsg := types.StakerOptOutWindowBlocksReq{StakerOptOutWindowBlocks: types.StakerOptOutWindowBlocks{Operator: operator}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) GetOperatorShares(operator string, strategies []string) (*types.GetOperatorSharesResp, error) {
	result := new(types.GetOperatorSharesResp)
	queryMsg := types.GetOperatorSharesReq{GetOperatorShares: types.GetOperatorShares{
		Operator:   operator,
		Strategies: strategies,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) GetOperatorStakers(operator string) (*types.GetOperatorStakersResp, error) {
	result := new(types.GetOperatorStakersResp)
	queryMsg := types.GetOperatorStakersReq{GetOperatorStakers: types.GetOperatorStakers{Operator: operator}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) GetDelegatableShares(staker string) (*types.GetDelegatableSharesResp, error) {
	result := new(types.GetDelegatableSharesResp)
	queryMsg := types.GetDelegatableSharesReq{GetDelegatableShares: types.GetDelegatableShares{Staker: staker}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) GetWithdrawalDelay(strategies []string) (*types.GetWithdrawalDelayResp, error) {
	result := new(types.GetWithdrawalDelayResp)
	queryMsg := types.GetWithdrawalDelayReq{GetWithdrawalDelay: types.GetWithdrawalDelay{Strategies: strategies}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) CalculateWithdrawalRoot(withdrawal types.Withdrawal) ([]byte, error) {
	var result []byte
	queryMsg := types.CalculateWithdrawalRootReq{CalculateWithdrawalRoot: types.CalculateWithdrawalRoot{Withdrawal: withdrawal}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) CalculateCurrentStakerDelegationDigestHash(currentStakerDigestHashParams types.CurrentStakerDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := types.CalculateCurrentStakerDelegationDigestHashReq{
		CalculateCurrentStakerDelegationDigestHash: types.CalculateCurrentStakerDelegationDigestHash{
			CurrentStakerDigestHashParams: currentStakerDigestHashParams,
		}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) StakerDelegationDigestHash(stakerDigestHashParams types.StakerDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := types.StakerDelegationDigestHashReq{StakerDelegationDigestHash: types.StakerDelegationDigestHash{
		StakerDigestHashParams: stakerDigestHashParams,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) DelegationApprovalDigestHash(approverDigestHashParams types.ApproverDigestHashParams) ([]byte, error) {
	var result []byte
	queryMsg := types.DelegationApprovalDigestHashReq{DelegationApprovalDigestHash: types.DelegationApprovalDigestHash{
		ApproverDigestHashParams: approverDigestHashParams,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, err
}

func (d *delegationImpl) GetStakerNonce(staker string) (*types.GetStakerNonceResp, error) {
	result := new(types.GetStakerNonceResp)
	queryMsg := types.GetStakerNonceReq{GetStakerNonce: types.GetStakerNonce{Staker: staker}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) GetCumulativeWithdrawalsQueuedNonce(staker string) (*types.GetCumulativeWithdrawalsQueuedNonceResp, error) {
	result := new(types.GetCumulativeWithdrawalsQueuedNonceResp)
	queryMsg := types.GetCumulativeWithdrawalsQueuedNonceReq{
		GetCumulativeWithdrawalsQueuedNonce: types.GetCumulativeWithdrawalsQueuedNonce{Staker: staker},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := d.newQueryOptions(d.contractAddr, queryMsgBytes)
	resp, err := d.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}
	return result, nil
}

func (d *delegationImpl) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: d.gasAdjustment,
		GasPrice:      d.gasPrice,
		Gas:           d.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (d *delegationImpl) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}

func NewDelegationImpl(chainIO io.ChainIO, contractAddr string) Delegation {
	return &delegationImpl{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
