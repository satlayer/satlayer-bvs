// This file was automatically generated from registry/schema.json.
// DO NOT MODIFY IT BY HAND.

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
  registry: SlashingRegistry;
}

export interface SlashingRegistry {
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
  max_slashing_percentage: number;
  /**
   * The minimum amount of time (in seconds) that the slashing can be delayed before it is
   * executed and finalized. It is recommended to set this value to a maximum of withdrawal
   * delay or less.
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

export interface QueryMsg {
  status?: Status;
  is_service?: string;
  is_operator?: string;
  is_operator_active?: string;
  slashing_registry?: SlashingRegistryClass;
  is_operator_opted_in_to_slashing?: IsOperatorOptedInToSlashing;
}

export interface IsOperatorOptedInToSlashing {
  height?: number | null;
  operator: string;
  service: string;
}

export interface SlashingRegistryClass {
  height?: number | null;
  service: string;
}

export interface Status {
  height?: number | null;
  operator: string;
  service: string;
}

export interface SlashingRegistryResponse {
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
  max_slashing_percentage: number;
  /**
   * The minimum amount of time (in seconds) that the slashing can be delayed before it is
   * executed and finalized. It is recommended to set this value to a maximum of withdrawal
   * delay or less.
   */
  resolution_window: number;
}
