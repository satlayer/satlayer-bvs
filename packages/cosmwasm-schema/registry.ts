// This file was automatically generated from registry/schema.json.
// DO NOT MODIFY IT BY HAND.

type Uint64 = number;

type IsOperatorResponse = boolean;

type IsOperatorActiveResponse = boolean;

type IsOperatorOptedInToSlashingResponse = boolean;

type IsServiceResponse = boolean;

type StatusResponse = number;

export interface InstantiateMsg {
  owner: string;
  pauser: string;
}

export interface ExecuteMsg {
  register_as_service?: RegisterAsService;
  update_service_metadata?: Metadata;
  register_as_operator?: RegisterAsOperator;
  update_operator_metadata?: Metadata;
  register_operator_to_service?: RegisterOperatorToService;
  deregister_operator_from_service?: DeregisterOperatorFromService;
  register_service_to_operator?: RegisterServiceToOperator;
  deregister_service_from_operator?: DeregisterServiceFromOperator;
  enable_slashing?: EnableSlashing;
  disable_slashing?: DisableSlashing;
  operator_opt_in_to_slashing?: OperatorOptInToSlashing;
  transfer_ownership?: TransferOwnership;
}

export interface DeregisterOperatorFromService {
  operator: string;
}

export interface DeregisterServiceFromOperator {
  service: string;
}

export interface DisableSlashing {}

export interface EnableSlashing {
  slashing_parameters: SlashingParameters;
}

export interface SlashingParameters {
  /**
   * The address to which the slashed funds will be sent after the slashing is finalized.
   * None, indicates that the slashed funds will be burned.
   */
  destination?: null | string;
  /**
   * The maximum percentage of the operator's total stake that can be slashed. The value is
   * represented in bips (basis points), where 100 bips = 1%. And the value must be between 0
   * and 10_000 (inclusive).
   */
  max_slashing_bips: number;
  /**
   * The minimum amount of time (in seconds) that the slashing can be delayed before it is
   * executed and finalized. Setting this value to a duration less than the queued withdrawal
   * delay is recommended. To prevent restaker's early withdrawal of their assets from the
   * vault due to the impending slash, defeating the purpose of shared security.
   */
  resolution_window: number;
}

export interface OperatorOptInToSlashing {
  service: string;
}

export interface RegisterAsOperator {
  metadata: Metadata;
}

/**
 * metadata is emitted as events and not stored on-chain.
 */
export interface Metadata {
  name?: null | string;
  uri?: null | string;
}

export interface RegisterAsService {
  metadata: Metadata;
}

export interface RegisterOperatorToService {
  operator: string;
}

export interface RegisterServiceToOperator {
  service: string;
}

export interface TransferOwnership {
  /**
   * See [`bvs_library::ownership::transfer_ownership`] for more information on this field
   */
  new_owner: string;
}

/**
 * QueryMsg Status: Returns the registration status of an operator to a service The response
 * is a StatusResponse that contains a u8 value that maps to a RegistrationStatus:
 *
 * - 0: Inactive: Default state when neither the Operator nor the Service has registered, or
 * when either has unregistered
 *
 * - 1: Active: State when both the Operator and Service have registered with each other,
 * indicating a fully established relationship
 *
 * - 2: OperatorRegistered: State when only the Operator has registered but the Service
 * hasn't yet, indicating a pending registration from the Service side
 *
 * - 3: ServiceRegistered: State when only the Service has registered but the Operator
 * hasn't yet, indicating a pending registration from the Operator side
 */
export interface QueryMsg {
  status?: Status;
  is_service?: string;
  is_operator?: string;
  is_operator_active?: string;
  slashing_parameters?: QueryMsgSlashingParameters;
  is_operator_opted_in_to_slashing?: IsOperatorOptedInToSlashing;
  active_operators_count?: ActiveOperatorsCount;
  active_services_count?: ActiveServicesCount;
}

export interface ActiveOperatorsCount {
  service: string;
}

export interface ActiveServicesCount {
  operator: string;
}

export interface IsOperatorOptedInToSlashing {
  operator: string;
  service: string;
  timestamp?: number | null;
}

export interface QueryMsgSlashingParameters {
  service: string;
  timestamp?: number | null;
}

export interface Status {
  operator: string;
  service: string;
  timestamp?: number | null;
}

export interface SlashingParametersResponse {
  /**
   * The address to which the slashed funds will be sent after the slashing is finalized.
   * None, indicates that the slashed funds will be burned.
   */
  destination?: null | string;
  /**
   * The maximum percentage of the operator's total stake that can be slashed. The value is
   * represented in bips (basis points), where 100 bips = 1%. And the value must be between 0
   * and 10_000 (inclusive).
   */
  max_slashing_bips: number;
  /**
   * The minimum amount of time (in seconds) that the slashing can be delayed before it is
   * executed and finalized. Setting this value to a duration less than the queued withdrawal
   * delay is recommended. To prevent restaker's early withdrawal of their assets from the
   * vault due to the impending slash, defeating the purpose of shared security.
   */
  resolution_window: number;
}
