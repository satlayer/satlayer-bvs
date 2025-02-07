package types

type SubmitSlashRequestReq struct {
	SubmitSlashRequest SubmitSlashRequest `json:"submit_slash_request"`
}

type SubmitSlashRequest struct {
	SlashDetails         ExecuteSlashDetails `json:"slash_details"`
	ValidatorsPublicKeys []string            `json:"validators_public_keys"`
}

type ExecuteSlashDetails struct {
	Slasher        string   `json:"slasher"`
	Operator       string   `json:"operator"`
	Share          string   `json:"share"`
	SlashSignature uint64   `json:"slash_signature"`
	SlashValidator []string `json:"slash_validator"`
	Reason         string   `json:"reason"`
	StartTime      uint64   `json:"start_time"`
	EndTime        uint64   `json:"end_time"`
	Status         bool     `json:"status"`
}

type ExecuteSlashRequestReq struct {
	ExecuteSlashRequest ExecuteSlashRequest `json:"execute_slash_request"`
}

type ExecuteSlashRequest struct {
	SlashHash            string   `json:"slash_hash"`
	Signatures           []string `json:"signatures"`
	ValidatorsPublicKeys []string `json:"validators_public_keys"`
}

type CancelSlashRequestReq struct {
	CancelSlashRequest CancelSlashRequest `json:"cancel_slash_request"`
}

type CancelSlashRequest struct {
	SlashHash string `json:"slash_hash"`
}

type SetMinimalSlashSignatureReq struct {
	SetMinimalSlashSignature SetMinimalSlashSignature `json:"set_minimal_slash_signature"`
}

type SetMinimalSlashSignature struct {
	MinimalSignature uint64 `json:"minimal_signature"`
}

type TransferSlashManagerOwnershipReq struct {
	TransferOwnership TransferSlashManagerOwnership `json:"transfer_ownership"`
}

type TransferSlashManagerOwnership struct {
	NewOwner string `json:"new_owner"`
}

type SetSlasherReq struct {
	SetSlasher SetSlasher `json:"set_slasher"`
}

type SetSlasher struct {
	Slasher string `json:"slasher"`
	Value   bool   `json:"value"`
}

type SetSlasherValidatorReq struct {
	SetSlasherValidator SetSlasherValidator `json:"set_slasher_validator"`
}

type SetSlasherValidator struct {
	Validators []string `json:"validators"`
	Values     []bool   `json:"values"`
}

type SetDelegationManagerSlashManagerReq struct {
	SetDelegationManager SetDelegationManagerSlashManager `json:"set_delegation_manager"`
}

type SetDelegationManagerSlashManager struct {
	NewDelegationManager string `json:"new_delegation_manager"`
}

type SlashPauseReq struct {
	Pause struct{} `json:"pause"`
}

type SlashUnPauseReq struct {
	UnPause struct{} `json:"unpause"`
}

type SetSlashPauserReq struct {
	SetPauser SetSlashPauser `json:"set_pauser"`
}

type SetSlashPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetSlashUnpauserReq struct {
	SetUnpauser SetSlashUnpauser `json:"set_unpauser"`
}

type SetSlashUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type SlashSetStrategyManagerReq struct {
	SetStrategyManager SlashSetStrategyManager `json:"set_strategy_manager"`
}

type SlashSetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}

type GetSlashDetailsReq struct {
	GetSlashDetails GetSlashDetails `json:"get_slash_details"`
}

type GetSlashDetails struct {
	SlashHash string `json:"slash_hash"`
}

type SlashDetailsResponse struct {
	SlashDetails ExecuteSlashDetails `json:"slash_details"`
}

type IsValidatorReq struct {
	IsValidator IsValidator `json:"is_validator"`
}

type IsValidator struct {
	Validator string `json:"validator"`
}

type GetMinimalSlashSignatureReq struct {
	GetMinimalSlashSignature GetMinimalSlashSignature `json:"get_minimal_slash_signature"`
}

type GetMinimalSlashSignature struct {
}

type CalculateSlashHashReq struct {
	CalculateSlashHash CalculateSlashHash `json:"calculate_slash_hash"`
}

type CalculateSlashHash struct {
	Sender               string              `json:"sender"`
	SlashDetails         ExecuteSlashDetails `json:"slash_details"`
	ValidatorsPublicKeys []string            `json:"validators_public_keys"`
}

type CalculateSlashHashResp struct {
	MessageBytes []byte `json:"message_bytes"`
}

type GetSlashDetailsResp struct {
	SlashDetails ExecuteSlashDetails `json:"slash_details"`
}

type IsValidatorResp struct {
	IsValidator bool `json:"is_validator"`
}

type GetMinimalSlashSignatureResp struct {
	MinimalSlashSignature uint64 `json:"minimal_slash_signature"`
}
