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
//    currentStakerDelegationDigestHashResponse, err := UnmarshalCurrentStakerDelegationDigestHashResponse(bytes)
//    bytes, err = currentStakerDelegationDigestHashResponse.Marshal()
//
//    calculateWithdrawalRootResponse, err := UnmarshalCalculateWithdrawalRootResponse(bytes)
//    bytes, err = calculateWithdrawalRootResponse.Marshal()
//
//    delegationApprovalDigestHashResponse, err := UnmarshalDelegationApprovalDigestHashResponse(bytes)
//    bytes, err = delegationApprovalDigestHashResponse.Marshal()
//
//    delegationApproverResponse, err := UnmarshalDelegationApproverResponse(bytes)
//    bytes, err = delegationApproverResponse.Marshal()
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
//    stakerNonceResponse, err := UnmarshalStakerNonceResponse(bytes)
//    bytes, err = stakerNonceResponse.Marshal()
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
//    stakerDelegationDigestHashResponse, err := UnmarshalStakerDelegationDigestHashResponse(bytes)
//    bytes, err = stakerDelegationDigestHashResponse.Marshal()
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

func UnmarshalCurrentStakerDelegationDigestHashResponse(data []byte) (CurrentStakerDelegationDigestHashResponse, error) {
	var r CurrentStakerDelegationDigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CurrentStakerDelegationDigestHashResponse) Marshal() ([]byte, error) {
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

func UnmarshalDelegationApprovalDigestHashResponse(data []byte) (DelegationApprovalDigestHashResponse, error) {
	var r DelegationApprovalDigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationApprovalDigestHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegationApproverResponse(data []byte) (DelegationApproverResponse, error) {
	var r DelegationApproverResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationApproverResponse) Marshal() ([]byte, error) {
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

func UnmarshalStakerNonceResponse(data []byte) (StakerNonceResponse, error) {
	var r StakerNonceResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerNonceResponse) Marshal() ([]byte, error) {
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

func UnmarshalStakerDelegationDigestHashResponse(data []byte) (StakerDelegationDigestHashResponse, error) {
	var r StakerDelegationDigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerDelegationDigestHashResponse) Marshal() ([]byte, error) {
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
	InitialOwner             string   `json:"initial_owner"`
	InitialPausedStatus      int64    `json:"initial_paused_status"`
	MinWithdrawalDelayBlocks int64    `json:"min_withdrawal_delay_blocks"`
	Pauser                   string   `json:"pauser"`
	SlashManager             string   `json:"slash_manager"`
	Strategies               []string `json:"strategies"`
	StrategyManager          string   `json:"strategy_manager"`
	Unpauser                 string   `json:"unpauser"`
	WithdrawalDelayBlocks    []int64  `json:"withdrawal_delay_blocks"`
}

type ExecuteMsg struct {
	RegisterAsOperator               *RegisterAsOperator               `json:"register_as_operator,omitempty"`
	ModifyOperatorDetails            *ModifyOperatorDetails            `json:"modify_operator_details,omitempty"`
	UpdateOperatorMetadataURI        *UpdateOperatorMetadataURI        `json:"update_operator_metadata_uri,omitempty"`
	DelegateTo                       *DelegateTo                       `json:"delegate_to,omitempty"`
	DelegateToBySignature            *DelegateToBySignature            `json:"delegate_to_by_signature,omitempty"`
	Undelegate                       *Undelegate                       `json:"undelegate,omitempty"`
	QueueWithdrawals                 *QueueWithdrawals                 `json:"queue_withdrawals,omitempty"`
	CompleteQueuedWithdrawal         *CompleteQueuedWithdrawal         `json:"complete_queued_withdrawal,omitempty"`
	CompleteQueuedWithdrawals        *CompleteQueuedWithdrawals        `json:"complete_queued_withdrawals,omitempty"`
	IncreaseDelegatedShares          *IncreaseDelegatedShares          `json:"increase_delegated_shares,omitempty"`
	DecreaseDelegatedShares          *DecreaseDelegatedShares          `json:"decrease_delegated_shares,omitempty"`
	SetMinWithdrawalDelayBlocks      *SetMinWithdrawalDelayBlocks      `json:"set_min_withdrawal_delay_blocks,omitempty"`
	SetSlashManager                  *SetSlashManager                  `json:"set_slash_manager,omitempty"`
	SetStrategyWithdrawalDelayBlocks *SetStrategyWithdrawalDelayBlocks `json:"set_strategy_withdrawal_delay_blocks,omitempty"`
	Pause                            *Pause                            `json:"pause,omitempty"`
	Unpause                          *Unpause                          `json:"unpause,omitempty"`
	SetPauser                        *SetPauser                        `json:"set_pauser,omitempty"`
	SetUnpauser                      *SetUnpauser                      `json:"set_unpauser,omitempty"`
	TwoStepTransferOwnership         *TwoStepTransferOwnership         `json:"two_step_transfer_ownership,omitempty"`
	AcceptOwnership                  *AcceptOwnership                  `json:"accept_ownership,omitempty"`
	CancelOwnershipTransfer          *CancelOwnershipTransfer          `json:"cancel_ownership_transfer,omitempty"`
}

type CompleteQueuedWithdrawal struct {
	MiddlewareTimesIndex int64             `json:"middleware_times_index"`
	ReceiveAsTokens      bool              `json:"receive_as_tokens"`
	Tokens               []string          `json:"tokens"`
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
	Tokens                 [][]string          `json:"tokens"`
	Withdrawals            []WithdrawalElement `json:"withdrawals"`
}

type DecreaseDelegatedShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type DelegateTo struct {
	ApproverSignatureAndExpiry ExecuteSignatureWithExpiry `json:"approver_signature_and_expiry"`
	Params                     ExecuteDelegateParams      `json:"params"`
}

type ExecuteSignatureWithExpiry struct {
	Expiry    int64  `json:"expiry"`
	Signature string `json:"signature"`
}

type ExecuteDelegateParams struct {
	Operator  string `json:"operator"`
	PublicKey string `json:"public_key"`
	Salt      string `json:"salt"`
	Staker    string `json:"staker"`
}

type DelegateToBySignature struct {
	ApproverSignatureAndExpiry ExecuteSignatureWithExpiry `json:"approver_signature_and_expiry"`
	Params                     ExecuteDelegateParams      `json:"params"`
	StakerPublicKey            string                     `json:"staker_public_key"`
	StakerSignatureAndExpiry   ExecuteSignatureWithExpiry `json:"staker_signature_and_expiry"`
}

type IncreaseDelegatedShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type ModifyOperatorDetails struct {
	NewOperatorDetails ExecuteOperatorDetails `json:"new_operator_details"`
}

type ExecuteOperatorDetails struct {
	DelegationApprover         string `json:"delegation_approver"`
	DeprecatedEarningsReceiver string `json:"deprecated_earnings_receiver"`
	StakerOptOutWindowBlocks   int64  `json:"staker_opt_out_window_blocks"`
}

type Pause struct {
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
	MetadataURI     string                 `json:"metadata_uri"`
	OperatorDetails ExecuteOperatorDetails `json:"operator_details"`
	SenderPublicKey string                 `json:"sender_public_key"`
}

type SetMinWithdrawalDelayBlocks struct {
	NewMinWithdrawalDelayBlocks int64 `json:"new_min_withdrawal_delay_blocks"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetSlashManager struct {
	NewSlashManager string `json:"new_slash_manager"`
}

type SetStrategyWithdrawalDelayBlocks struct {
	Strategies            []string `json:"strategies"`
	WithdrawalDelayBlocks []int64  `json:"withdrawal_delay_blocks"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TwoStepTransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type AcceptOwnership struct {
}

type CancelOwnershipTransfer struct {
}

type Undelegate struct {
	Staker string `json:"staker"`
}

type Unpause struct {
}

type UpdateOperatorMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type QueryMsg struct {
	IsDelegated                                *IsDelegated                                `json:"is_delegated,omitempty"`
	IsOperator                                 *IsOperator                                 `json:"is_operator,omitempty"`
	OperatorDetails                            *OperatorDetails                            `json:"operator_details,omitempty"`
	DelegationApprover                         *DelegationApprover                         `json:"delegation_approver,omitempty"`
	StakerOptOutWindowBlocks                   *StakerOptOutWindowBlocks                   `json:"staker_opt_out_window_blocks,omitempty"`
	GetOperatorShares                          *GetOperatorShares                          `json:"get_operator_shares,omitempty"`
	GetDelegatableShares                       *GetDelegatableShares                       `json:"get_delegatable_shares,omitempty"`
	GetWithdrawalDelay                         *GetWithdrawalDelay                         `json:"get_withdrawal_delay,omitempty"`
	CalculateWithdrawalRoot                    *CalculateWithdrawalRoot                    `json:"calculate_withdrawal_root,omitempty"`
	StakerDelegationDigestHash                 *StakerDelegationDigestHash                 `json:"staker_delegation_digest_hash,omitempty"`
	DelegationApprovalDigestHash               *DelegationApprovalDigestHash               `json:"delegation_approval_digest_hash,omitempty"`
	CalculateCurrentStakerDelegationDigestHash *CalculateCurrentStakerDelegationDigestHash `json:"calculate_current_staker_delegation_digest_hash,omitempty"`
	GetStakerNonce                             *GetStakerNonce                             `json:"get_staker_nonce,omitempty"`
	GetOperatorStakers                         *GetOperatorStakers                         `json:"get_operator_stakers,omitempty"`
	GetCumulativeWithdrawalsQueued             *GetCumulativeWithdrawalsQueued             `json:"get_cumulative_withdrawals_queued,omitempty"`
}

type CalculateCurrentStakerDelegationDigestHash struct {
	CurrentStakerDigestHashParams QueryCurrentStakerDigestHashParams `json:"current_staker_digest_hash_params"`
}

type QueryCurrentStakerDigestHashParams struct {
	ContractAddr    string `json:"contract_addr"`
	CurrentNonce    string `json:"current_nonce"`
	Expiry          int64  `json:"expiry"`
	Operator        string `json:"operator"`
	Staker          string `json:"staker"`
	StakerPublicKey string `json:"staker_public_key"`
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

type DelegationApprovalDigestHash struct {
	ApproverDigestHashParams QueryApproverDigestHashParams `json:"approver_digest_hash_params"`
}

type QueryApproverDigestHashParams struct {
	Approver          string `json:"approver"`
	ApproverPublicKey string `json:"approver_public_key"`
	ApproverSalt      string `json:"approver_salt"`
	ContractAddr      string `json:"contract_addr"`
	Expiry            int64  `json:"expiry"`
	Operator          string `json:"operator"`
	Staker            string `json:"staker"`
}

type DelegationApprover struct {
	Operator string `json:"operator"`
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

type GetStakerNonce struct {
	Staker string `json:"staker"`
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

type StakerDelegationDigestHash struct {
	StakerDigestHashParams QueryStakerDigestHashParams `json:"staker_digest_hash_params"`
}

type QueryStakerDigestHashParams struct {
	ContractAddr    string `json:"contract_addr"`
	Expiry          int64  `json:"expiry"`
	Operator        string `json:"operator"`
	Staker          string `json:"staker"`
	StakerNonce     string `json:"staker_nonce"`
	StakerPublicKey string `json:"staker_public_key"`
}

type StakerOptOutWindowBlocks struct {
	Operator string `json:"operator"`
}

type CurrentStakerDelegationDigestHashResponse struct {
	CurrentStakerDelegationDigestHash string `json:"current_staker_delegation_digest_hash"`
}

type CalculateWithdrawalRootResponse struct {
	WithdrawalRoot string `json:"withdrawal_root"`
}

type DelegationApprovalDigestHashResponse struct {
	ApproverDelegationDigestHash string `json:"approver_delegation_digest_hash"`
}

type DelegationApproverResponse struct {
	DelegationApprover string `json:"delegation_approver"`
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

type StakerNonceResponse struct {
	Nonce string `json:"nonce"`
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
	DelegationApprover         string `json:"delegation_approver"`
	DeprecatedEarningsReceiver string `json:"deprecated_earnings_receiver"`
	StakerOptOutWindowBlocks   int64  `json:"staker_opt_out_window_blocks"`
}

type StakerDelegationDigestHashResponse struct {
	StakerDelegationDigestHash string `json:"staker_delegation_digest_hash"`
}

type StakerOptOutWindowBlocksResponse struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}
