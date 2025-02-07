package types

type RegisterAsOperatorReq struct {
	RegisterAsOperator RegisterAsOperator `json:"register_as_operator"`
}

type RegisterAsOperator struct {
	SenderPublicKey string          `json:"sender_public_key"` // base64
	OperatorDetails OperatorDetails `json:"operator_details"`
	MetadataURI     string          `json:"metadata_uri"`
}

type OperatorDetails struct {
	DeprecatedEarningsReceiver string `json:"deprecated_earnings_receiver"`
	DelegationApprover         string `json:"delegation_approver"`
	StakerOptOutWindowBlocks   uint64 `json:"staker_opt_out_window_blocks"`
}

type ModifyOperatorDetailsReq struct {
	ModifyOperatorDetails ModifyOperatorDetails `json:"modify_operator_details"`
}

type ModifyOperatorDetails struct {
	NewOperatorDetails OperatorDetails `json:"new_operator_details"`
}

type UpdateOperatorMetadataURIReq struct {
	UpdateOperatorMetadataURI UpdateOperatorMetadataURI `json:"update_operator_metadata_uri"`
}

type UpdateOperatorMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type DelegateToReq struct {
	DelegateTo DelegateTo `json:"delegate_to"`
}

type DelegateTo struct {
	Params                     DelegateParams      `json:"params"`
	ApproverSignatureAndExpiry SignatureWithExpiry `json:"approver_signature_and_expiry"`
}

type SignatureWithExpiry struct {
	Signature string `json:"signature"`
	Expiry    uint64 `json:"expiry"`
}

type DelegateToBySignatureReq struct {
	DelegateToBySignature DelegateToBySignature `json:"delegate_to_by_signature"`
}

type DelegateToBySignature struct {
	Params                     DelegateParams      `json:"params"`
	StakerPublicKey            string              `json:"staker_public_key"` // base64
	StakerSignatureAndExpiry   SignatureWithExpiry `json:"staker_signature_and_expiry"`
	ApproverSignatureAndExpiry SignatureWithExpiry `json:"approver_signature_and_expiry"`
}

type DelegateParams struct {
	Staker    string `json:"staker"`
	Operator  string `json:"operator"`
	PublicKey string `json:"public_key"` // base64
	Salt      string `json:"salt"`       // base64
}

type UnDelegateReq struct {
	UnDelegate UnDelegate `json:"undelegate"`
}

type UnDelegate struct {
	Staker string `json:"staker"`
}

type QueueWithdrawalsReq struct {
	QueueWithdrawals QueueWithdrawals `json:"queue_withdrawals"`
}

type QueueWithdrawals struct {
	QueuedWithdrawalParams []QueuedWithdrawalParams `json:"queued_withdrawal_params"`
}

type QueuedWithdrawalParams struct {
	WithDrawer string   `json:"withdrawer"` // The staker address of the calling function
	Strategies []string `json:"strategies"`
	Shares     []string `json:"shares"`
}

type CompleteQueuedWithdrawalReq struct {
	CompleteQueuedWithdrawal CompleteQueuedWithdrawal `json:"complete_queued_withdrawal"`
}

type CompleteQueuedWithdrawal struct {
	Withdrawal           Withdrawal `json:"withdrawal"`
	Tokens               []string   `json:"tokens"`
	MiddlewareTimesIndex uint64     `json:"middleware_times_index"`
	ReceiveAsTokens      bool       `json:"receive_as_tokens"`
}

type Withdrawal struct {
	Staker      string   `json:"staker"`
	DelegatedTo string   `json:"delegated_to"`
	Withdrawer  string   `json:"withdrawer"`
	Nonce       string   `json:"nonce"`
	StartBlock  uint64   `json:"start_block"` // When executing UnDelegate or QueueWithdrawals, the block height of the transaction
	Strategies  []string `json:"strategies"`
	Shares      []string `json:"shares"`
}

type CompleteQueuedWithdrawalsReq struct {
	CompleteQueuedWithdrawals CompleteQueuedWithdrawals `json:"complete_queued_withdrawals"`
}

