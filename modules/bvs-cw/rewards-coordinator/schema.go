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
	GenesisRewardsTimestamp    int64  `json:"genesis_rewards_timestamp"`
	MaxFutureLength            int64  `json:"max_future_length"`
	MaxRetroactiveLength       int64  `json:"max_retroactive_length"`
	MaxRewardsDuration         int64  `json:"max_rewards_duration"`
	Owner                      string `json:"owner"`
	Registry                   string `json:"registry"`
}

type ExecuteMsg struct {
	CreateRewardsSubmission     *CreateRewardsSubmission     `json:"create_rewards_submission,omitempty"`
	ProcessClaim                *ProcessClaim                `json:"process_claim,omitempty"`
	SubmitRoot                  *SubmitRoot                  `json:"submit_root,omitempty"`
	DisableRoot                 *DisableRoot                 `json:"disable_root,omitempty"`
	SetClaimerFor               *SetClaimerFor               `json:"set_claimer_for,omitempty"`
	SetActivationDelay          *SetActivationDelay          `json:"set_activation_delay,omitempty"`
	SetGlobalOperatorCommission *SetGlobalOperatorCommission `json:"set_global_operator_commission,omitempty"`
	TransferOwnership           *TransferOwnership           `json:"transfer_ownership,omitempty"`
	SetRewardsUpdater           *SetRewardsUpdater           `json:"set_rewards_updater,omitempty"`
	SetRouting                  *SetRouting                  `json:"set_routing,omitempty"`
}

type CreateRewardsSubmission struct {
	RewardsSubmissions []RewardsSubmission `json:"rewards_submissions"`
}

type RewardsSubmission struct {
	// token amount to be transferred to the contract as rewards to be distributed
	Amount string `json:"amount"`
	// duration must be multiple of calculation_interval_seconds
	Duration int64 `json:"duration"`
	// start_timestamp must be multiple of calculation_interval_seconds
	StartTimestamp           string                  `json:"start_timestamp"`
	StrategiesAndMultipliers []StrategyAndMultiplier `json:"strategies_and_multipliers"`
	// token contract address
	Token string `json:"token"`
}

type StrategyAndMultiplier struct {
	// TODO: add desc/usage
	Multiplier int64 `json:"multiplier"`
	// strategy contract address
	Strategy string `json:"strategy"`
}

type DisableRoot struct {
	RootIndex int64 `json:"root_index"`
}

type ProcessClaim struct {
	Claim     ProcessClaimClaim `json:"claim"`
	Recipient string            `json:"recipient"`
}

type ProcessClaimClaim struct {
	EarnerIndex     int64                       `json:"earner_index"`
	EarnerLeaf      PurpleEarnerTreeMerkleLeaf  `json:"earner_leaf"`
	EarnerTreeProof []int64                     `json:"earner_tree_proof"`
	RootIndex       int64                       `json:"root_index"`
	TokenIndices    []int64                     `json:"token_indices"`
	TokenLeaves     []PurpleTokenTreeMerkleLeaf `json:"token_leaves"`
	TokenTreeProofs [][]int64                   `json:"token_tree_proofs"`
}

type PurpleEarnerTreeMerkleLeaf struct {
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

type SetRewardsUpdater struct {
	Addr string `json:"addr"`
}

type SetRouting struct {
	StrategyManager string `json:"strategy_manager"`
}

type SubmitRoot struct {
	RewardsCalculationEndTimestamp int64  `json:"rewards_calculation_end_timestamp"`
	Root                           string `json:"root"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	OperatorCommissionBips              *OperatorCommissionBips              `json:"operator_commission_bips,omitempty"`
	GetDistributionRootsLength          *GetDistributionRootsLength          `json:"get_distribution_roots_length,omitempty"`
	GetCurrentDistributionRoot          *GetCurrentDistributionRoot          `json:"get_current_distribution_root,omitempty"`
	GetDistributionRootAtIndex          *GetDistributionRootAtIndex          `json:"get_distribution_root_at_index,omitempty"`
	GetCurrentClaimableDistributionRoot *GetCurrentClaimableDistributionRoot `json:"get_current_claimable_distribution_root,omitempty"`
	GetRootIndexFromHash                *GetRootIndexFromHash                `json:"get_root_index_from_hash,omitempty"`
	CheckClaim                          *CheckClaim                          `json:"check_claim,omitempty"`
}

type CheckClaim struct {
	Claim CheckClaimClaim `json:"claim"`
}

type CheckClaimClaim struct {
	EarnerIndex     int64                       `json:"earner_index"`
	EarnerLeaf      FluffyEarnerTreeMerkleLeaf  `json:"earner_leaf"`
	EarnerTreeProof []int64                     `json:"earner_tree_proof"`
	RootIndex       int64                       `json:"root_index"`
	TokenIndices    []int64                     `json:"token_indices"`
	TokenLeaves     []FluffyTokenTreeMerkleLeaf `json:"token_leaves"`
	TokenTreeProofs [][]int64                   `json:"token_tree_proofs"`
}

type FluffyEarnerTreeMerkleLeaf struct {
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

type OperatorCommissionBips struct {
	Operator string `json:"operator"`
	Service  string `json:"service"`
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

type OperatorCommissionBipsResponse struct {
	CommissionBips int64 `json:"commission_bips"`
}
