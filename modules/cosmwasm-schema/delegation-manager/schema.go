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
//    calculateWithdrawalRootResponse, err := UnmarshalCalculateWithdrawalRootResponse(bytes)
//    bytes, err = calculateWithdrawalRootResponse.Marshal()
//
//    cumulativeWithdrawalsQueuedResponse, err := UnmarshalCumulativeWithdrawalsQueuedResponse(bytes)
//    bytes, err = cumulativeWithdrawalsQueuedResponse.Marshal()
//
//    delegatableSharesResponse, err := UnmarshalDelegatableSharesResponse(bytes)
//    bytes, err = delegatableSharesResponse.Marshal()
//
//    operatorSharesResponse, err := UnmarshalOperatorSharesResponse(bytes)
//    bytes, err = operatorSharesResponse.Marshal()
//
//    operatorStakersResponse, err := UnmarshalOperatorStakersResponse(bytes)
//    bytes, err = operatorStakersResponse.Marshal()
//
//    withdrawalDelayResponse, err := UnmarshalWithdrawalDelayResponse(bytes)
//    bytes, err = withdrawalDelayResponse.Marshal()
//
//    delegatedResponse, err := UnmarshalDelegatedResponse(bytes)
//    bytes, err = delegatedResponse.Marshal()
//
//    operatorResponse, err := UnmarshalOperatorResponse(bytes)
//    bytes, err = operatorResponse.Marshal()
//
//    operatorDetailsResponse, err := UnmarshalOperatorDetailsResponse(bytes)
//    bytes, err = operatorDetailsResponse.Marshal()
//
//    stakerOptOutWindowBlocksResponse, err := UnmarshalStakerOptOutWindowBlocksResponse(bytes)
//    bytes, err = stakerOptOutWindowBlocksResponse.Marshal()

package delegationmanager

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

