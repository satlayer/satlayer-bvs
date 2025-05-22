// This file was automatically generated from governance-contract/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * Instantiate message for the contract.
 */
export interface InstantiateMsg {
  cw3_instantiate_msg: Cw3InstantiateMsgClass;
  /**
   * Used for administrative operations.
   */
  owner: string;
  registry: string;
  router: string;
}

export interface Cw3InstantiateMsgClass {
  max_voting_period: Duration;
  threshold: Threshold;
  voters: Voter[];
}

/**
 * Duration is a delta of time. You can add it to a BlockInfo or Expiration to move that
 * further in the future. Note that an height-based Duration and a time-based Expiration
 * cannot be combined
 *
 * Time in seconds
 */
export interface Duration {
  height?: number;
  time?: number;
}

/**
 * This defines the different ways tallies can happen.
 *
 * The total_weight used for calculating success as well as the weights of each individual
 * voter used in tallying should be snapshotted at the beginning of the block at which the
 * proposal starts (this is likely the responsibility of a correct cw4 implementation). See
 * also `ThresholdResponse` in the cw3 spec.
 *
 * Declares that a fixed weight of Yes votes is needed to pass. See
 * `ThresholdResponse.AbsoluteCount` in the cw3 spec for details.
 *
 * Declares a percentage of the total weight that must cast Yes votes in order for a
 * proposal to pass. See `ThresholdResponse.AbsolutePercentage` in the cw3 spec for
 * details.
 *
 * Declares a `quorum` of the total votes that must participate in the election in order for
 * the vote to be considered at all. See `ThresholdResponse.ThresholdQuorum` in the cw3 spec
 * for details.
 */
export interface Threshold {
  absolute_count?: AbsoluteCount;
  absolute_percentage?: AbsolutePercentage;
  threshold_quorum?: ThresholdQuorum;
}

export interface AbsoluteCount {
  weight: number;
}

export interface AbsolutePercentage {
  percentage: string;
}

export interface ThresholdQuorum {
  quorum: string;
  threshold: string;
}

export interface Voter {
  addr: string;
  weight: number;
}

export interface ExecuteMsg {
  base?: ExecuteMsgClass;
  extended?: ExtendedExecuteMsg;
}

export interface ExecuteMsgClass {
  propose?: Propose;
  vote?: ExecuteMsgVote;
  execute?: ExecuteMsgExecute;
  close?: Close;
}

export interface Close {
  proposal_id: number;
}

export interface ExecuteMsgExecute {
  proposal_id: number;
}

export interface Propose {
  description: string;
  latest?: Expiration | null;
  msgs: CosmosMsgForEmpty[];
  title: string;
}

/**
 * AtHeight will expire when `env.block.height` >= height
 *
 * AtTime will expire when `env.block.time` >= time
 *
 * Never will never expire. Used to express the empty variant
 */
export interface Expiration {
  at_height?: number;
  at_time?: string;
  never?: Never;
}

export interface Never {}

/**
 * `CosmosMsg::Any` is the replaces the "stargate message" – a message wrapped in a
 * [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any) that is supported by
 * the chain. It behaves the same as `CosmosMsg::Stargate` but has a better name and
 * slightly improved syntax.
 *
 * This is feature-gated at compile time with `cosmwasm_2_0` because a chain running
 * CosmWasm < 2.0 cannot process this.
 */
export interface CosmosMsgForEmpty {
  bank?: BankMsg;
  custom?: Empty;
  any?: AnyMsg;
  wasm?: WASMMsg;
}

/**
 * A message encoded the same way as a protobuf
 * [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
 * This is the same structure as messages in `TxBody` from
 * [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
 */
export interface AnyMsg {
  type_url: string;
  value: string;
}

/**
 * The message types of the bank module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
 *
 * Sends native tokens from the contract to the given address.
 *
 * This is translated to a
 * [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
 * `from_address` is automatically filled with the current contract's address.
 *
 * This will burn the given coins from the contract's account. There is no Cosmos SDK
 * message that performs this, but it can be done by calling the bank keeper. Important if a
 * contract controls significant token supply that must be retired.
 */
export interface BankMsg {
  send?: Send;
  burn?: Burn;
}

export interface Burn {
  amount: Coin[];
}

export interface Coin {
  amount: string;
  denom: string;
}

export interface Send {
  amount: Coin[];
  to_address: string;
}

