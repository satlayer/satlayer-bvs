// This file was automatically generated from slash-manager/schema.json.
// DO NOT MODIFY IT BY HAND.

package slashmanager

type InstantiateMsg struct {
	Owner  string `json:"owner"`
	Pauser string `json:"pauser"`
}

type ExecuteMsg struct {
	SubmitSlashRequest       *SubmitSlashRequest       `json:"submit_slash_request,omitempty"`
	ExecuteSlashRequest      *ExecuteSlashRequest      `json:"execute_slash_request,omitempty"`
	CancelSlashRequest       *CancelSlashRequest       `json:"cancel_slash_request,omitempty"`
	SetMinimalSlashSignature *SetMinimalSlashSignature `json:"set_minimal_slash_signature,omitempty"`
	SetSlasher               *SetSlasher               `json:"set_slasher,omitempty"`
	SetSlasherValidator      *SetSlasherValidator      `json:"set_slasher_validator,omitempty"`
	TransferOwnership        *TransferOwnership        `json:"transfer_ownership,omitempty"`
	SetRouting               *SetRouting               `json:"set_routing,omitempty"`
}

type CancelSlashRequest struct {
	SlashHash string `json:"slash_hash"`
}

type ExecuteSlashRequest struct {
	Signatures           []string `json:"signatures"`
	SlashHash            string   `json:"slash_hash"`
	ValidatorsPublicKeys []string `json:"validators_public_keys"`
}

type SetMinimalSlashSignature struct {
	MinimalSignature int64 `json:"minimal_signature"`
}

type SetRouting struct {
	DelegationManager string `json:"delegation_manager"`
	StrategyManager   string `json:"strategy_manager"`
}

type SetSlasher struct {
	Slasher string `json:"slasher"`
	Value   bool   `json:"value"`
}

type SetSlasherValidator struct {
	Validators []string `json:"validators"`
	Values     []bool   `json:"values"`
}

type SubmitSlashRequest struct {
	SlashDetails         SubmitSlashRequestSlashDetails `json:"slash_details"`
	ValidatorsPublicKeys []string                       `json:"validators_public_keys"`
}

type SubmitSlashRequestSlashDetails struct {
	EndTime        int64    `json:"end_time"`
	Operator       string   `json:"operator"`
	Reason         string   `json:"reason"`
	Share          string   `json:"share"`
	SlashSignature int64    `json:"slash_signature"`
	SlashValidator []string `json:"slash_validator"`
	Slasher        string   `json:"slasher"`
	StartTime      int64    `json:"start_time"`
	Status         bool     `json:"status"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	GetSlashDetails          *GetSlashDetails          `json:"get_slash_details,omitempty"`
	IsValidator              *IsValidator              `json:"is_validator,omitempty"`
	GetMinimalSlashSignature *GetMinimalSlashSignature `json:"get_minimal_slash_signature,omitempty"`
	CalculateSlashHash       *CalculateSlashHash       `json:"calculate_slash_hash,omitempty"`
}

type CalculateSlashHash struct {
	Sender               string                         `json:"sender"`
	SlashDetails         CalculateSlashHashSlashDetails `json:"slash_details"`
	ValidatorsPublicKeys []string                       `json:"validators_public_keys"`
}

type CalculateSlashHashSlashDetails struct {
	EndTime        int64    `json:"end_time"`
	Operator       string   `json:"operator"`
	Reason         string   `json:"reason"`
	Share          string   `json:"share"`
	SlashSignature int64    `json:"slash_signature"`
	SlashValidator []string `json:"slash_validator"`
	Slasher        string   `json:"slasher"`
	StartTime      int64    `json:"start_time"`
	Status         bool     `json:"status"`
}

type GetMinimalSlashSignature struct {
}

type GetSlashDetails struct {
	SlashHash string `json:"slash_hash"`
}

type IsValidator struct {
	Validator string `json:"validator"`
}

type CalculateSlashHashResponse struct {
	MessageBytes []int64 `json:"message_bytes"`
}

type MinimalSlashSignatureResponse struct {
	MinimalSlashSignature int64 `json:"minimal_slash_signature"`
}

type SlashDetailsResponse struct {
	SlashDetails SlashDetails `json:"slash_details"`
}

type SlashDetails struct {
	EndTime        int64    `json:"end_time"`
	Operator       string   `json:"operator"`
	Reason         string   `json:"reason"`
	Share          string   `json:"share"`
	SlashSignature int64    `json:"slash_signature"`
	SlashValidator []string `json:"slash_validator"`
	Slasher        string   `json:"slasher"`
	StartTime      int64    `json:"start_time"`
	Status         bool     `json:"status"`
}

type ValidatorResponse struct {
	IsValidator bool `json:"is_validator"`
}
