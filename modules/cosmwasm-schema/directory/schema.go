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
//    statusResponse, err := UnmarshalStatusResponse(bytes)
//    bytes, err = statusResponse.Marshal()

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

type StatusResponse int64

func UnmarshalStatusResponse(data []byte) (StatusResponse, error) {
	var r StatusResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StatusResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Owner  string `json:"owner"`
	Pauser string `json:"pauser"`
}

type ExecuteMsg struct {
	ServiceRegister           *ServiceRegister           `json:"service_register,omitempty"`
	ServiceUpdateMetadata     *ServiceMetadata           `json:"service_update_metadata,omitempty"`
	ServiceRegisterOperator   *ServiceRegisterOperator   `json:"service_register_operator,omitempty"`
	OperatorDeregisterService *OperatorDeregisterService `json:"operator_deregister_service,omitempty"`
	OperatorRegisterService   *OperatorRegisterService   `json:"operator_register_service,omitempty"`
	ServiceDeregisterOperator *ServiceDeregisterOperator `json:"service_deregister_operator,omitempty"`
	TransferOwnership         *TransferOwnership         `json:"transfer_ownership,omitempty"`
	SetRouting                *SetRouting                `json:"set_routing,omitempty"`
}

type OperatorDeregisterService struct {
	Service string `json:"service"`
}

type OperatorRegisterService struct {
	Service string `json:"service"`
}

type ServiceDeregisterOperator struct {
	Operator string `json:"operator"`
}

type ServiceRegister struct {
	Metadata ServiceMetadata `json:"metadata"`
}

// Service metadata is emitted as events and not stored on-chain.
type ServiceMetadata struct {
	Name *string `json:"name"`
	URI  *string `json:"uri"`
}

type ServiceRegisterOperator struct {
	Operator string `json:"operator"`
}

type SetRouting struct {
	DelegationManager string `json:"delegation_manager"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	Status Status `json:"status"`
}

type Status struct {
	Operator string `json:"operator"`
	Service  string `json:"service"`
}