/**
 * An empty struct that serves as a placeholder in different places, such as contracts that
 * don't set a custom message.
 *
 * It is designed to be expressible in correct JSON and JSON Schema but contains no
 * meaningful data. Previously we used enums without cases, but those cannot represented as
 * valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {}

/**
 * The message types of the wasm module.
 *
 * See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
 *
 * Dispatches a call to another contract at a known address (with known ABI).
 *
 * This is translated to a
 * [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
 * `sender` is automatically filled with the current contract's address.
 *
 * Instantiates a new contracts from previously uploaded Wasm code.
 *
 * The contract address is non-predictable. But it is guaranteed that when emitting the same
 * Instantiate message multiple times, multiple instances on different addresses will be
 * generated. See also Instantiate2.
 *
 * This is translated to a
 * [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
 * `sender` is automatically filled with the current contract's address.
 *
 * Instantiates a new contracts from previously uploaded Wasm code using a predictable
 * address derivation algorithm implemented in [`cosmwasm_std::instantiate2_address`].
 *
 * This is translated to a
 * [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
 * `sender` is automatically filled with the current contract's address. `fix_msg` is
 * automatically set to false.
 *
 * Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
 * customize behavior.
 *
 * Only the contract admin (as defined in wasmd), if any, is able to make this call.
 *
 * This is translated to a
 * [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
 * `sender` is automatically filled with the current contract's address.
 *
 * Sets a new admin (for migrate) on the given contract. Fails if this contract is not
 * currently admin of the target contract.
 *
 * Clears the admin on the given contract, so no more migration possible. Fails if this
 * contract is not currently admin of the target contract.
 */
export interface WASMMsg {
  execute?: WASMMsgExecute;
  instantiate?: Instantiate;
  instantiate2?: Instantiate2;
  migrate?: Migrate;
  update_admin?: UpdateAdmin;
  clear_admin?: ClearAdmin;
}

export interface ClearAdmin {
  contract_addr: string;
}

export interface WASMMsgExecute {
  contract_addr: string;
  funds: Coin[];
  /**
   * msg is the json-encoded ExecuteMsg struct (as raw Binary)
   */
  msg: string;
}

export interface Instantiate {
  admin?: null | string;
  code_id: number;
  funds: Coin[];
  /**
   * A human-readable label for the contract.
   *
   * Valid values should: - not be empty - not be bigger than 128 bytes (or some
   * chain-specific limit) - not start / end with whitespace
   */
  label: string;
  /**
   * msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
   */
  msg: string;
}

export interface Instantiate2 {
  admin?: null | string;
  code_id: number;
  funds: Coin[];
  /**
   * A human-readable label for the contract.
   *
   * Valid values should: - not be empty - not be bigger than 128 bytes (or some
   * chain-specific limit) - not start / end with whitespace
   */
  label: string;
  /**
   * msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
   */
  msg: string;
  salt: string;
}

export interface Migrate {
  contract_addr: string;
  /**
   * msg is the json-encoded MigrateMsg struct that will be passed to the new code
   */
  msg: string;
  /**
   * the code_id of the new logic to place in the given contract
   */
  new_code_id: number;
}

export interface UpdateAdmin {
  admin: string;
  contract_addr: string;
}

export interface ExecuteMsgVote {
  proposal_id: number;
  vote: Vote;
}

/**
 * Marks support for the proposal.
 *
 * Marks opposition to the proposal.
 *
 * Marks participation but does not count towards the ratio of support / opposed
 *
 * Veto is generally to be treated as a No vote. Some implementations may allow certain
 * voters to be able to Veto, or them to be counted stronger than No in some way.
 */
export enum Vote {
  Abstain = "abstain",
  No = "no",
  Veto = "veto",
  Yes = "yes",
}

export interface ExtendedExecuteMsg {
  enable_slashing: SlashingParameters;
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

export interface QueryMsg {
  threshold?: ThresholdClass;
  proposal?: Proposal;
  list_proposals?: ListProposals;
  reverse_proposals?: ReverseProposals;
  vote?: QueryMsgVote;
  list_votes?: ListVotes;
  voter?: QueryMsgVoter;
  list_voters?: ListVoters;
}

export interface ListProposals {
  limit?: number | null;
  start_after?: number | null;
}

export interface ListVoters {
  limit?: number | null;
  start_after?: null | string;
}

export interface ListVotes {
  limit?: number | null;
  proposal_id: number;
  start_after?: null | string;
}

export interface Proposal {
  proposal_id: number;
}

export interface ReverseProposals {
  limit?: number | null;
  start_before?: number | null;
}

export interface ThresholdClass {}

export interface QueryMsgVote {
  proposal_id: number;
  voter: string;
}

export interface QueryMsgVoter {
  address: string;
}

export interface ExtendedQueryMsg {
  service_info: ServiceInfo;
}

export interface ServiceInfo {}
