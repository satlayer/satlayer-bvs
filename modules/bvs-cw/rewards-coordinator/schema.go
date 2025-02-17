// This file was generated from JSON Schema using quicktype, do not modify it directly.
// To parse and unparse this JSON data, add this code to your project and do:
//
//    instantiateMsg, err := UnmarshalInstantiateMsg(bytes)
//    bytes, err = instantiateMsg.Marshal()
//
//    executeMsg, err := UnmarshalExecuteMsg(bytes)
//    bytes, err = executeMsg.Marshal()
//
//    queryMsg, err := UnmarshalQueryMsg(bytes)
//    bytes, err = queryMsg.Marshal()

package rewardscoordinator

import "encoding/json"

func UnmarshalInstantiateMsg(data []byte) (InstantiateMsg, error) {
	var r InstantiateMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *InstantiateMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalExecuteMsg(data []byte) (ExecuteMsg, error) {
	var r ExecuteMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ExecuteMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalQueryMsg(data []byte) (QueryMsg, error) {
	var r QueryMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *QueryMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	ActivationDelay            int64  `json:"activation_delay"`
	CalculationIntervalSeconds int64  `json:"calculation_interval_seconds"`
	DelegationManager          string `json:"delegation_manager"`
	GenesisRewardsTimestamp    int64  `json:"genesis_rewards_timestamp"`
	InitialOwner               string `json:"initial_owner"`
	InitialPausedStatus        int64  `json:"initial_paused_status"`
	MaxFutureLength            int64  `json:"max_future_length"`
	MaxRetroactiveLength       int64  `json:"max_retroactive_length"`
	MaxRewardsDuration         int64  `json:"max_rewards_duration"`
	Pauser                     string `json:"pauser"`
	RewardsUpdater             string `json:"rewards_updater"`
	StrategyManager            string `json:"strategy_manager"`
	Unpauser                   string `json:"unpauser"`
}

type ExecuteMsg struct {
	CreateBvsRewardsSubmission    *CreateBvsRewardsSubmission    `json:"create_bvs_rewards_submission,omitempty"`
	CreateRewardsForAllSubmission *CreateRewardsForAllSubmission `json:"create_rewards_for_all_submission,omitempty"`
	ProcessClaim                  *ProcessClaim                  `json:"process_claim,omitempty"`
	SubmitRoot                    *SubmitRoot                    `json:"submit_root,omitempty"`
	DisableRoot                   *DisableRoot                   `json:"disable_root,omitempty"`
	SetClaimerFor                 *SetClaimerFor                 `json:"set_claimer_for,omitempty"`
	SetActivationDelay            *SetActivationDelay            `json:"set_activation_delay,omitempty"`
	SetRewardsUpdater             *SetRewardsUpdater             `json:"set_rewards_updater,omitempty"`
	SetRewardsForAllSubmitter     *SetRewardsForAllSubmitter     `json:"set_rewards_for_all_submitter,omitempty"`
	SetGlobalOperatorCommission   *SetGlobalOperatorCommission   `json:"set_global_operator_commission,omitempty"`
	TransferOwnership             *TransferOwnership             `json:"transfer_ownership,omitempty"`
	Pause                         *Pause                         `json:"pause,omitempty"`
	Unpause                       *Unpause                       `json:"unpause,omitempty"`
	SetPauser                     *SetPauser                     `json:"set_pauser,omitempty"`
	SetUnpauser                   *SetUnpauser                   `json:"set_unpauser,omitempty"`
}

type CreateBvsRewardsSubmission struct {
	RewardsSubmissions []RewardsSubmission `json:"rewards_submissions"`
}

type RewardsSubmission struct {
	Amount                   string                  `json:"amount"`
	Duration                 int64                   `json:"duration"`
	StartTimestamp           string                  `json:"start_timestamp"`
	StrategiesAndMultipliers []StrategyAndMultiplier `json:"strategies_and_multipliers"`
	Token                    string                  `json:"token"`
}

type StrategyAndMultiplier struct {
	Multiplier int64  `json:"multiplier"`
	Strategy   string `json:"strategy"`
}

type CreateRewardsForAllSubmission struct {
	RewardsSubmissions []RewardsSubmission `json:"rewards_submissions"`
}

type DisableRoot struct {
	RootIndex int64 `json:"root_index"`
}

type Pause struct {
}

type ProcessClaim struct {
	Claim     ProcessClaimClaim `json:"claim"`
	Recipient string            `json:"recipient"`
}

type ProcessClaimClaim struct {
	EarnerIndex     int64                             `json:"earner_index"`
	EarnerLeaf      PurpleExecuteEarnerTreeMerkleLeaf `json:"earner_leaf"`
	EarnerTreeProof []int64                           `json:"earner_tree_proof"`
	RootIndex       int64                             `json:"root_index"`
	TokenIndices    []int64                           `json:"token_indices"`
	TokenLeaves     []PurpleTokenTreeMerkleLeaf       `json:"token_leaves"`
	TokenTreeProofs [][]int64                         `json:"token_tree_proofs"`
}

type PurpleExecuteEarnerTreeMerkleLeaf struct {
	Earner          string `json:"earner"`
	EarnerTokenRoot string `json:"earner_token_root"`
}

type PurpleTokenTreeMerkleLeaf struct {
	CumulativeEarnings string `json:"cumulative_earnings"`
	Token              string `json:"token"`
}

type SetActivationDelay struct {
	NewActivationDelay int64 `json:"new_activation_delay"`
}

type SetClaimerFor struct {
	Claimer string `json:"claimer"`
}

type SetGlobalOperatorCommission struct {
	NewCommissionBips int64 `json:"new_commission_bips"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetRewardsForAllSubmitter struct {
	NewValue  bool   `json:"new_value"`
	Submitter string `json:"submitter"`
}

type SetRewardsUpdater struct {
	NewUpdater string `json:"new_updater"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type SubmitRoot struct {
	RewardsCalculationEndTimestamp int64  `json:"rewards_calculation_end_timestamp"`
	Root                           string `json:"root"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
}

type QueryMsg struct {
	CalculateEarnerLeafHash             *CalculateEarnerLeafHash             `json:"calculate_earner_leaf_hash,omitempty"`
	CalculateTokenLeafHash              *CalculateTokenLeafHash              `json:"calculate_token_leaf_hash,omitempty"`
	OperatorCommissionBips              *OperatorCommissionBips              `json:"operator_commission_bips,omitempty"`
	GetDistributionRootsLength          *GetDistributionRootsLength          `json:"get_distribution_roots_length,omitempty"`
	GetCurrentDistributionRoot          *GetCurrentDistributionRoot          `json:"get_current_distribution_root,omitempty"`
	GetDistributionRootAtIndex          *GetDistributionRootAtIndex          `json:"get_distribution_root_at_index,omitempty"`
	GetCurrentClaimableDistributionRoot *GetCurrentClaimableDistributionRoot `json:"get_current_claimable_distribution_root,omitempty"`
	GetRootIndexFromHash                *GetRootIndexFromHash                `json:"get_root_index_from_hash,omitempty"`
	CalculateDomainSeparator            *CalculateDomainSeparator            `json:"calculate_domain_separator,omitempty"`
	MerkleizeLeaves                     *MerkleizeLeaves                     `json:"merkleize_leaves,omitempty"`
	CheckClaim                          *CheckClaim                          `json:"check_claim,omitempty"`
}

type CalculateDomainSeparator struct {
	ChainID      string `json:"chain_id"`
	ContractAddr string `json:"contract_addr"`
}

type CalculateEarnerLeafHash struct {
	Earner          string `json:"earner"`
	EarnerTokenRoot string `json:"earner_token_root"`
}

type CalculateTokenLeafHash struct {
	CumulativeEarnings string `json:"cumulative_earnings"`
	Token              string `json:"token"`
}

type CheckClaim struct {
	Claim CheckClaimClaim `json:"claim"`
}

type CheckClaimClaim struct {
	EarnerIndex     int64                             `json:"earner_index"`
	EarnerLeaf      FluffyExecuteEarnerTreeMerkleLeaf `json:"earner_leaf"`
	EarnerTreeProof []int64                           `json:"earner_tree_proof"`
	RootIndex       int64                             `json:"root_index"`
	TokenIndices    []int64                           `json:"token_indices"`
	TokenLeaves     []FluffyTokenTreeMerkleLeaf       `json:"token_leaves"`
	TokenTreeProofs [][]int64                         `json:"token_tree_proofs"`
}

type FluffyExecuteEarnerTreeMerkleLeaf struct {
	Earner          string `json:"earner"`
	EarnerTokenRoot string `json:"earner_token_root"`
}

type FluffyTokenTreeMerkleLeaf struct {
	CumulativeEarnings string `json:"cumulative_earnings"`
	Token              string `json:"token"`
}

type GetCurrentClaimableDistributionRoot struct {
}

type GetCurrentDistributionRoot struct {
}

type GetDistributionRootAtIndex struct {
	Index string `json:"index"`
}

type GetDistributionRootsLength struct {
}

type GetRootIndexFromHash struct {
	RootHash string `json:"root_hash"`
}

type MerkleizeLeaves struct {
	Leaves []string `json:"leaves"`
}

type OperatorCommissionBips struct {
	Bvs      string `json:"bvs"`
	Operator string `json:"operator"`
}
