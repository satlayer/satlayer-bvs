package api

import (
	"context"
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

type RewardsCoordinator interface {
	WithGasAdjustment(gasAdjustment float64) RewardsCoordinator
	WithGasPrice(gasPrice sdktypes.DecCoin) RewardsCoordinator
	WithGasLimit(gasLimit uint64) RewardsCoordinator

	BindClient(string)

	CreateBVSRewardsSubmission(ctx context.Context, submissions []types.RewardsSubmission) (*coretypes.ResultTx, error)
	CreateRewardsForAllSubmission(ctx context.Context, submissions []types.RewardsSubmission) (*coretypes.ResultTx, error)
	ProcessClaim(ctx context.Context, claim types.ExeuteRewardsMerkleClaim, recipient string) (*coretypes.ResultTx, error)
	SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp uint64) (*coretypes.ResultTx, error)
	DisableRoot(ctx context.Context, rootIndex uint64) (*coretypes.ResultTx, error)
	SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error)
	SetActivationDelay(ctx context.Context, newActivationDelay uint32) (*coretypes.ResultTx, error)
	SetGlobalOperatorCommission(ctx context.Context, newCommissionBips uint16) (*coretypes.ResultTx, error)
	SetRewardsUpdater(ctx context.Context, newUpdater string) (*coretypes.ResultTx, error)
	SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)

	CalculateEarnerLeafHash(earner string, earnerTokenRoot string) (*wasmtypes.QuerySmartContractStateResponse, error)
	CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error)
	OperatorCommissionBips(operator string, bvs string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error)
	CalculateDomainSeparator(chainId string, contractAddr string) (*wasmtypes.QuerySmartContractStateResponse, error)
	MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error)
	CheckClaim(claim types.ExeuteRewardsMerkleClaim) (*wasmtypes.QuerySmartContractStateResponse, error)
}

type rewardsCoordinatorImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (a *rewardsCoordinatorImpl) WithGasAdjustment(gasAdjustment float64) RewardsCoordinator {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *rewardsCoordinatorImpl) WithGasPrice(gasPrice sdktypes.DecCoin) RewardsCoordinator {
	a.gasPrice = gasPrice
	return a
}

func (a *rewardsCoordinatorImpl) WithGasLimit(gasLimit uint64) RewardsCoordinator {
	a.gasLimit = gasLimit
	return a
}

func (a *rewardsCoordinatorImpl) BindClient(contractAddress string) {
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

func (a *rewardsCoordinatorImpl) CreateBVSRewardsSubmission(ctx context.Context, submissions []types.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := types.CreateBVSRewardsSubmissionReq{
		CreateBVSRewardsSubmission: types.CreateRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) CreateRewardsForAllSubmission(ctx context.Context, submissions []types.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := types.CreateRewardsForAllSubmissionReq{
		CreateRewardsForAllSubmission: types.CreateRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) ProcessClaim(ctx context.Context, claim types.ExeuteRewardsMerkleClaim, recipient string) (*coretypes.ResultTx, error) {
	msg := types.ProcessClaimReq{
		ProcessClaim: types.ProcessClaim{
			Claim:     claim,
			Recipient: recipient,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp uint64) (*coretypes.ResultTx, error) {
	msg := types.SubmitRootReq{
		SubmitRoot: types.SubmitRoot{
			Root:                           root,
			RewardsCalculationEndTimestamp: rewardsCalculationEndTimestamp,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) DisableRoot(ctx context.Context, rootIndex uint64) (*coretypes.ResultTx, error) {
	msg := types.DisableRootReq{
		DisableRoot: types.DisableRoot{
			RootIndex: rootIndex,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error) {
	msg := types.SetClaimerForReq{
		SetClaimerFor: types.SetClaimerFor{
			Claimer: claimer,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetActivationDelay(ctx context.Context, newActivationDelay uint32) (*coretypes.ResultTx, error) {
	msg := types.SetActivationDelayReq{
		SetActivationDelay: types.SetActivationDelay{
			NewActivationDelay: newActivationDelay,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetGlobalOperatorCommission(ctx context.Context, newCommissionBips uint16) (*coretypes.ResultTx, error) {
	msg := types.SetGlobalOperatorCommissionReq{
		SetGlobalOperatorCommission: types.SetGlobalOperatorCommission{
			NewCommissionBips: newCommissionBips,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := types.PauseReq{}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := types.UnPauseReq{}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := types.SetPauserReq{
		SetPauser: types.SetPauser{NewPauser: newPauser},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := types.SetUnpauserReq{
		SetUnpauser: types.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := types.TransferRewardsCoordinatorOwnershipReq{
		TransferOwnership: types.TransferRewardsCoordinatorOwnership{
			NewOwner: newOwner,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetRewardsUpdater(ctx context.Context, newUpdater string) (*coretypes.ResultTx, error) {
	msg := types.SetRewardsUpdaterReq{
		SetRewardsUpdater: types.SetRewardsUpdater{
			NewUpdater: newUpdater,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error) {
	msg := types.SetRewardsForAllSubmitterReq{
		SetRewardsForAllSubmitter: types.SetRewardsForAllSubmitter{
			Submitter: submitter,
			NewValue:  newValue,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes
	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *rewardsCoordinatorImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *rewardsCoordinatorImpl) CalculateEarnerLeafHash(earner string, earnerTokenRoot string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.CalculateEarnerLeafHashReq{
		CalculateEarnerLeafHash: types.CalculateEarnerLeafHash{
			Earner:          earner,
			EarnerTokenRoot: earnerTokenRoot,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.CalculateTokenLeafHashReq{
		CalculateTokenLeafHash: types.CalculateTokenLeafHash{
			Token:              token,
			CumulativeEarnings: cumulativeEarnings,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) OperatorCommissionBips(operator string, bvs string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.OperatorCommissionBipsReq{
		OperatorCommissionBips: types.OperatorCommissionBips{
			Operator: operator,
			BVS:      bvs,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.GetDistributionRootsLengthReq{
		GetDistributionRootsLength: types.GetDistributionRootsLength{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.GetCurrentDistributionRootReq{
		GetCurrentDistributionRoot: types.GetCurrentDistributionRoot{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.GetDistributionRootAtIndexReq{
		GetDistributionRootAtIndex: types.GetDistributionRootAtIndex{
			Index: index,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.GetCurrentClaimableDistributionRootReq{
		GetCurrentClaimableDistributionRoot: types.GetCurrentClaimableDistributionRoot{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.GetRootIndexFromHashReq{
		GetRootIndexFromHash: types.GetRootIndexFromHash{
			RootHash: rootHash,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CalculateDomainSeparator(chainId string, contractAddr string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.CalculateDomainSeparatorReq{
		CalculateDomainSeparator: types.CalculateDomainSeparator{
			ChainId:      chainId,
			ContractAddr: contractAddr,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.MerkleizeLeavesReq{
		MerkleizeLeaves: types.MerkleizeLeaves{
			Leaves: leaves,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CheckClaim(claim types.ExeuteRewardsMerkleClaim) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := types.CheckClaimReq{
		CheckClaim: types.CheckClaim{
			Claim: claim,
		},
	}

	return a.query(msg)
}

func NewRewardsCoordinator(chainIO io.ChainIO) RewardsCoordinator {
	return &rewardsCoordinatorImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
