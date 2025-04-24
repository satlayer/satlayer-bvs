// This file was automatically generated from registry/schema.json.
// DO NOT MODIFY IT BY HAND.

type IsOperatorResponse = boolean;

type IsOperatorActiveResponse = boolean;

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
  transfer_ownership?: TransferOwnership;
}

export interface DeregisterOperatorFromService {
  operator: string;
}

export interface DeregisterServiceFromOperator {
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
 * - 0: Inactive - Default state when neither the Operator nor the Service has registered,
 * or when either has unregistered
 *
 * - 1: Active - State when both the Operator and Service have registered with each other,
 * indicating a fully established relationship
 *
 * - 2: OperatorRegistered - State when only the Operator has registered but the Service
 * hasn't yet, indicating a pending registration from the Service side
 *
 * - 3: ServiceRegistered - State when only the Service has registered but the Operator
 * hasn't yet, indicating a pending registration from the Operator side
 */
export interface QueryMsg {
  status?: Status;
  is_service?: string;
  is_operator?: string;
  is_operator_active?: string;
}

export interface Status {
  height?: number | null;
  operator: string;
  service: string;
}
