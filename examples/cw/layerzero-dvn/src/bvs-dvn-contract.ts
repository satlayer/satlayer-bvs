// This file was automatically generated from schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * Instantiate message for the contract.
 */
export interface InstantiateMsg {
  /**
   * Used for administrative operations.
   */
  owner: string;
  registry: string;
  required_verification_threshold: number;
  router: string;
}

/**
 * Execute messages for the contract. These messages allow modifying the state of the
 * contract and emits event.
 *
 * Broadcast a new packet for operators to verify
 *
 * Operator calls this to post their packet verification.
 *
 * Check if quorum has been reached and select the payload hash
 *
 * Resolve the slashing process for the `operator` by canceling it.
 *
 * Move to the 2nd stage of the slashing process, locking the operator vault's funds.
 *
 * Finalize the slashing process for the `operator`.
 *
 * Register a new operator for the squaring service.
 *
 * Deregister an operator from running the squaring service.
 *
 * Enable slashing for the squaring service.
 *
 * Disable slashing for the squaring service.
 */
export interface ExecuteMsg {
  broadcast_packet?: BroadcastPacketClass;
  submit_packet?: SubmitPacket;
  finalize_packet_verification?: FinalizePacketVerification;
  slashing_cancel?: SlashingCancel;
  slashing_lock?: SlashingLock;
  slashing_finalize?: SlashingFinalize;
  register_operator?: RegisterOperator;
  deregister_operator?: DeregisterOperator;
  enable_slashing?: EnableSlashing;
  disable_slashing?: DisableSlashing;
}

export interface BroadcastPacketClass {
  /**
   * The destination endpoint ID
   */
  dst_eid: number;
  /**
   * A global unique identifier (32 bytes)
   */
  guid: string;
  /**
   * The message payload
   */
  message: string;
  /**
   * The nonce of the message in the pathway
   */
  nonce: string;
  /**
   * The receiving address (32 bytes)
   */
  receiver: string;
  /**
   * The sender address (20 bytes)
   */
  sender: string;
  /**
   * The source endpoint ID
   */
  src_eid: number;
}

export interface DeregisterOperator {
  operator: string;
}

export interface DisableSlashing {}

export interface EnableSlashing {}

export interface FinalizePacketVerification {
  packet_guid: string;
}

export interface RegisterOperator {
  operator: string;
}

export interface SlashingCancel {
  operator: string;
}

export interface SlashingFinalize {
  operator: string;
}

export interface SlashingLock {
  operator: string;
}

export interface SubmitPacket {
  packet_guid: string;
  payload_hash: string;
}

/**
 * Query messages for the contract. These messages allow querying the state of the contract.
 * Does not allow modifying the state of the contract.
 *
 * Get response for a given `input` with `operator` that responded to the request.
 */
export interface QueryMsg {
  get_packet?: string;
  get_packet_submission?: GetPacketSubmission;
}

export interface GetPacketSubmission {
  operator: string;
  packet_guid: string;
}

/**
 * Response for the `GetPacket` query.
 */
export interface GetPacketResponse {
  packet: PacketClass;
  packet_broadcasted: string;
  packet_verification_config: PacketVerificationConfig;
  packet_verification_finalized?: PacketFinalized | null;
}

export interface PacketClass {
  /**
   * The destination endpoint ID
   */
  dst_eid: number;
  /**
   * A global unique identifier (32 bytes)
   */
  guid: string;
  /**
   * The message payload
   */
  message: string;
  /**
   * The nonce of the message in the pathway
   */
  nonce: string;
  /**
   * The receiving address (32 bytes)
   */
  receiver: string;
  /**
   * The sender address (20 bytes)
   */
  sender: string;
  /**
   * The source endpoint ID
   */
  src_eid: number;
}

export interface PacketVerificationConfig {
  required_verifications: number;
  total_active_operators: number;
}

export interface PacketFinalized {
  payload_hash: string;
  timestamp: string;
}

/**
 * Response for the `GetPacketSubmission` query.
 */
export interface GetPacketSubmissionResponse {
  payload_hash: string;
  timestamp: string;
}
