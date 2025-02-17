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

package slashmanager

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
	DelegationManager   string `json:"delegation_manager"`
	InitialOwner        string `json:"initial_owner"`
	InitialPausedStatus int64  `json:"initial_paused_status"`
	Pauser              string `json:"pauser"`
	StrategyManager     string `json:"strategy_manager"`
	Unpauser            string `json:"unpauser"`
}

type ExecuteMsg struct {
	SubmitSlashRequest       *SubmitSlashRequest       `json:"submit_slash_request,omitempty"`
	ExecuteSlashRequest      *ExecuteSlashRequest      `json:"execute_slash_request,omitempty"`
	CancelSlashRequest       *CancelSlashRequest       `json:"cancel_slash_request,omitempty"`
	SetMinimalSlashSignature *SetMinimalSlashSignature `json:"set_minimal_slash_signature,omitempty"`
	SetSlasher               *SetSlasher               `json:"set_slasher,omitempty"`
	SetSlasherValidator      *SetSlasherValidator      `json:"set_slasher_validator,omitempty"`
	SetDelegationManager     *SetDelegationManager     `json:"set_delegation_manager,omitempty"`
	SetStrategyManager       *SetStrategyManager       `json:"set_strategy_manager,omitempty"`
	TransferOwnership        *TransferOwnership        `json:"transfer_ownership,omitempty"`
	Pause                    *Pause                    `json:"pause,omitempty"`
	Unpause                  *Unpause                  `json:"unpause,omitempty"`
	SetPauser                *SetPauser                `json:"set_pauser,omitempty"`
	SetUnpauser              *SetUnpauser              `json:"set_unpauser,omitempty"`
}

type CancelSlashRequest struct {
	SlashHash string `json:"slash_hash"`
}

type ExecuteSlashRequest struct {
	Signatures           []string `json:"signatures"`
	SlashHash            string   `json:"slash_hash"`
	ValidatorsPublicKeys []string `json:"validators_public_keys"`
}

type Pause struct {
}

type SetDelegationManager struct {
	NewDelegationManager string `json:"new_delegation_manager"`
}

type SetMinimalSlashSignature struct {
	MinimalSignature int64 `json:"minimal_signature"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetSlasher struct {
	Slasher string `json:"slasher"`
	Value   bool   `json:"value"`
}

type SetSlasherValidator struct {
	Validators []string `json:"validators"`
	Values     []bool   `json:"values"`
}

type SetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
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
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
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
