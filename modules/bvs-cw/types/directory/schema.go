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

package directory

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
	Unpauser            string `json:"unpauser"`
}

type ExecuteMsg struct {
	RegisterBVS               *RegisterBVS               `json:"register_b_v_s,omitempty"`
	RegisterOperatorToBVS     *RegisterOperatorToBVS     `json:"register_operator_to_b_v_s,omitempty"`
	DeregisterOperatorFromBVS *DeregisterOperatorFromBVS `json:"deregister_operator_from_b_v_s,omitempty"`
	UpdateBVSMetadataURI      *UpdateBVSMetadataURI      `json:"update_b_v_s_metadata_u_r_i,omitempty"`
	SetDelegationManager      *SetDelegationManager      `json:"set_delegation_manager,omitempty"`
	CancelSalt                *CancelSalt                `json:"cancel_salt,omitempty"`
	TransferOwnership         *TransferOwnership         `json:"transfer_ownership,omitempty"`
	Pause                     *Pause                     `json:"pause,omitempty"`
	Unpause                   *Unpause                   `json:"unpause,omitempty"`
	SetPauser                 *SetPauser                 `json:"set_pauser,omitempty"`
	SetUnpauser               *SetUnpauser               `json:"set_unpauser,omitempty"`
}

type CancelSalt struct {
	Salt string `json:"salt"`
}

type DeregisterOperatorFromBVS struct {
	Operator string `json:"operator"`
}

type Pause struct {
}

type RegisterBVS struct {
	BvsContract string `json:"bvs_contract"`
}

type RegisterOperatorToBVS struct {
	ContractAddr               string                            `json:"contract_addr"`
	Operator                   string                            `json:"operator"`
	PublicKey                  string                            `json:"public_key"`
	SignatureWithSaltAndExpiry ExecuteSignatureWithSaltAndExpiry `json:"signature_with_salt_and_expiry"`
}

type ExecuteSignatureWithSaltAndExpiry struct {
	Expiry    int64  `json:"expiry"`
	Salt      string `json:"salt"`
	Signature string `json:"signature"`
}

type SetDelegationManager struct {
	DelegationManager string `json:"delegation_manager"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
}

type UpdateBVSMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type QueryMsg struct {
	GetOperatorStatus                  *GetOperatorStatus                  `json:"get_operator_status,omitempty"`
	CalculateDigestHash                *CalculateDigestHash                `json:"calculate_digest_hash,omitempty"`
	IsSaltSpent                        *IsSaltSpent                        `json:"is_salt_spent,omitempty"`
	GetBVSInfo                         *GetBVSInfo                         `json:"get_b_v_s_info,omitempty"`
	GetDelegationManager               *GetDelegationManager               `json:"get_delegation_manager,omitempty"`
	GetOwner                           *GetOwner                           `json:"get_owner,omitempty"`
	GetOperatorBVSRegistrationTypeHash *GetOperatorBVSRegistrationTypeHash `json:"get_operator_b_v_s_registration_type_hash,omitempty"`
	GetDomainTypeHash                  *GetDomainTypeHash                  `json:"get_domain_type_hash,omitempty"`
	GetDomainName                      *GetDomainName                      `json:"get_domain_name,omitempty"`
}

type CalculateDigestHash struct {
	Bvs               string `json:"bvs"`
	ContractAddr      string `json:"contract_addr"`
	Expiry            int64  `json:"expiry"`
	OperatorPublicKey string `json:"operator_public_key"`
	Salt              string `json:"salt"`
}

type GetBVSInfo struct {
	BvsHash string `json:"bvs_hash"`
}

type GetDelegationManager struct {
}

type GetDomainName struct {
}

type GetDomainTypeHash struct {
}

type GetOperatorBVSRegistrationTypeHash struct {
}

type GetOperatorStatus struct {
	Bvs      string `json:"bvs"`
	Operator string `json:"operator"`
}

type GetOwner struct {
}

type IsSaltSpent struct {
	Operator string `json:"operator"`
	Salt     string `json:"salt"`
}
