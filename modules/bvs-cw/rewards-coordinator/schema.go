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
//
//    calculateDomainSeparatorResponse, err := UnmarshalCalculateDomainSeparatorResponse(bytes)
//    bytes, err = calculateDomainSeparatorResponse.Marshal()
//
//    calculateEarnerLeafHashResponse, err := UnmarshalCalculateEarnerLeafHashResponse(bytes)
//    bytes, err = calculateEarnerLeafHashResponse.Marshal()
//
//    calculateTokenLeafHashResponse, err := UnmarshalCalculateTokenLeafHashResponse(bytes)
//    bytes, err = calculateTokenLeafHashResponse.Marshal()
//
//    checkClaimResponse, err := UnmarshalCheckClaimResponse(bytes)
//    bytes, err = checkClaimResponse.Marshal()
//
//    getCurrentClaimableDistributionRootResponse, err := UnmarshalGetCurrentClaimableDistributionRootResponse(bytes)
//    bytes, err = getCurrentClaimableDistributionRootResponse.Marshal()
//
//    getCurrentDistributionRootResponse, err := UnmarshalGetCurrentDistributionRootResponse(bytes)
//    bytes, err = getCurrentDistributionRootResponse.Marshal()
//
//    getDistributionRootAtIndexResponse, err := UnmarshalGetDistributionRootAtIndexResponse(bytes)
//    bytes, err = getDistributionRootAtIndexResponse.Marshal()
//
//    getDistributionRootsLengthResponse, err := UnmarshalGetDistributionRootsLengthResponse(bytes)
//    bytes, err = getDistributionRootsLengthResponse.Marshal()
//
//    getRootIndexFromHashResponse, err := UnmarshalGetRootIndexFromHashResponse(bytes)
//    bytes, err = getRootIndexFromHashResponse.Marshal()
//
//    merkleizeLeavesResponse, err := UnmarshalMerkleizeLeavesResponse(bytes)
//    bytes, err = merkleizeLeavesResponse.Marshal()
//
//    operatorCommissionBipsResponse, err := UnmarshalOperatorCommissionBipsResponse(bytes)
//    bytes, err = operatorCommissionBipsResponse.Marshal()

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

func UnmarshalCalculateDomainSeparatorResponse(data []byte) (CalculateDomainSeparatorResponse, error) {
	var r CalculateDomainSeparatorResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateDomainSeparatorResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalCalculateEarnerLeafHashResponse(data []byte) (CalculateEarnerLeafHashResponse, error) {
	var r CalculateEarnerLeafHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateEarnerLeafHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalCalculateTokenLeafHashResponse(data []byte) (CalculateTokenLeafHashResponse, error) {
	var r CalculateTokenLeafHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateTokenLeafHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalCheckClaimResponse(data []byte) (CheckClaimResponse, error) {
	var r CheckClaimResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CheckClaimResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalGetCurrentClaimableDistributionRootResponse(data []byte) (GetCurrentClaimableDistributionRootResponse, error) {
	var r GetCurrentClaimableDistributionRootResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *GetCurrentClaimableDistributionRootResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalGetCurrentDistributionRootResponse(data []byte) (GetCurrentDistributionRootResponse, error) {
	var r GetCurrentDistributionRootResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *GetCurrentDistributionRootResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalGetDistributionRootAtIndexResponse(data []byte) (GetDistributionRootAtIndexResponse, error) {
	var r GetDistributionRootAtIndexResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *GetDistributionRootAtIndexResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalGetDistributionRootsLengthResponse(data []byte) (GetDistributionRootsLengthResponse, error) {
	var r GetDistributionRootsLengthResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *GetDistributionRootsLengthResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalGetRootIndexFromHashResponse(data []byte) (GetRootIndexFromHashResponse, error) {
	var r GetRootIndexFromHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *GetRootIndexFromHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalMerkleizeLeavesResponse(data []byte) (MerkleizeLeavesResponse, error) {
	var r MerkleizeLeavesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *MerkleizeLeavesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorCommissionBipsResponse(data []byte) (OperatorCommissionBipsResponse, error) {
	var r OperatorCommissionBipsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorCommissionBipsResponse) Marshal() ([]byte, error) {
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
	PauseAll                      *PauseAll                      `json:"pause_all,omitempty"`
	UnpauseAll                    *UnpauseAll                    `json:"unpause_all,omitempty"`
	PauseBit                      *PauseBit                      `json:"pause_bit,omitempty"`
	UnpauseBit                    *UnpauseBit                    `json:"unpause_bit,omitempty"`
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

type PauseAll struct {
}

type UnpauseAll struct {
}

type PauseBit struct {
	Index uint8 `json:"index"`
}

type UnpauseBit struct {
	Index uint8 `json:"index"`
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

type CalculateDomainSeparatorResponse struct {
	DomainSeparatorBinary string `json:"domain_separator_binary"`
}

type CalculateEarnerLeafHashResponse struct {
	HashBinary string `json:"hash_binary"`
}

type CalculateTokenLeafHashResponse struct {
	HashBinary string `json:"hash_binary"`
}

type CheckClaimResponse struct {
	CheckClaim bool `json:"check_claim"`
}

type GetCurrentClaimableDistributionRootResponse struct {
	Root GetCurrentClaimableDistributionRootResponseRoot `json:"root"`
}

type GetCurrentClaimableDistributionRootResponseRoot struct {
	ActivatedAt                    int64  `json:"activated_at"`
	Disabled                       bool   `json:"disabled"`
	RewardsCalculationEndTimestamp int64  `json:"rewards_calculation_end_timestamp"`
	Root                           string `json:"root"`
}

type GetCurrentDistributionRootResponse struct {
	Root GetCurrentDistributionRootResponseRoot `json:"root"`
}

type GetCurrentDistributionRootResponseRoot struct {
	ActivatedAt                    int64  `json:"activated_at"`
	Disabled                       bool   `json:"disabled"`
	RewardsCalculationEndTimestamp int64  `json:"rewards_calculation_end_timestamp"`
	Root                           string `json:"root"`
}

type GetDistributionRootAtIndexResponse struct {
	Root GetDistributionRootAtIndexResponseRoot `json:"root"`
}

type GetDistributionRootAtIndexResponseRoot struct {
	ActivatedAt                    int64  `json:"activated_at"`
	Disabled                       bool   `json:"disabled"`
	RewardsCalculationEndTimestamp int64  `json:"rewards_calculation_end_timestamp"`
	Root                           string `json:"root"`
}

type GetDistributionRootsLengthResponse struct {
	RootsLength int64 `json:"roots_length"`
}

type GetRootIndexFromHashResponse struct {
	RootIndex int64 `json:"root_index"`
}

type MerkleizeLeavesResponse struct {
	RootHashBinary string `json:"root_hash_binary"`
}

type OperatorCommissionBipsResponse struct {
	CommissionBips int64 `json:"commission_bips"`
}
