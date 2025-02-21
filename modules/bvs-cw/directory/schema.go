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
//    digestHashResponse, err := UnmarshalDigestHashResponse(bytes)
//    bytes, err = digestHashResponse.Marshal()
//
//    bVSInfoResponse, err := UnmarshalBVSInfoResponse(bytes)
//    bytes, err = bVSInfoResponse.Marshal()
//
//    delegationResponse, err := UnmarshalDelegationResponse(bytes)
//    bytes, err = delegationResponse.Marshal()
//
//    domainNameResponse, err := UnmarshalDomainNameResponse(bytes)
//    bytes, err = domainNameResponse.Marshal()
//
//    domainTypeHashResponse, err := UnmarshalDomainTypeHashResponse(bytes)
//    bytes, err = domainTypeHashResponse.Marshal()
//
//    registrationTypeHashResponse, err := UnmarshalRegistrationTypeHashResponse(bytes)
//    bytes, err = registrationTypeHashResponse.Marshal()
//
//    operatorStatusResponse, err := UnmarshalOperatorStatusResponse(bytes)
//    bytes, err = operatorStatusResponse.Marshal()
//
//    ownerResponse, err := UnmarshalOwnerResponse(bytes)
//    bytes, err = ownerResponse.Marshal()
//
//    saltResponse, err := UnmarshalSaltResponse(bytes)
//    bytes, err = saltResponse.Marshal()

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

func UnmarshalDigestHashResponse(data []byte) (DigestHashResponse, error) {
	var r DigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DigestHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalBVSInfoResponse(data []byte) (BVSInfoResponse, error) {
	var r BVSInfoResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *BVSInfoResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegationResponse(data []byte) (DelegationResponse, error) {
	var r DelegationResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDomainNameResponse(data []byte) (DomainNameResponse, error) {
	var r DomainNameResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DomainNameResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDomainTypeHashResponse(data []byte) (DomainTypeHashResponse, error) {
	var r DomainTypeHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DomainTypeHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalRegistrationTypeHashResponse(data []byte) (RegistrationTypeHashResponse, error) {
	var r RegistrationTypeHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *RegistrationTypeHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorStatusResponse(data []byte) (OperatorStatusResponse, error) {
	var r OperatorStatusResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorStatusResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOwnerResponse(data []byte) (OwnerResponse, error) {
	var r OwnerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OwnerResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalSaltResponse(data []byte) (SaltResponse, error) {
	var r SaltResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SaltResponse) Marshal() ([]byte, error) {
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
	RegisterBvs               *RegisterBvs               `json:"register_bvs,omitempty"`
	RegisterOperatorToBvs     *RegisterOperatorToBvs     `json:"register_operator_to_bvs,omitempty"`
	DeregisterOperatorFromBvs *DeregisterOperatorFromBvs `json:"deregister_operator_from_bvs,omitempty"`
	UpdateBvsMetadataURI      *UpdateBvsMetadataURI      `json:"update_bvs_metadata_u_r_i,omitempty"`
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

type DeregisterOperatorFromBvs struct {
	Operator string `json:"operator"`
}

type Pause struct {
}

type RegisterBvs struct {
	BvsContract string `json:"bvs_contract"`
}

type RegisterOperatorToBvs struct {
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

type UpdateBvsMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type QueryMsg struct {
	GetOperatorStatus                  *GetOperatorStatus                  `json:"get_operator_status,omitempty"`
	CalculateDigestHash                *CalculateDigestHash                `json:"calculate_digest_hash,omitempty"`
	IsSaltSpent                        *IsSaltSpent                        `json:"is_salt_spent,omitempty"`
	GetBvsInfo                         *GetBvsInfo                         `json:"get_bvs_info,omitempty"`
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

type GetBvsInfo struct {
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

type DigestHashResponse struct {
	DigestHash string `json:"digest_hash"`
}

type BVSInfoResponse struct {
	BvsContract string `json:"bvs_contract"`
	BvsHash     string `json:"bvs_hash"`
}

type DelegationResponse struct {
	DelegationAddr string `json:"delegation_addr"`
}

type DomainNameResponse struct {
	DomainName string `json:"domain_name"`
}

type DomainTypeHashResponse struct {
	DomainTypeHash string `json:"domain_type_hash"`
}

type RegistrationTypeHashResponse struct {
	OperatorBvsRegistrationTypeHash string `json:"operator_bvs_registration_type_hash"`
}

type OperatorStatusResponse struct {
	Status OperatorBVSRegistrationStatus `json:"status"`
}

type OwnerResponse struct {
	OwnerAddr string `json:"owner_addr"`
}

type SaltResponse struct {
	IsSaltSpent bool `json:"is_salt_spent"`
}

type OperatorBVSRegistrationStatus string

const (
	Registered   OperatorBVSRegistrationStatus = "registered"
	Unregistered OperatorBVSRegistrationStatus = "unregistered"
)
