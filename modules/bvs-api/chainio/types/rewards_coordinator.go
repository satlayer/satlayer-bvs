package types

type CreateBVSRewardsSubmissionReq struct {
	CreateBVSRewardsSubmission CreateRewardsSubmission `json:"create_bvs_rewards_submission"`
}

type CreateRewardsForAllSubmissionReq struct {
	CreateRewardsForAllSubmission CreateRewardsSubmission `json:"create_rewards_for_all_submission"`
}

type CreateRewardsSubmission struct {
	RewardsSubmissions []RewardsSubmission `json:"rewards_submissions"`
}

type ProcessClaimReq struct {
	ProcessClaim ProcessClaim `json:"process_claim"`
}

type ExeuteRewardsMerkleClaim struct {
	RootIndex       uint32                      `json:"root_index"`
	EarnerIndex     uint32                      `json:"earner_index"`
	EarnerTreeProof []uint16                    `json:"earner_tree_proof"`
	EarnerLeaf      ExecuteEarnerTreeMerkleLeaf `json:"earner_leaf"`
	TokenIndices    []uint32                    `json:"token_indices"`
	TokenTreeProofs [][]uint16                  `json:"token_tree_proofs"`
	TokenLeaves     []TokenTreeMerkleLeaf       `json:"token_leaves"`
}

type TokenTreeMerkleLeaf struct {
	Token              string `json:"token"`
	CumulativeEarnings string `json:"cumulative_earnings"`
}

type ExecuteEarnerTreeMerkleLeaf struct {
	Earner          string `json:"earner"`
	EarnerTokenRoot string `json:"earner_token_root"`
}

type ProcessClaim struct {
	Claim     ExeuteRewardsMerkleClaim `json:"claim"`
	Recipient string                   `json:"recipient"`
}

type SubmitRootReq struct {
	SubmitRoot SubmitRoot `json:"submit_root"`
}

type SubmitRoot struct {
	Root                           string `json:"root"`
	RewardsCalculationEndTimestamp uint64 `json:"rewards_calculation_end_timestamp"`
}

type DisableRootReq struct {
	DisableRoot DisableRoot `json:"disable_root"`
}

type DisableRoot struct {
	RootIndex uint64 `json:"root_index"`
}

type SetClaimerForReq struct {
	SetClaimerFor SetClaimerFor `json:"set_claimer_for"`
}

type SetClaimerFor struct {
	Claimer string `json:"claimer"`
}

type SetActivationDelayReq struct {
	SetActivationDelay SetActivationDelay `json:"set_activation_delay"`
}

type SetActivationDelay struct {
	NewActivationDelay uint32 `json:"new_activation_delay"`
}

type SetRewardsForAllSubmitterReq struct {
	SetRewardsForAllSubmitter SetRewardsForAllSubmitter `json:"set_rewards_for_all_submitter"`
}

type SetRewardsForAllSubmitter struct {
	Submitter string `json:"submitter"`
	NewValue  bool   `json:"new_value"`
}

type SetGlobalOperatorCommissionReq struct {
	SetGlobalOperatorCommission SetGlobalOperatorCommission `json:"SetGlobalOperatorCommission"`
}

type SetGlobalOperatorCommission struct {
	NewCommissionBips uint16 `json:"new_commission_bips"`
}

type TransferRewardsCoordinatorOwnershipReq struct {
	TransferOwnership TransferRewardsCoordinatorOwnership `json:"transfer_ownership"`
}

type TransferRewardsCoordinatorOwnership struct {
	NewOwner string `json:"new_owner"`
}

type SetRewardsUpdaterReq struct {
	SetRewardsUpdater SetRewardsUpdater `json:"set_rewards_updater"`
}

type SetRewardsUpdater struct {
	NewUpdater string `json:"new_updater"`
}

type CalculateEarnerLeafHashReq struct {
	CalculateEarnerLeafHash CalculateEarnerLeafHash `json:"calculate_earner_leaf_hash"`
}

type CalculateEarnerLeafHash struct {
	Earner          string `json:"earner"`
	EarnerTokenRoot string `json:"earner_token_root"`
}

type CalculateTokenLeafHashReq struct {
	CalculateTokenLeafHash CalculateTokenLeafHash `json:"calculate_token_leaf_hash"`
}

type CalculateTokenLeafHash struct {
	Token              string `json:"token"`
	CumulativeEarnings string `json:"cumulative_earnings"`
}

type OperatorCommissionBipsReq struct {
	OperatorCommissionBips OperatorCommissionBips `json:"operator_commission_bips"`
}

type OperatorCommissionBips struct {
	Operator string `json:"operator"`
	BVS      string `json:"bvs"`
}

type GetDistributionRootsLengthReq struct {
	GetDistributionRootsLength GetDistributionRootsLength `json:"get_distribution_roots_length"`
}

type GetDistributionRootsLength struct {
}

type GetCurrentDistributionRootReq struct {
	GetCurrentDistributionRoot GetCurrentDistributionRoot `json:"get_current_distribution_root"`
}

type GetCurrentDistributionRoot struct {
}

type GetDistributionRootAtIndexReq struct {
	GetDistributionRootAtIndex GetDistributionRootAtIndex `json:"get_distribution_root_at_index"`
}

type GetDistributionRootAtIndex struct {
	Index string `json:"index"`
}

type GetCurrentClaimableDistributionRootReq struct {
	GetCurrentClaimableDistributionRoot GetCurrentClaimableDistributionRoot `json:"get_current_claimable_distribution_root"`
}

type GetCurrentClaimableDistributionRoot struct {
}

type GetRootIndexFromHashReq struct {
	GetRootIndexFromHash GetRootIndexFromHash `json:"get_root_index_from_hash"`
}

type GetRootIndexFromHash struct {
	RootHash string `json:"root_hash"`
}

type CalculateDomainSeparatorReq struct {
	CalculateDomainSeparator CalculateDomainSeparator `json:"calculate_domain_separator"`
}

type CalculateDomainSeparator struct {
	ChainId      string `json:"chain_id"`
	ContractAddr string `json:"contract_addr"`
}

type MerkleizeLeavesReq struct {
	MerkleizeLeaves MerkleizeLeaves `json:"merkleize_leaves"`
}

type MerkleizeLeaves struct {
	Leaves []string `json:"leaves"`
}

type CheckClaimReq struct {
	CheckClaim CheckClaim `json:"check_claim"`
}

type CheckClaim struct {
	Claim ExeuteRewardsMerkleClaim `json:"claim"`
}

type StrategyAndMultiplier struct {
	Strategy   string `json:"strategy"`
	Multiplier uint64 `json:"multiplier"`
}

type RewardsSubmission struct {
	StrategiesAndMultipliers []StrategyAndMultiplier `json:"strategies_and_multipliers"`
	Token                    string                  `json:"token"`
	Amount                   string                  `json:"amount"`
	StartTimestamp           string                  `json:"start_timestamp"`
	Duration                 uint64                  `json:"duration"`
}