type CompleteQueuedWithdrawals struct {
	Withdrawals            []Withdrawal `json:"withdrawals"`
	Tokens                 [][]string   `json:"tokens"`
	MiddlewareTimesIndexes []uint64     `json:"middleware_times_indexes"`
	ReceiveAsTokens        []bool       `json:"receive_as_tokens"`
}

type IncreaseDelegatedSharesReq struct {
	IncreaseDelegatedShares DelegatedShares `json:"increase_delegated_shares"`
}

type DelegatedShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
	Shares   string `json:"shares"`
}

type DecreaseDelegatedSharesReq struct {
	DecreaseDelegatedShares DelegatedShares `json:"decrease_delegated_shares"`
}

type SetMinWithdrawalDelayBlocksReq struct {
	SetMinWithdrawalDelayBlocks SetMinWithdrawalDelayBlocks `json:"set_min_withdrawal_delay_blocks"`
}

type SetMinWithdrawalDelayBlocks struct {
	NewMinWithdrawalDelayBlocks uint64 `json:"new_min_withdrawal_delay_blocks"`
}

type SetStrategyWithdrawalDelayBlocksReq struct {
	SetStrategyWithdrawalDelayBlocks SetStrategyWithdrawalDelayBlocks `json:"set_strategy_withdrawal_delay_blocks"`
}

type SetStrategyWithdrawalDelayBlocks struct {
	Strategies            []string `json:"strategies"`
	WithdrawalDelayBlocks []uint64 `json:"withdrawal_delay_blocks"`
}

type DelegateTransferOwnershipReq struct {
	DelegateTransferOwnership DelegateTransferOwnership `json:"transfer_ownership"`
}

type DelegateTransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type DelegationPauseReq struct {
	Pause struct{} `json:"pause"`
}

type DelegationUnpauseReq struct {
	Unpause struct{} `json:"unpause"`
}

type DelegationSetPauserReq struct {
	SetPauser DelegationSetPauser `json:"set_pauser"`
}
type DelegationSetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type DelegationSetUnpauserReq struct {
	SetUnpauser DelegationSetUnpauser `json:"set_unpauser"`
}

type DelegationSetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type DelegationSetSlashManagerReq struct {
	SetSlashManager DelegationSetSlashManager `json:"set_slash_manager"`
}

type DelegationSetSlashManager struct {
	NewSlashManager string `json:"new_slash_manager"`
}

type IsDelegatedReq struct {
	IsDelegated IsDelegated `json:"is_delegated"`
}

type IsDelegated struct {
	Staker string `json:"staker"`
}

type IsDelegatedResp struct {
	IsDelegated bool `json:"is_delegated"`
}

type IsOperatorReq struct {
	IsOperator IsOperator `json:"is_operator"`
}

type IsOperator struct {
	Operator string `json:"operator"`
}

type IsOperatorResp struct {
	IsOperator bool `json:"is_operator"`
}

type OperatorDetailsReq struct {
	QueryOperatorDetails QueryOperatorDetails `json:"operator_details"`
}

type QueryOperatorDetails struct {
	Operator string `json:"operator"`
}

type OperatorDetailsResp struct {
	Details OperatorDetails `json:"details"`
}

type DelegationApproverReq struct {
	DelegationApprover DelegationApprover `json:"delegation_approver"`
}

type DelegationApprover struct {
	Operator string `json:"operator"`
}

type DelegationApproverResp struct {
	DelegationApprover string `json:"delegation_approver"`
}

type StakerOptOutWindowBlocksReq struct {
	StakerOptOutWindowBlocks StakerOptOutWindowBlocks `json:"staker_opt_out_window_blocks"`
}

type StakerOptOutWindowBlocks struct {
	Operator string `json:"operator"`
}

type StakerOptOutWindowBlocksResp struct {
	StakerOptOutWindowBlocks uint64 `json:"staker_opt_out_window_blocks"`
}

type GetOperatorSharesReq struct {
	GetOperatorShares GetOperatorShares `json:"get_operator_shares"`
}

type GetOperatorShares struct {
	Operator   string   `json:"operator"`
	Strategies []string `json:"strategies"`
}

type GetOperatorSharesResp struct {
	Shares []string `json:"shares"`
}

type GetOperatorStakersReq struct {
	GetOperatorStakers GetOperatorStakers `json:"get_operator_stakers"`
}

