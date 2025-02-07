package types

type RegisterBVSReq struct {
	RegisterBVS RegisterBVS `json:"register_b_v_s"`
}

type RegisterBVS struct {
	BVSContract BVSContract `json:"bvs_contract"`
}

type BVSContract struct {
	BVSContract string `json:"bvs_contract"`
	ChainName   string `json:"chain_name"`
	ChainID     string `json:"chain_id"`
}

type RegisterOperatorReq struct {
	RegisterOperator RegisterOperator `json:"register_operator_to_b_v_s"`
}

type RegisterOperator struct {
	Operator                   string                     `json:"operator"`
	PublicKey                  string                     `json:"public_key"` // base64
	ContractAddr               string                     `json:"contract_addr"`
	SignatureWithSaltAndExpiry SignatureWithSaltAndExpiry `json:"signature_with_salt_and_expiry"`
}
type SignatureWithSaltAndExpiry struct {
	Sig    string `json:"signature"`
	Salt   string `json:"salt"` // base64
	Expiry uint64 `json:"expiry"`
}

type DeregisterOperatorReq struct {
	DeregisterOperator DeregisterOperator `json:"deregister_operator_from_b_v_s"`
}

type DeregisterOperator struct {
	Operator string `json:"operator"`
}

type UpdateMetadataURIReq struct {
	UpdateMetadataURI UpdateMetadataURI `json:"update_b_v_s_metadata_u_r_i"`
}

type UpdateMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type CancelSaltReq struct {
	CancelSalt CancelSalt `json:"cancel_salt"`
}

type CancelSalt struct {
	Salt string `json:"salt"` // base64
}

type DirectoryTransferOwnershipReq struct {
	TransferOwnership DirectoryTransferOwnership `json:"transfer_ownership"`
}

type DirectoryTransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type BVSDirectoryPauseReq struct {
	Pause struct{} `json:"pause"`
}

type BVSDirectoryUnpauseReq struct {
	Unpause struct{} `json:"unpause"`
}

type BVSDirectorySetPauserReq struct {
	SetPauser BVSDirectorySetPauser `json:"set_pauser"`
}
type BVSDirectorySetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type BVSDirectorySetUnpauserReq struct {
	SetUnpauser BVSDirectorySetUnpauser `json:"set_unpauser"`
}

type BVSDirectorySetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type BVSDirectorySetDelegationManagerReq struct {
	SetDelegationManager BVSDirectorySetDelegationManager `json:"set_delegation_manager"`
}

type BVSDirectorySetDelegationManager struct {
	DelegationManager string `json:"delegation_manager"`
}

type GetOperatorStatusReq struct {
	GetOperatorStatus GetOperatorStatus `json:"get_operator_status"`
}

type GetOperatorStatus struct {
	Operator string `json:"operator"`
	BVS      string `json:"bvs"`
}

type QueryOperatorResp struct {
	Status string `json:"status"`
}

type CalculateDigestHashReq struct {
	CalculateDigestHash CalculateDigestHash `json:"calculate_digest_hash"`
}

type CalculateDigestHash struct {
	OperatorPublicKey string `json:"operator_public_key"` // base64
	BVS               string `json:"bvs"`
	Salt              string `json:"salt"` // base64
	Expiry            uint64 `json:"expiry"`
	ContractAddr      string `json:"contract_addr"`
}

type CalculateDigestHashResp struct {
	DigestHash []byte `json:"digest_hash"`
}

type IsSaltSpentReq struct {
	IsSaltSpent IsSaltSpent `json:"is_salt_spent"`
}
type IsSaltSpent struct {
	Operator string `json:"operator"`
	Salt     string `json:"salt"`
}

type IsSaltSpentResp struct {
	IsSpent bool `json:"is_salt_spent"`
}

type GetDelegationManagerReq struct {
	GetDelegationManager struct{} `json:"get_delegation_manager"`
}

type GetDelegationManagerResp struct {
	DelegationAddr string `json:"delegation_addr"`
}

type GetOwnerReq struct {
	GetOwner struct{} `json:"get_owner"`
}

type GetOwnerResp struct {
	OwnerAddr *string `json:"owner_addr"`
}
type GetOperatorBVSRegistrationTypeHashReq struct {
	GetOperatorBVSRegistrationTypeHash struct{} `json:"get_operator_b_v_s_registration_type_hash"`
}

type GetOperatorBVSRegistrationTypeHashResp struct {
	OperatorBVSRegistrationTypeHash string `json:"operator_bvs_registration_type_hash"`
}

type GetDomainTypeHashReq struct {
	GetDomainTypeHash struct{} `json:"get_domain_type_hash"`
}

type GetDomainTypeHashResp struct {
	DomainTypeHash string `json:"domain_type_hash"`
}

type GetDomainNameReq struct {
	GetDomainName struct{} `json:"get_domain_name"`
}

type GetDomainNameResp struct {
	DomainName string `json:"domain_name"`
}

type GetBVSInfoReq struct {
	GetBVSInfo GetBVSInfo `json:"get_b_v_s_info"`
}
type GetBVSInfo struct {
	BVSHash string `json:"bvs_hash"` //  Get from RegisterBVS event log
}

type GetBVSInfoResp struct {
	BVSContract string `json:"bvs_contract"`
}
