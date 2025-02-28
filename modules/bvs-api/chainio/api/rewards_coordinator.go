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
	io             io.ChainIO
	ContractAddr   string
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewRewardsCoordinator(chainIO io.ChainIO) *RewardsCoordinator {
	return &RewardsCoordinator{
		io:            chainIO,
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

func (r *RewardsCoordinator) BindClient(contractAddress string) {
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

	r.ContractAddr = contractAddress
}

func (r *RewardsCoordinator) CreateRewardsSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateRewardsSubmission: &rewardscoordinator.CreateRewardsSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) CreateRewardsForAllSubmission(ctx context.Context, submissions []rewardscoordinator.RewardsSubmission) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		CreateRewardsForAllSubmission: &rewardscoordinator.CreateRewardsForAllSubmission{
			RewardsSubmissions: submissions,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) ProcessClaim(ctx context.Context, claim rewardscoordinator.ProcessClaimClaim, recipient string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		ProcessClaim: &rewardscoordinator.ProcessClaim{
			Claim:     claim,
			Recipient: recipient,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SubmitRoot(ctx context.Context, root string, rewardsCalculationEndTimestamp int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SubmitRoot: &rewardscoordinator.SubmitRoot{
			Root:                           root,
			RewardsCalculationEndTimestamp: rewardsCalculationEndTimestamp,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) DisableRoot(ctx context.Context, rootIndex int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		DisableRoot: &rewardscoordinator.DisableRoot{
			RootIndex: rootIndex,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetClaimerFor(ctx context.Context, claimer string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetClaimerFor: &rewardscoordinator.SetClaimerFor{
			Claimer: claimer,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetActivationDelay(ctx context.Context, newActivationDelay int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetActivationDelay: &rewardscoordinator.SetActivationDelay{
			NewActivationDelay: newActivationDelay,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetGlobalOperatorCommission(ctx context.Context, newCommissionBips int64) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetGlobalOperatorCommission: &rewardscoordinator.SetGlobalOperatorCommission{
			NewCommissionBips: newCommissionBips,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		TransferOwnership: &rewardscoordinator.TransferOwnership{
			NewOwner: newOwner,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetRewardsUpdater(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsUpdater: &rewardscoordinator.SetRewardsUpdater{
			Addr: addr,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetRouting(ctx context.Context, strategyManager string) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRouting: &rewardscoordinator.SetRouting{
			StrategyManager: strategyManager,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) SetRewardsForAllSubmitter(ctx context.Context, submitter string, newValue bool) (*coretypes.ResultTx, error) {
	msg := rewardscoordinator.ExecuteMsg{
		SetRewardsForAllSubmitter: &rewardscoordinator.SetRewardsForAllSubmitter{
			Submitter: submitter,
			NewValue:  newValue,
		},
	}

	return r.execute(ctx, msg)
}

func (r *RewardsCoordinator) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *RewardsCoordinator) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *RewardsCoordinator) CalculateEarnerLeafHash(earner string, earnerTokenRoot string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateEarnerLeafHash: &rewardscoordinator.CalculateEarnerLeafHash{
			Earner:          earner,
			EarnerTokenRoot: earnerTokenRoot,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) CalculateTokenLeafHash(token string, cumulativeEarnings string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CalculateTokenLeafHash: &rewardscoordinator.CalculateTokenLeafHash{
			Token:              token,
			CumulativeEarnings: cumulativeEarnings,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) OperatorCommissionBips(operator string, service string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		OperatorCommissionBips: &rewardscoordinator.OperatorCommissionBips{
			Operator: operator,
			Service:  service,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) GetDistributionRootsLength() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootsLength: &rewardscoordinator.GetDistributionRootsLength{},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) GetCurrentDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentDistributionRoot: &rewardscoordinator.GetCurrentDistributionRoot{},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) GetDistributionRootAtIndex(index string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetDistributionRootAtIndex: &rewardscoordinator.GetDistributionRootAtIndex{
			Index: index,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) GetCurrentClaimableDistributionRoot() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetCurrentClaimableDistributionRoot: &rewardscoordinator.GetCurrentClaimableDistributionRoot{},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) GetRootIndexFromHash(rootHash string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		GetRootIndexFromHash: &rewardscoordinator.GetRootIndexFromHash{
			RootHash: rootHash,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) MerkleizeLeaves(leaves []string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		MerkleizeLeaves: &rewardscoordinator.MerkleizeLeaves{
			Leaves: leaves,
		},
	}

	return r.query(msg)
}

func (r *RewardsCoordinator) CheckClaim(claim rewardscoordinator.CheckClaimClaim) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := rewardscoordinator.QueryMsg{
		CheckClaim: &rewardscoordinator.CheckClaim{
			Claim: claim,
		},
	}

	return r.query(msg)
}