type GetOperatorStakers struct {
	Operator string `json:"operator"`
}

type GetOperatorStakersResp struct {
	StakersAndShares []StakerShares `json:"stakers_and_shares"`
}

type StakerShares struct {
	Staker            string     `json:"staker"`
	SharesPerStrategy [][]string `json:"shares_per_strategy"`
}

type GetDelegatableSharesReq struct {
	GetDelegatableShares GetDelegatableShares `json:"get_delegatable_shares"`
}

type GetDelegatableShares struct {
	Staker string `json:"staker"`
}

type GetDelegatableSharesResp struct {
	Strategies []string `json:"strategies"`
	Shares     []string `json:"shares"`
}

type GetWithdrawalDelayReq struct {
	GetWithdrawalDelay GetWithdrawalDelay `json:"get_withdrawal_delay"`
}

type GetWithdrawalDelay struct {
	Strategies []string `json:"strategies"`
}

type GetWithdrawalDelayResp struct {
	WithdrawalDelays []uint64 `json:"withdrawal_delays"`
}

type CalculateWithdrawalRootReq struct {
	CalculateWithdrawalRoot CalculateWithdrawalRoot `json:"calculate_withdrawal_root"`
}

type CalculateWithdrawalRoot struct {
	Withdrawal Withdrawal `json:"withdrawal"`
}

type CalculateCurrentStakerDelegationDigestHashReq struct {
	CalculateCurrentStakerDelegationDigestHash CalculateCurrentStakerDelegationDigestHash `json:"calculate_current_staker_delegation_digest_hash"`
}

type CalculateCurrentStakerDelegationDigestHash struct {
	CurrentStakerDigestHashParams CurrentStakerDigestHashParams `json:"current_staker_digest_hash_params"`
}

type CurrentStakerDigestHashParams struct {
	Staker          string `json:"staker"`
	Operator        string `json:"operator"`
	StakerPublicKey string `json:"staker_public_key"` // base64
	Expiry          uint64 `json:"expiry"`
	CurrentNonce    string `json:"current_nonce"`
	ContractAddr    string `json:"contract_addr"`
}

type StakerDelegationDigestHashReq struct {
	StakerDelegationDigestHash StakerDelegationDigestHash `json:"staker_delegation_digest_hash"`
}

type StakerDelegationDigestHash struct {
	StakerDigestHashParams StakerDigestHashParams `json:"staker_digest_hash_params"`
}

type StakerDigestHashParams struct {
	Staker          string `json:"staker"`
	StakerNonce     string `json:"staker_nonce"`
	Operator        string `json:"operator"`
	StakerPublicKey string `json:"staker_public_key"` // base64
	Expiry          uint64 `json:"expiry"`
	ContractAddr    string `json:"contract_addr"`
}

type DelegationApprovalDigestHashReq struct {
	DelegationApprovalDigestHash DelegationApprovalDigestHash `json:"delegation_approval_digest_hash"`
}

type DelegationApprovalDigestHash struct {
	ApproverDigestHashParams ApproverDigestHashParams `json:"approver_digest_hash_params"`
}

type ApproverDigestHashParams struct {
	Staker            string `json:"staker"`
	Operator          string `json:"operator"`
	Approver          string `json:"approver"`
	ApproverPublicKey string `json:"approver_public_key"` // base64
	ApproverSalt      string `json:"approver_salt"`       // base64
	Expiry            uint64 `json:"expiry"`
	ContractAddr      string `json:"contract_addr"`
}

type GetStakerNonceReq struct {
	GetStakerNonce GetStakerNonce `json:"get_staker_nonce"`
}

type GetStakerNonce struct {
	Staker string `json:"staker"`
}

type GetStakerNonceResp struct {
	Nonce string `json:"nonce"`
}

type GetCumulativeWithdrawalsQueuedNonceReq struct {
	GetCumulativeWithdrawalsQueuedNonce GetCumulativeWithdrawalsQueuedNonce `json:"get_cumulative_withdrawals_queued"`
}

type GetCumulativeWithdrawalsQueuedNonce struct {
	Staker string `json:"staker"`
}

type GetCumulativeWithdrawalsQueuedNonceResp struct {
	CumulativeWithdrawals string `json:"cumulative_withdrawals"`
}
