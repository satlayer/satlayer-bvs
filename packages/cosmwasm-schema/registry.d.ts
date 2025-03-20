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

export interface QueryMsg {
  status?: Status;
  is_service?: string;
  is_operator?: string;
  is_operator_active?: string;
}

export interface Status {
  operator: string;
  service: string;
}