func UnmarshalCalculateWithdrawalRootResponse(data []byte) (CalculateWithdrawalRootResponse, error) {
	var r CalculateWithdrawalRootResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateWithdrawalRootResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalCumulativeWithdrawalsQueuedResponse(data []byte) (CumulativeWithdrawalsQueuedResponse, error) {
	var r CumulativeWithdrawalsQueuedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CumulativeWithdrawalsQueuedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegatableSharesResponse(data []byte) (DelegatableSharesResponse, error) {
	var r DelegatableSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegatableSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorSharesResponse(data []byte) (OperatorSharesResponse, error) {
	var r OperatorSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorStakersResponse(data []byte) (OperatorStakersResponse, error) {
	var r OperatorStakersResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorStakersResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalWithdrawalDelayResponse(data []byte) (WithdrawalDelayResponse, error) {
	var r WithdrawalDelayResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *WithdrawalDelayResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegatedResponse(data []byte) (DelegatedResponse, error) {
	var r DelegatedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegatedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorResponse(data []byte) (OperatorResponse, error) {
	var r OperatorResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorDetailsResponse(data []byte) (OperatorDetailsResponse, error) {
	var r OperatorDetailsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorDetailsResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStakerOptOutWindowBlocksResponse(data []byte) (StakerOptOutWindowBlocksResponse, error) {
	var r StakerOptOutWindowBlocksResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerOptOutWindowBlocksResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	MinWithdrawalDelayBlocks int64    `json:"min_withdrawal_delay_blocks"`
	Owner                    string   `json:"owner"`
	Pauser                   string   `json:"pauser"`
	Strategies               []string `json:"strategies"`
	WithdrawalDelayBlocks    []int64  `json:"withdrawal_delay_blocks"`
}

type ExecuteMsg struct {
	RegisterAsOperator               *RegisterAsOperator               `json:"register_as_operator,omitempty"`
	ModifyOperatorDetails            *ModifyOperatorDetails            `json:"modify_operator_details,omitempty"`
	UpdateOperatorMetadataURI        *UpdateOperatorMetadataURI        `json:"update_operator_metadata_uri,omitempty"`
	DelegateTo                       *DelegateTo                       `json:"delegate_to,omitempty"`
	Undelegate                       *Undelegate                       `json:"undelegate,omitempty"`
	QueueWithdrawals                 *QueueWithdrawals                 `json:"queue_withdrawals,omitempty"`
	CompleteQueuedWithdrawal         *CompleteQueuedWithdrawal         `json:"complete_queued_withdrawal,omitempty"`
	CompleteQueuedWithdrawals        *CompleteQueuedWithdrawals        `json:"complete_queued_withdrawals,omitempty"`
	IncreaseDelegatedShares          *IncreaseDelegatedShares          `json:"increase_delegated_shares,omitempty"`
	DecreaseDelegatedShares          *DecreaseDelegatedShares          `json:"decrease_delegated_shares,omitempty"`
	SetMinWithdrawalDelayBlocks      *SetMinWithdrawalDelayBlocks      `json:"set_min_withdrawal_delay_blocks,omitempty"`
	SetStrategyWithdrawalDelayBlocks *SetStrategyWithdrawalDelayBlocks `json:"set_strategy_withdrawal_delay_blocks,omitempty"`
	TransferOwnership                *TransferOwnership                `json:"transfer_ownership,omitempty"`
	SetRouting                       *SetRouting                       `json:"set_routing,omitempty"`
}

type CompleteQueuedWithdrawal struct {
	MiddlewareTimesIndex int64             `json:"middleware_times_index"`
	ReceiveAsTokens      bool              `json:"receive_as_tokens"`
	Withdrawal           WithdrawalElement `json:"withdrawal"`
}

type WithdrawalElement struct {
	DelegatedTo string   `json:"delegated_to"`
	Nonce       string   `json:"nonce"`
	Shares      []string `json:"shares"`
	Staker      string   `json:"staker"`
	StartBlock  int64    `json:"start_block"`
	Strategies  []string `json:"strategies"`
	Withdrawer  string   `json:"withdrawer"`
}

type CompleteQueuedWithdrawals struct {
	MiddlewareTimesIndexes []int64             `json:"middleware_times_indexes"`
	ReceiveAsTokens        []bool              `json:"receive_as_tokens"`
	Withdrawals            []WithdrawalElement `json:"withdrawals"`
}

type DecreaseDelegatedShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type DelegateTo struct {
	Operator string `json:"operator"`
}

type IncreaseDelegatedShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type ModifyOperatorDetails struct {
	NewOperatorDetails NewOperatorDetailsClass `json:"new_operator_details"`
}

type NewOperatorDetailsClass struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}

type QueueWithdrawals struct {
	QueuedWithdrawalParams []QueuedWithdrawalParams `json:"queued_withdrawal_params"`
}

type QueuedWithdrawalParams struct {
	Shares     []string `json:"shares"`
	Strategies []string `json:"strategies"`
	Withdrawer string   `json:"withdrawer"`
}

type RegisterAsOperator struct {
	MetadataURI     string                  `json:"metadata_uri"`
	OperatorDetails NewOperatorDetailsClass `json:"operator_details"`
}

type SetMinWithdrawalDelayBlocks struct {
	NewMinWithdrawalDelayBlocks int64 `json:"new_min_withdrawal_delay_blocks"`
}

type SetRouting struct {
	SlashManager    string `json:"slash_manager"`
	StrategyManager string `json:"strategy_manager"`
}

type SetStrategyWithdrawalDelayBlocks struct {
	Strategies            []string `json:"strategies"`
	WithdrawalDelayBlocks []int64  `json:"withdrawal_delay_blocks"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type Undelegate struct {
	Staker string `json:"staker"`
}

type UpdateOperatorMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type QueryMsg struct {
	IsDelegated                    *IsDelegated                    `json:"is_delegated,omitempty"`
	IsOperator                     *IsOperator                     `json:"is_operator,omitempty"`
	OperatorDetails                *OperatorDetails                `json:"operator_details,omitempty"`
	StakerOptOutWindowBlocks       *StakerOptOutWindowBlocks       `json:"staker_opt_out_window_blocks,omitempty"`
	GetOperatorShares              *GetOperatorShares              `json:"get_operator_shares,omitempty"`
	GetDelegatableShares           *GetDelegatableShares           `json:"get_delegatable_shares,omitempty"`
	GetWithdrawalDelay             *GetWithdrawalDelay             `json:"get_withdrawal_delay,omitempty"`
	CalculateWithdrawalRoot        *CalculateWithdrawalRoot        `json:"calculate_withdrawal_root,omitempty"`
	GetOperatorStakers             *GetOperatorStakers             `json:"get_operator_stakers,omitempty"`
	GetCumulativeWithdrawalsQueued *GetCumulativeWithdrawalsQueued `json:"get_cumulative_withdrawals_queued,omitempty"`
}

type CalculateWithdrawalRoot struct {
	Withdrawal CalculateWithdrawalRootWithdrawal `json:"withdrawal"`
}

type CalculateWithdrawalRootWithdrawal struct {
	DelegatedTo string   `json:"delegated_to"`
	Nonce       string   `json:"nonce"`
	Shares      []string `json:"shares"`
	Staker      string   `json:"staker"`
	StartBlock  int64    `json:"start_block"`
	Strategies  []string `json:"strategies"`
	Withdrawer  string   `json:"withdrawer"`
}

type GetCumulativeWithdrawalsQueued struct {
	Staker string `json:"staker"`
}

type GetDelegatableShares struct {
	Staker string `json:"staker"`
}

type GetOperatorShares struct {
	Operator   string   `json:"operator"`
	Strategies []string `json:"strategies"`
}

type GetOperatorStakers struct {
	Operator string `json:"operator"`
}

type GetWithdrawalDelay struct {
	Strategies []string `json:"strategies"`
}

type IsDelegated struct {
	Staker string `json:"staker"`
}

type IsOperator struct {
	Operator string `json:"operator"`
}

type OperatorDetails struct {
	Operator string `json:"operator"`
}

type StakerOptOutWindowBlocks struct {
	Operator string `json:"operator"`
}

type CalculateWithdrawalRootResponse struct {
	WithdrawalRoot string `json:"withdrawal_root"`
}

type CumulativeWithdrawalsQueuedResponse struct {
	CumulativeWithdrawals string `json:"cumulative_withdrawals"`
}

type DelegatableSharesResponse struct {
	Shares     []string `json:"shares"`
	Strategies []string `json:"strategies"`
}

type OperatorSharesResponse struct {
	Shares []string `json:"shares"`
}

type OperatorStakersResponse struct {
	StakersAndShares []StakerShares `json:"stakers_and_shares"`
}

type StakerShares struct {
	SharesPerStrategy [][]string `json:"shares_per_strategy"`
	Staker            string     `json:"staker"`
}

type WithdrawalDelayResponse struct {
	WithdrawalDelays []int64 `json:"withdrawal_delays"`
}

type DelegatedResponse struct {
	IsDelegated bool `json:"is_delegated"`
}

type OperatorResponse struct {
	IsOperator bool `json:"is_operator"`
}

type OperatorDetailsResponse struct {
	Details DetailsClass `json:"details"`
}

type DetailsClass struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}

type StakerOptOutWindowBlocksResponse struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}
