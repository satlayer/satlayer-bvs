// This file was automatically generated from registry/schema.json.
// DO NOT MODIFY IT BY HAND.

package registry

type IsOperatorResponse bool

type IsOperatorActiveResponse bool

type IsOperatorOptedInToSlashingResponse bool

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
	EnableSlashing                *EnableSlashing                `json:"enable_slashing,omitempty"`
	DisableSlashing               *DisableSlashing               `json:"disable_slashing,omitempty"`
	OperatorOptInToSlashing       *OperatorOptInToSlashing       `json:"operator_opt_in_to_slashing,omitempty"`
	TransferOwnership             *TransferOwnership             `json:"transfer_ownership,omitempty"`
}

type DeregisterOperatorFromService struct {
	Operator string `json:"operator"`
}

type DeregisterServiceFromOperator struct {
	Service string `json:"service"`
}

type DisableSlashing struct {
}

type EnableSlashing struct {
	SlashingParameters EnableSlashingSlashingParameters `json:"slashing_parameters"`
}

type EnableSlashingSlashingParameters struct {
	// The address to which the slashed funds will be sent after the slashing is finalized.
	// None, indicates that the slashed funds will be burned.
	Destination *string `json:"destination"`
	// The maximum percentage of the operator's total stake that can be slashed. The value is
	// represented in bips (basis points), where 100 bips = 1%. And the value must be between 0
	// and 10_000 (inclusive).
	MaxSlashingPercentage int64 `json:"max_slashing_percentage"`
	// The minimum amount of time (in seconds) that the slashing can be delayed before it is
	// executed and finalized. Setting this value to a duration less than the queued withdrawal
	// delay is recommended. To prevent restaker's early withdrawal of their assets from the
	// vault due to the impending slash, defeating the purpose of shared security.
	ResolutionWindow int64 `json:"resolution_window"`
}

type OperatorOptInToSlashing struct {
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

type QueryMsg struct {
	Status                      *Status                      `json:"status,omitempty"`
	IsService                   *string                      `json:"is_service,omitempty"`
	IsOperator                  *string                      `json:"is_operator,omitempty"`
	IsOperatorActive            *string                      `json:"is_operator_active,omitempty"`
	SlashingParameters          *QueryMsgSlashingParameters  `json:"slashing_parameters,omitempty"`
	IsOperatorOptedInToSlashing *IsOperatorOptedInToSlashing `json:"is_operator_opted_in_to_slashing,omitempty"`
}

type IsOperatorOptedInToSlashing struct {
	Height   *int64 `json:"height"`
	Operator string `json:"operator"`
	Service  string `json:"service"`
}

type QueryMsgSlashingParameters struct {
	Height  *int64 `json:"height"`
	Service string `json:"service"`
}

type Status struct {
	Height   *int64 `json:"height"`
	Operator string `json:"operator"`
	Service  string `json:"service"`
}

type SlashingParameters struct {
	// The address to which the slashed funds will be sent after the slashing is finalized.
	// None, indicates that the slashed funds will be burned.
	Destination *string `json:"destination"`
	// The maximum percentage of the operator's total stake that can be slashed. The value is
	// represented in bips (basis points), where 100 bips = 1%. And the value must be between 0
	// and 10_000 (inclusive).
	MaxSlashingPercentage int64 `json:"max_slashing_percentage"`
	// The minimum amount of time (in seconds) that the slashing can be delayed before it is
	// executed and finalized. Setting this value to a duration less than the queued withdrawal
	// delay is recommended. To prevent restaker's early withdrawal of their assets from the
	// vault due to the impending slash, defeating the purpose of shared security.
	ResolutionWindow int64 `json:"resolution_window"`
}
