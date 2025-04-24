// This file was automatically generated from registry/schema.json.
// DO NOT MODIFY IT BY HAND.

package registry

type IsOperatorResponse bool

type IsOperatorActiveResponse bool

type IsServiceResponse bool

type StatusResponse int64

type InstantiateMsg struct {
	Owner  string `json:"owner"`
	Pauser string `json:"pauser"`
}

type ExecuteMsg struct {
	RegisterAsService             *RegisterAsService             `json:"register_as_service,omitempty"`
	UpdateServiceMetadata         *Metadata                      `json:"update_service_metadata,omitempty"`
	RegisterAsOperator            *RegisterAsOperator            `json:"register_as_operator,omitempty"`
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
	Metadata Metadata `json:"metadata"`
}

// metadata is emitted as events and not stored on-chain.
type Metadata struct {
	Name *string `json:"name"`
	URI  *string `json:"uri"`
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

// QueryMsg Status: Returns the registration status of an operator to a service The response
// is a StatusResponse that contains a u8 value that maps to a RegistrationStatus:
//
// - 0: Inactive: Default state when neither the Operator nor the Service has registered, or
// when either has unregistered
//
// - 1: Active: State when both the Operator and Service have registered with each other,
// indicating a fully established relationship
//
// - 2: OperatorRegistered: State when only the Operator has registered but the Service
// hasn't yet, indicating a pending registration from the Service side
//
// - 3: ServiceRegistered: State when only the Service has registered but the Operator
// hasn't yet, indicating a pending registration from the Operator side
type QueryMsg struct {
	Status           *Status `json:"status,omitempty"`
	IsService        *string `json:"is_service,omitempty"`
	IsOperator       *string `json:"is_operator,omitempty"`
	IsOperatorActive *string `json:"is_operator_active,omitempty"`
}

type Status struct {
	Height   *int64 `json:"height"`
	Operator string `json:"operator"`
	Service  string `json:"service"`
}
