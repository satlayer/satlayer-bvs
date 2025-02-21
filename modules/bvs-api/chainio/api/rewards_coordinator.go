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

func (a *rewardsCoordinatorImpl) CreateBVSRewardsSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateBvsRewardsSubmission: &rewardscoordinator.CreateBvsRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) CreateRewardsForAllSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateRewardsForAllSubmission: &rewardscoordinator.CreateRewardsForAllSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) ProcessClaim(ctx context.Context, claim rewardscoordinator.ProcessClaimClaim, recipient string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		ProcessClaim: &rewardscoordinator.ProcessClaim{
			Claim:     claim,
			Recipient: recipient,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SubmitRoot: &rewardscoordinator.SubmitRoot{
			Root:                           root,
			RewardsCalculationEndTimestamp: rewardsCalculationEndTimestamp,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) DisableRoot(ctx context.Context, rootIndex int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		DisableRoot: &rewardscoordinator.DisableRoot{
			RootIndex: rootIndex,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetClaimerFor: &rewardscoordinator.SetClaimerFor{
			Claimer: claimer,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetActivationDelay(ctx context.Context, newActivationDelay int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetActivationDelay: &rewardscoordinator.SetActivationDelay{
			NewActivationDelay: newActivationDelay,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetGlobalOperatorCommission(ctx context.Context, newCommissionBips int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetGlobalOperatorCommission: &rewardscoordinator.SetGlobalOperatorCommission{
			NewCommissionBips: newCommissionBips,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Pause: &rewardscoordinator.Pause{},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		Unpause: &rewardscoordinator.Unpause{},
	}
	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetPauser: &rewardscoordinator.SetPauser{
			NewPauser: newPauser,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetUnpauser: &rewardscoordinator.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		TransferOwnership: &rewardscoordinator.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetRewardsUpdater(ctx context.Context, newUpdater string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsUpdater: &rewardscoordinator.SetRewardsUpdater{
			NewUpdater: newUpdater,
		},
	}

	return a.execute(ctx, msg)
}

func (a *rewardsCoordinatorImpl) SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsForAllSubmitter: &rewardscoordinator.SetRewardsForAllSubmitter{
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
	msg := rewardscoordinator.QueryMsg{
		CalculateEarnerLeafHash: &rewardscoordinator.CalculateEarnerLeafHash{
			Earner:          earner,
			EarnerTokenRoot: earnerTokenRoot,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateTokenLeafHash: &rewardscoordinator.CalculateTokenLeafHash{
			Token:              token,
			CumulativeEarnings: cumulativeEarnings,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) OperatorCommissionBips(operator string, bvs string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		OperatorCommissionBips: &rewardscoordinator.OperatorCommissionBips{
			Operator: operator,
			Bvs:      bvs,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootsLength: &rewardscoordinator.GetDistributionRootsLength{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentDistributionRoot: &rewardscoordinator.GetCurrentDistributionRoot{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootAtIndex: &rewardscoordinator.GetDistributionRootAtIndex{
			Index: index,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentClaimableDistributionRoot: &rewardscoordinator.GetCurrentClaimableDistributionRoot{},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetRootIndexFromHash: &rewardscoordinator.GetRootIndexFromHash{
			RootHash: rootHash,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CalculateDomainSeparator(chainId string, contractAddr string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateDomainSeparator: &rewardscoordinator.CalculateDomainSeparator{
			ChainID:      chainId,
			ContractAddr: contractAddr,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		MerkleizeLeaves: &rewardscoordinator.MerkleizeLeaves{
			Leaves: leaves,
		},
	}

	return a.query(msg)
}

func (a *rewardsCoordinatorImpl) CheckClaim(claim rewardscoordinator.CheckClaimClaim) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CheckClaim: &rewardscoordinator.CheckClaim{
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
