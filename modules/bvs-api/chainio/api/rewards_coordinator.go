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

type RewardsCoordinator interface {
	WithGasAdjustment(gasAdjustment float64) RewardsCoordinator
	WithGasPrice(gasPrice sdktypes.DecCoin) RewardsCoordinator
	WithGasLimit(gasLimit uint64) RewardsCoordinator

	BindClient(string)

	CreateBVSRewardsSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error)
	CreateRewardsForAllSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error)
	ProcessClaim(ctx context.Context, claim rewardscoordinator.ProcessClaimClaim, recipient string) (*coretypes.ResultTx, error)
	SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp int64) (*coretypes.ResultTx, error)
	DisableRoot(ctx context.Context, rootIndex int64) (*coretypes.ResultTx, error)
	SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error)
	SetActivationDelay(ctx context.Context, newActivationDelay int64) (*coretypes.ResultTx, error)
	SetGlobalOperatorCommission(ctx context.Context, newCommissionBips int64) (*coretypes.ResultTx, error)
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
	CheckClaim(claim rewardscoordinator.CheckClaimClaim) (*wasmtypes.QuerySmartContractStateResponse, error)
}

type rewardsCoordinatorImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewRewardsCoordinator(chainIO io.ChainIO) RewardsCoordinator {
	return &rewardsCoordinatorImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *rewardsCoordinatorImpl) WithGasAdjustment(gasAdjustment float64) RewardsCoordinator {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *rewardsCoordinatorImpl) WithGasPrice(gasPrice sdktypes.DecCoin) RewardsCoordinator {
	r.gasPrice = gasPrice
	return r
}

func (r *rewardsCoordinatorImpl) WithGasLimit(gasLimit uint64) RewardsCoordinator {
	r.gasLimit = gasLimit
	return r
}

func (r *rewardsCoordinatorImpl) BindClient(contractAddress string) {
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

func (r *rewardsCoordinatorImpl) CreateBVSRewardsSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateBvsRewardsSubmission: &rewardscoordinator.CreateBvsRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) CreateRewardsForAllSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateRewardsForAllSubmission: &rewardscoordinator.CreateRewardsForAllSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) ProcessClaim(ctx context.Context, claim rewardscoordinator.ProcessClaimClaim, recipient string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		ProcessClaim: &rewardscoordinator.ProcessClaim{
			Claim:     claim,
			Recipient: recipient,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SubmitRoot: &rewardscoordinator.SubmitRoot{
			Root:                           root,
			RewardsCalculationEndTimestamp: rewardsCalculationEndTimestamp,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) DisableRoot(ctx context.Context, rootIndex int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		DisableRoot: &rewardscoordinator.DisableRoot{
			RootIndex: rootIndex,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetClaimerFor: &rewardscoordinator.SetClaimerFor{
			Claimer: claimer,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetActivationDelay(ctx context.Context, newActivationDelay int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetActivationDelay: &rewardscoordinator.SetActivationDelay{
			NewActivationDelay: newActivationDelay,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetGlobalOperatorCommission(ctx context.Context, newCommissionBips int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetGlobalOperatorCommission: &rewardscoordinator.SetGlobalOperatorCommission{
			NewCommissionBips: newCommissionBips,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Pause: &rewardscoordinator.Pause{},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Unpause: &rewardscoordinator.Unpause{},
	}
	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetPauser: &rewardscoordinator.SetPauser{
			NewPauser: newPauser,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetUnpauser: &rewardscoordinator.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		TransferOwnership: &rewardscoordinator.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetRewardsUpdater(ctx context.Context, newUpdater string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsUpdater: &rewardscoordinator.SetRewardsUpdater{
			NewUpdater: newUpdater,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsForAllSubmitter: &rewardscoordinator.SetRewardsForAllSubmitter{
			Submitter: submitter,
			NewValue:  newValue,
		},
	}

	return r.execute(ctx, msg)
}

func (r *rewardsCoordinatorImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *rewardsCoordinatorImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *rewardsCoordinatorImpl) CalculateEarnerLeafHash(earner string, earnerTokenRoot string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateEarnerLeafHash: &rewardscoordinator.CalculateEarnerLeafHash{
			Earner:          earner,
			EarnerTokenRoot: earnerTokenRoot,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateTokenLeafHash: &rewardscoordinator.CalculateTokenLeafHash{
			Token:              token,
			CumulativeEarnings: cumulativeEarnings,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) OperatorCommissionBips(operator string, bvs string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		OperatorCommissionBips: &rewardscoordinator.OperatorCommissionBips{
			Operator: operator,
			Bvs:      bvs,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootsLength: &rewardscoordinator.GetDistributionRootsLength{},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentDistributionRoot: &rewardscoordinator.GetCurrentDistributionRoot{},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootAtIndex: &rewardscoordinator.GetDistributionRootAtIndex{
			Index: index,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentClaimableDistributionRoot: &rewardscoordinator.GetCurrentClaimableDistributionRoot{},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetRootIndexFromHash: &rewardscoordinator.GetRootIndexFromHash{
			RootHash: rootHash,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) CalculateDomainSeparator(chainId string, contractAddr string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateDomainSeparator: &rewardscoordinator.CalculateDomainSeparator{
			ChainID:      chainId,
			ContractAddr: contractAddr,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		MerkleizeLeaves: &rewardscoordinator.MerkleizeLeaves{
			Leaves: leaves,
		},
	}

	return r.query(msg)
}

func (r *rewardsCoordinatorImpl) CheckClaim(claim rewardscoordinator.CheckClaimClaim) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CheckClaim: &rewardscoordinator.CheckClaim{
			Claim: claim,
		},
	}

	return r.query(msg)
}
