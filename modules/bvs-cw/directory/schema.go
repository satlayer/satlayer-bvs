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
//    bvsInfoResponse, err := UnmarshalBvsInfoResponse(bytes)
//    bytes, err = bvsInfoResponse.Marshal()
//
//    calculateDigestHashResponse, err := UnmarshalCalculateDigestHashResponse(bytes)
//    bytes, err = calculateDigestHashResponse.Marshal()
//
//    delegationManagerResponse, err := UnmarshalDelegationManagerResponse(bytes)
//    bytes, err = delegationManagerResponse.Marshal()
//
//    domainNameResponse, err := UnmarshalDomainNameResponse(bytes)
//    bytes, err = domainNameResponse.Marshal()
//
//    domainTypeHashResponse, err := UnmarshalDomainTypeHashResponse(bytes)
//    bytes, err = domainTypeHashResponse.Marshal()
//
//    isSaltSpentResponse, err := UnmarshalIsSaltSpentResponse(bytes)
//    bytes, err = isSaltSpentResponse.Marshal()
//
//    operatorBvsRegistrationTypeHashResponse, err := UnmarshalOperatorBvsRegistrationTypeHashResponse(bytes)
//    bytes, err = operatorBvsRegistrationTypeHashResponse.Marshal()
//
//    operatorStatusResponse, err := UnmarshalOperatorStatusResponse(bytes)
//    bytes, err = operatorStatusResponse.Marshal()

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

func UnmarshalBvsInfoResponse(data []byte) (BvsInfoResponse, error) {
	var r BvsInfoResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *BvsInfoResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalCalculateDigestHashResponse(data []byte) (CalculateDigestHashResponse, error) {
	var r CalculateDigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateDigestHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegationManagerResponse(data []byte) (DelegationManagerResponse, error) {
	var r DelegationManagerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationManagerResponse) Marshal() ([]byte, error) {
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

func UnmarshalIsSaltSpentResponse(data []byte) (IsSaltSpentResponse, error) {
	var r IsSaltSpentResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsSaltSpentResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOperatorBvsRegistrationTypeHashResponse(data []byte) (OperatorBvsRegistrationTypeHashResponse, error) {
	var r OperatorBvsRegistrationTypeHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OperatorBvsRegistrationTypeHashResponse) Marshal() ([]byte, error) {
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

type InstantiateMsg struct {
	DelegationManager string `json:"delegation_manager"`
	Owner             string `json:"owner"`
	Registry          string `json:"registry"`
}

type ExecuteMsg struct {
	RegisterBvs               *RegisterBvs               `json:"register_bvs,omitempty"`
	RegisterOperatorToBvs     *RegisterOperatorToBvs     `json:"register_operator_to_bvs,omitempty"`
	DeregisterOperatorFromBvs *DeregisterOperatorFromBvs `json:"deregister_operator_from_bvs,omitempty"`
	UpdateBvsMetadataURI      *UpdateBvsMetadataURI      `json:"update_bvs_metadata_uri,omitempty"`
	SetDelegationManager      *SetDelegationManager      `json:"set_delegation_manager,omitempty"`
	CancelSalt                *CancelSalt                `json:"cancel_salt,omitempty"`
	TransferOwnership         *TransferOwnership         `json:"transfer_ownership,omitempty"`
}

type CancelSalt struct {
	Salt string `json:"salt"`
}

type DeregisterOperatorFromBvs struct {
	Operator string `json:"operator"`
}

type RegisterBvs struct {
	BvsContract string `json:"bvs_contract"`
}

type RegisterOperatorToBvs struct {
	ContractAddr               string                     `json:"contract_addr"`
	Operator                   string                     `json:"operator"`
	PublicKey                  string                     `json:"public_key"`
	SignatureWithSaltAndExpiry SignatureWithSaltAndExpiry `json:"signature_with_salt_and_expiry"`
}

type SignatureWithSaltAndExpiry struct {
	Expiry    int64  `json:"expiry"`
	Salt      string `json:"salt"`
	Signature string `json:"signature"`
}

type SetDelegationManager struct {
	DelegationManager string `json:"delegation_manager"`
}

type TransferOwnership struct {
	// See `ownership::transfer_ownership` for more information on this field
	NewOwner string `json:"new_owner"`
}

type UpdateBvsMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type QueryMsg struct {
	OperatorStatus                  *OperatorStatus                  `json:"operator_status,omitempty"`
	CalculateDigestHash             *CalculateDigestHash             `json:"calculate_digest_hash,omitempty"`
	IsSaltSpent                     *IsSaltSpent                     `json:"is_salt_spent,omitempty"`
	BvsInfo                         *BvsInfo                         `json:"bvs_info,omitempty"`
	DelegationManager               *DelegationManager               `json:"delegation_manager,omitempty"`
	OperatorBvsRegistrationTypeHash *OperatorBvsRegistrationTypeHash `json:"operator_bvs_registration_type_hash,omitempty"`
	DomainTypeHash                  *DomainTypeHash                  `json:"domain_type_hash,omitempty"`
	DomainName                      *DomainName                      `json:"domain_name,omitempty"`
}

type BvsInfo struct {
	BvsHash string `json:"bvs_hash"`
}

type CalculateDigestHash struct {
	Bvs               string `json:"bvs"`
	ContractAddr      string `json:"contract_addr"`
	Expiry            int64  `json:"expiry"`
	OperatorPublicKey string `json:"operator_public_key"`
	Salt              string `json:"salt"`
}

type DelegationManager struct {
}

type DomainName struct {
}

type DomainTypeHash struct {
}

type IsSaltSpent struct {
	Operator string `json:"operator"`
	Salt     string `json:"salt"`
}

type OperatorBvsRegistrationTypeHash struct {
}

type OperatorStatus struct {
	Bvs      string `json:"bvs"`
	Operator string `json:"operator"`
}

type BvsInfoResponse struct {
	BvsContract string `json:"bvs_contract"`
	BvsHash     string `json:"bvs_hash"`
}

type CalculateDigestHashResponse struct {
	DigestHash string `json:"digest_hash"`
}

type DelegationManagerResponse struct {
	DelegationAddr string `json:"delegation_addr"`
}

type DomainNameResponse struct {
	DomainName string `json:"domain_name"`
}

type DomainTypeHashResponse struct {
	DomainTypeHash string `json:"domain_type_hash"`
}

type IsSaltSpentResponse struct {
	IsSaltSpent bool `json:"is_salt_spent"`
}

type OperatorBvsRegistrationTypeHashResponse struct {
	OperatorBvsRegistrationTypeHash string `json:"operator_bvs_registration_type_hash"`
}

type OperatorStatusResponse struct {
	Status OperatorBvsRegistrationStatus `json:"status"`
}

type OperatorBvsRegistrationStatus string

const (
	Registered   OperatorBvsRegistrationStatus = "registered"
	Unregistered OperatorBvsRegistrationStatus = "unregistered"
)
