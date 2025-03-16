// This file was automatically generated from directory/schema.json.
// DO NOT MODIFY IT BY HAND.

package directory

type StatusResponse int64

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
