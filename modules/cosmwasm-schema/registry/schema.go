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
//    isOperatorResponse, err := UnmarshalIsOperatorResponse(bytes)
//    bytes, err = isOperatorResponse.Marshal()
//
//    isServiceResponse, err := UnmarshalIsServiceResponse(bytes)
//    bytes, err = isServiceResponse.Marshal()
//
//    operatorDetailsResponse, err := UnmarshalOperatorDetailsResponse(bytes)
//    bytes, err = operatorDetailsResponse.Marshal()
//
//    statusResponse, err := UnmarshalStatusResponse(bytes)
//    bytes, err = statusResponse.Marshal()

package registry

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

type IsOperatorResponse bool

func UnmarshalIsOperatorResponse(data []byte) (IsOperatorResponse, error) {
	var r IsOperatorResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsOperatorResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type IsServiceResponse bool

func UnmarshalIsServiceResponse(data []byte) (IsServiceResponse, error) {
	var r IsServiceResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsServiceResponse) Marshal() ([]byte, error) {
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
	RegisterAsService             *RegisterAsService             `json:"register_as_service,omitempty"`
	UpdateServiceMetadata         *Metadata                      `json:"update_service_metadata,omitempty"`
	RegisterAsOperator            *RegisterAsOperator            `json:"register_as_operator,omitempty"`
	UpdateOperatorDetails         *UpdateOperatorDetailsClass    `json:"update_operator_details,omitempty"`
	UpdateOperatorMetadata        *Metadata                      `json:"update_operator_metadata,omitempty"`
	RegisterOperatorToService     *RegisterOperatorToService     `json:"register_operator_to_service,omitempty"`
	DeregisterOperatorFromService *DeregisterOperatorFromService `json:"deregister_operator_from_service,omitempty"`
	RegisterServiceToOperator     *RegisterServiceToOperator     `json:"register_service_to_operator,omitempty"`
	DeregisterServiceFromOperator *DeregisterServiceFromOperator `json:"deregister_service_from_operator,omitempty"`
	TransferOwnership             *TransferOwnership             `json:"transfer_ownership,omitempty"`
}

type DeregisterOperatorFromService struct {
	Operator string `json:"operator"`
}

type DeregisterServiceFromOperator struct {
	Service string `json:"service"`
}

type RegisterAsOperator struct {
	Metadata        Metadata                   `json:"metadata"`
	OperatorDetails UpdateOperatorDetailsClass `json:"operator_details"`
}

// metadata is emitted as events and not stored on-chain.
type Metadata struct {
	Name *string `json:"name"`
	URI  *string `json:"uri"`
}

type UpdateOperatorDetailsClass struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}

type RegisterAsService struct {
	Metadata Metadata `json:"metadata"`
}

type RegisterOperatorToService struct {
	Operator string `json:"operator"`
}

type RegisterServiceToOperator struct {
	Service string `json:"service"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	Status          *Status `json:"status,omitempty"`
	IsService       *string `json:"is_service,omitempty"`
	IsOperator      *string `json:"is_operator,omitempty"`
	OperatorDetails *string `json:"operator_details,omitempty"`
}

type Status struct {
	Operator string `json:"operator"`
	Service  string `json:"service"`
}

type OperatorDetailsResponse struct {
	Details DetailsClass `json:"details"`
}

type DetailsClass struct {
	StakerOptOutWindowBlocks int64 `json:"staker_opt_out_window_blocks"`
}
