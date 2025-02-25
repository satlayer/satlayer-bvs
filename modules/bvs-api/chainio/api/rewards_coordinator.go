package api

import (
	"context"
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"
)

type RewardsCoordinator struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewRewardsCoordinator(chainIO io.ChainIO, contractAddr string) *RewardsCoordinator {
	return &RewardsCoordinator{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *RewardsCoordinator) WithGasAdjustment(gasAdjustment float64) *RewardsCoordinator {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *RewardsCoordinator) WithGasPrice(gasPrice sdktypes.DecCoin) *RewardsCoordinator {
	r.gasPrice = gasPrice
	return r
}

func (r *RewardsCoordinator) WithGasLimit(gasLimit uint64) *RewardsCoordinator {
	r.gasLimit = gasLimit
	return r
}

func (r *RewardsCoordinator) CreateBVSRewardsSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateBvsRewardsSubmission: &rewardscoordinator.CreateBvsRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}
	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "CreateBVSRewardsSubmission")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) CreateRewardsForAllSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateRewardsForAllSubmission: &rewardscoordinator.CreateRewardsForAllSubmission{
			RewardsSubmissions: submissions,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "CreateRewardsForAllSubmission")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) ProcessClaim(ctx context.Context, claim rewardscoordinator.ProcessClaimClaim, recipient string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		ProcessClaim: &rewardscoordinator.ProcessClaim{
			Claim:     claim,
			Recipient: recipient,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "ProcessClaim")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SubmitRoot: &rewardscoordinator.SubmitRoot{
			Root:                           root,
			RewardsCalculationEndTimestamp: rewardsCalculationEndTimestamp,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SubmitRoot")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) DisableRoot(ctx context.Context, rootIndex int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		DisableRoot: &rewardscoordinator.DisableRoot{
			RootIndex: rootIndex,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "DisableRoot")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetClaimerFor: &rewardscoordinator.SetClaimerFor{
			Claimer: claimer,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetClaimerFor")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetActivationDelay(ctx context.Context, newActivationDelay int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetActivationDelay: &rewardscoordinator.SetActivationDelay{
			NewActivationDelay: newActivationDelay,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetActivationDelay")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetGlobalOperatorCommission(ctx context.Context, newCommissionBips int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetGlobalOperatorCommission: &rewardscoordinator.SetGlobalOperatorCommission{
			NewCommissionBips: newCommissionBips,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetGlobalOperatorCommission")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Pause: &rewardscoordinator.Pause{},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Unpause: &rewardscoordinator.Unpause{},
	}
	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetPauser: &rewardscoordinator.SetPauser{
			NewPauser: newPauser,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetUnpauser: &rewardscoordinator.SetUnpauser{NewUnpauser: newUnpauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) TwoStepTransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := rewardscoordinator.ExecuteMsg{
		TwoStepTransferOwnership: &rewardscoordinator.TwoStepTransferOwnership{
			NewOwner: newOwner,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TwoStepTransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) AcceptOwnership(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := rewardscoordinator.ExecuteMsg{
		AcceptOwnership: &rewardscoordinator.AcceptOwnership{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "AcceptOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) CancelOwnershipTransfer(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := rewardscoordinator.ExecuteMsg{
		CancelOwnershipTransfer: &rewardscoordinator.CancelOwnershipTransfer{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "CancelOwnershipTransfer")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetRewardsUpdater(ctx context.Context, newUpdater string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsUpdater: &rewardscoordinator.SetRewardsUpdater{
			NewUpdater: newUpdater,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetRewardsUpdater")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsForAllSubmitter: &rewardscoordinator.SetRewardsForAllSubmitter{
			Submitter: submitter,
			NewValue:  newValue,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetRewardsForAllSubmitter")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *RewardsCoordinator) CalculateEarnerLeafHash(earner string, earnerTokenRoot string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateEarnerLeafHash: &rewardscoordinator.CalculateEarnerLeafHash{
			Earner:          earner,
			EarnerTokenRoot: earnerTokenRoot,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateTokenLeafHash: &rewardscoordinator.CalculateTokenLeafHash{
			Token:              token,
			CumulativeEarnings: cumulativeEarnings,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) OperatorCommissionBips(operator string, bvs string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		OperatorCommissionBips: &rewardscoordinator.OperatorCommissionBips{
			Operator: operator,
			Bvs:      bvs,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootsLength: &rewardscoordinator.GetDistributionRootsLength{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentDistributionRoot: &rewardscoordinator.GetCurrentDistributionRoot{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootAtIndex: &rewardscoordinator.GetDistributionRootAtIndex{
			Index: index,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentClaimableDistributionRoot: &rewardscoordinator.GetCurrentClaimableDistributionRoot{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetRootIndexFromHash: &rewardscoordinator.GetRootIndexFromHash{
			RootHash: rootHash,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) CalculateDomainSeparator(chainId string, contractAddr string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateDomainSeparator: &rewardscoordinator.CalculateDomainSeparator{
			ChainID:      chainId,
			ContractAddr: contractAddr,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		MerkleizeLeaves: &rewardscoordinator.MerkleizeLeaves{
			Leaves: leaves,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) CheckClaim(claim rewardscoordinator.CheckClaimClaim) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CheckClaim: &rewardscoordinator.CheckClaim{
			Claim: claim,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *RewardsCoordinator) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *RewardsCoordinator) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
