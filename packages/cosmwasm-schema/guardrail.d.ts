// This file was automatically generated from guardrail/schema.json.
// DO NOT MODIFY IT BY HAND.

export interface InstantiateMsg {
  members: MemberElement[];
  owner: string;
  threshold: Threshold;
}

/**
 * A group member has a weight associated with them. This may all be equal, or may have
 * meaning in the app that makes use of the group (eg. voting power)
 */
export interface MemberElement {
  addr: string;
  weight: number;
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
  absolute_count?: ThresholdAbsoluteCount;
  absolute_percentage?: ThresholdAbsolutePercentage;
  threshold_quorum?: ThresholdThresholdQuorum;
}

export interface ThresholdAbsoluteCount {
  weight: number;
}

export interface ThresholdAbsolutePercentage {
  percentage: string;
}

export interface ThresholdThresholdQuorum {
  quorum: string;
  threshold: string;
}

/**
 * apply a diff to the existing members. remove is applied after add, so if an address is in
 * both, it is removed
 */
export interface ExecuteMsg {
  propose?: Propose;
  vote?: ExecuteMsgVote;
  close?: Close;
  update_members?: UpdateMembers;
}

export interface Close {
  slashing_request_id: string;
}

export interface Propose {
  expiration: ProposeExpiration;
  reason: string;
  slashing_request_id: string;
}

/**
 * Expiration represents a point in time when some event happens. It can compare with a
 * BlockInfo and will return is_expired() == true once the condition is hit (and for every
 * block in the future)
 *
 * AtHeight will expire when `env.block.height` >= height
 *
 * AtTime will expire when `env.block.time` >= time
 *
 * Never will never expire. Used to express the empty variant
 */
export interface ProposeExpiration {
  at_height?: number;
  at_time?: string;
  never?: PurpleNever;
}

export interface PurpleNever {}

export interface UpdateMembers {
  add: AddElement[];
  remove: string[];
}

/**
 * A group member has a weight associated with them. This may all be equal, or may have
 * meaning in the app that makes use of the group (eg. voting power)
 */
export interface AddElement {
  addr: string;
  weight: number;
}

export interface ExecuteMsgVote {
  slashing_request_id: string;
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

export interface QueryMsg {
  threshold?: ThresholdClass;
  proposal?: Proposal;
  proposal_by_slashing_request_id?: ProposalBySlashingRequestID;
  list_proposals?: ListProposals;
  vote?: QueryMsgVote;
  vote_by_slashing_request_id?: VoteBySlashingRequestID;
  list_votes?: ListVotes;
  voter?: Voter;
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

export interface ProposalBySlashingRequestID {
  slashing_request_id: string;
}

export interface ThresholdClass {
  height?: number | null;
}

export interface QueryMsgVote {
  proposal_id: number;
  voter: string;
}

export interface VoteBySlashingRequestID {
  slashing_request_id: string;
  voter: string;
}

export interface Voter {
  address: string;
  height?: number | null;
}

export interface ProposalListResponseForEmpty {
  proposals: ProposalElement[];
}

/**
 * Note, if you are storing custom messages in the proposal, the querier needs to know what
 * possible custom message types those are in order to parse the response
 */
export interface ProposalElement {
  deposit?: ProposalDepositInfo | null;
  description: string;
  expires: ProposalExpiration;
  id: number;
  msgs: ProposalCosmosMsgForEmpty[];
  proposer: string;
  status: Status;
  /**
   * This is the threshold that is applied to this proposal. Both the rules of the voting
   * contract, as well as the total_weight of the voting group may have changed since this
   * time. That means that the generic `Threshold{}` query does not provide valid information
   * for existing proposals.
   */
  threshold: ProposalThresholdResponse;
  title: string;
}

/**
 * Information about the deposit required to create a proposal.
 */
export interface ProposalDepositInfo {
  /**
   * The number tokens required for payment.
   */
  amount: string;
  /**
   * The denom of the deposit payment.
   */
  denom: PurpleDenom;
  /**
   * Should failed proposals have their deposits refunded?
   */
  refund_failed_proposals: boolean;
}

/**
 * The denom of the deposit payment.
 */
export interface PurpleDenom {
  native?: string;
  cw20?: string;
}

/**
 * Expiration represents a point in time when some event happens. It can compare with a
 * BlockInfo and will return is_expired() == true once the condition is hit (and for every
 * block in the future)
 *
 * AtHeight will expire when `env.block.height` >= height
 *
 * AtTime will expire when `env.block.time` >= time
 *
 * Never will never expire. Used to express the empty variant
 */
export interface ProposalExpiration {
  at_height?: number;
  at_time?: string;
  never?: FluffyNever;
}

export interface FluffyNever {}

/**
 * `CosmosMsg::Any` is the replaces the "stargate message" – a message wrapped in a
 * [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any) that is supported by
 * the chain. It behaves the same as `CosmosMsg::Stargate` but has a better name and
 * slightly improved syntax.
 *
 * This is feature-gated at compile time with `cosmwasm_2_0` because a chain running
 * CosmWasm < 2.0 cannot process this.
 */
export interface ProposalCosmosMsgForEmpty {
  bank?: PurpleBankMsg;
  custom?: PurpleEmpty;
  any?: PurpleAnyMsg;
  wasm?: PurpleWASMMsg;
}

/**
 * A message encoded the same way as a protobuf
 * [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
 * This is the same structure as messages in `TxBody` from
 * [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
 */
export interface PurpleAnyMsg {
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
export interface PurpleBankMsg {
  send?: PurpleSend;
  burn?: PurpleBurn;
}

export interface PurpleBurn {
  amount: PurpleCoin[];
}

export interface PurpleCoin {
  amount: string;
  denom: string;
}

export interface PurpleSend {
  amount: PurpleCoin[];
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
export interface PurpleEmpty {}

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
export interface PurpleWASMMsg {
  execute?: PurpleExecute;
  instantiate?: PurpleInstantiate;
  instantiate2?: PurpleInstantiate2;
  migrate?: PurpleMigrate;
  update_admin?: PurpleUpdateAdmin;
  clear_admin?: PurpleClearAdmin;
}

export interface PurpleClearAdmin {
  contract_addr: string;
}

export interface PurpleExecute {
  contract_addr: string;
  funds: PurpleCoin[];
  /**
   * msg is the json-encoded ExecuteMsg struct (as raw Binary)
   */
  msg: string;
}

export interface PurpleInstantiate {
  admin?: null | string;
  code_id: number;
  funds: PurpleCoin[];
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

export interface PurpleInstantiate2 {
  admin?: null | string;
  code_id: number;
  funds: PurpleCoin[];
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

export interface PurpleMigrate {
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

export interface PurpleUpdateAdmin {
  admin: string;
  contract_addr: string;
}

/**
 * proposal was created, but voting has not yet begun for whatever reason
 *
 * you can vote on this
 *
 * voting is over and it did not pass
 *
 * voting is over and it did pass, but has not yet executed
 *
 * voting is over it passed, and the proposal was executed
 */
export enum Status {
  Executed = "executed",
  Open = "open",
  Passed = "passed",
  Pending = "pending",
  Rejected = "rejected",
}

/**
 * This is the threshold that is applied to this proposal. Both the rules of the voting
 * contract, as well as the total_weight of the voting group may have changed since this
 * time. That means that the generic `Threshold{}` query does not provide valid information
 * for existing proposals.
 *
 * This defines the different ways tallies can happen. Every contract should support a
 * subset of these, ideally all.
 *
 * The total_weight used for calculating success as well as the weights of each individual
 * voter used in tallying should be snapshotted at the beginning of the block at which the
 * proposal starts (this is likely the responsibility of a correct cw4 implementation).
 *
 * Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
 * no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
 *
 * This is the simplest format and usually suitable for small multisigs of trusted parties,
 * like 3 of 5. (weight: 3, total_weight: 5)
 *
 * A proposal of this type can pass early as soon as the needed weight of yes votes has been
 * cast.
 *
 * Declares a percentage of the total weight that must cast Yes votes, in order for a
 * proposal to pass. The passing weight is computed over the total weight minus the weight
 * of the abstained votes.
 *
 * This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
 * small set of voters, and participation is required. It is understood that if the voting
 * set (group) changes between different proposals that refer to the same group, each
 * proposal will work with a different set of voter weights (the ones snapshotted at
 * proposal creation), and the passing weight for each proposal will be computed based on
 * the absolute percentage, times the total weights of the members at the time of each
 * proposal creation.
 *
 * Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
 * 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
 * but the `total_weight` of the group has increased to 9. That proposal will then
 * automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
 * with `AbsoluteCount`.
 *
 * In addition to a `threshold`, declares a `quorum` of the total votes that must
 * participate in the election in order for the vote to be considered at all. Within the
 * votes that were cast, it requires `threshold` votes in favor. That is calculated by
 * ignoring the Abstain votes (they count towards `quorum`, but do not influence
 * `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
 * `threshold` to consider if the proposal was passed.
 *
 * It is rather difficult for a proposal of this type to pass early. That can only happen if
 * the required quorum has been already met, and there are already enough Yes votes for the
 * proposal to pass.
 *
 * 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
 * cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
 * 50% no). Once the voting period has passed with no additional votes, that same proposal
 * would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
 * we ignore abstain votes).
 *
 * This type is more common in general elections, where participation is often expected to
 * be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
 * percentages to pass, independently of if there was high participation in the election or
 * not.
 */
export interface ProposalThresholdResponse {
  absolute_count?: PurpleAbsoluteCount;
  absolute_percentage?: PurpleAbsolutePercentage;
  threshold_quorum?: PurpleThresholdQuorum;
}

export interface PurpleAbsoluteCount {
  total_weight: number;
  weight: number;
}

export interface PurpleAbsolutePercentage {
  percentage: string;
  total_weight: number;
}

export interface PurpleThresholdQuorum {
  quorum: string;
  threshold: string;
  total_weight: number;
}

export interface VoterListResponse {
  voters: VoterDetail[];
}

export interface VoterDetail {
  addr: string;
  weight: number;
}

export interface VoteListResponse {
  votes: VoteElement[];
}

/**
 * Returns the vote (opinion as well as weight counted) as well as the address of the voter
 * who submitted it
 */
export interface VoteElement {
  proposal_id: number;
  vote: Vote;
  voter: string;
  weight: number;
}

/**
 * Note, if you are storing custom messages in the proposal, the querier needs to know what
 * possible custom message types those are in order to parse the response
 */
export interface ProposalResponseForEmpty {
  deposit?: ProposalResponseForEmptyDepositInfo | null;
  description: string;
  expires: ProposalResponseForEmptyExpiration;
  id: number;
  msgs: ProposalResponseForEmptyCosmosMsgForEmpty[];
  proposer: string;
  status: Status;
  /**
   * This is the threshold that is applied to this proposal. Both the rules of the voting
   * contract, as well as the total_weight of the voting group may have changed since this
   * time. That means that the generic `Threshold{}` query does not provide valid information
   * for existing proposals.
   */
  threshold: ProposalResponseForEmptyThresholdResponse;
  title: string;
}

/**
 * Information about the deposit required to create a proposal.
 */
export interface ProposalResponseForEmptyDepositInfo {
  /**
   * The number tokens required for payment.
   */
  amount: string;
  /**
   * The denom of the deposit payment.
   */
  denom: FluffyDenom;
  /**
   * Should failed proposals have their deposits refunded?
   */
  refund_failed_proposals: boolean;
}

/**
 * The denom of the deposit payment.
 */
export interface FluffyDenom {
  native?: string;
  cw20?: string;
}

/**
 * Expiration represents a point in time when some event happens. It can compare with a
 * BlockInfo and will return is_expired() == true once the condition is hit (and for every
 * block in the future)
 *
 * AtHeight will expire when `env.block.height` >= height
 *
 * AtTime will expire when `env.block.time` >= time
 *
 * Never will never expire. Used to express the empty variant
 */
export interface ProposalResponseForEmptyExpiration {
  at_height?: number;
  at_time?: string;
  never?: TentacledNever;
}

export interface TentacledNever {}

/**
 * `CosmosMsg::Any` is the replaces the "stargate message" – a message wrapped in a
 * [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any) that is supported by
 * the chain. It behaves the same as `CosmosMsg::Stargate` but has a better name and
 * slightly improved syntax.
 *
 * This is feature-gated at compile time with `cosmwasm_2_0` because a chain running
 * CosmWasm < 2.0 cannot process this.
 */
export interface ProposalResponseForEmptyCosmosMsgForEmpty {
  bank?: FluffyBankMsg;
  custom?: FluffyEmpty;
  any?: FluffyAnyMsg;
  wasm?: FluffyWASMMsg;
}

/**
 * A message encoded the same way as a protobuf
 * [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
 * This is the same structure as messages in `TxBody` from
 * [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
 */
export interface FluffyAnyMsg {
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
export interface FluffyBankMsg {
  send?: FluffySend;
  burn?: FluffyBurn;
}

export interface FluffyBurn {
  amount: FluffyCoin[];
}

export interface FluffyCoin {
  amount: string;
  denom: string;
}

export interface FluffySend {
  amount: FluffyCoin[];
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
export interface FluffyEmpty {}

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
export interface FluffyWASMMsg {
  execute?: FluffyExecute;
  instantiate?: FluffyInstantiate;
  instantiate2?: FluffyInstantiate2;
  migrate?: FluffyMigrate;
  update_admin?: FluffyUpdateAdmin;
  clear_admin?: FluffyClearAdmin;
}

export interface FluffyClearAdmin {
  contract_addr: string;
}

export interface FluffyExecute {
  contract_addr: string;
  funds: FluffyCoin[];
  /**
   * msg is the json-encoded ExecuteMsg struct (as raw Binary)
   */
  msg: string;
}

export interface FluffyInstantiate {
  admin?: null | string;
  code_id: number;
  funds: FluffyCoin[];
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

export interface FluffyInstantiate2 {
  admin?: null | string;
  code_id: number;
  funds: FluffyCoin[];
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

export interface FluffyMigrate {
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

export interface FluffyUpdateAdmin {
  admin: string;
  contract_addr: string;
}

/**
 * This is the threshold that is applied to this proposal. Both the rules of the voting
 * contract, as well as the total_weight of the voting group may have changed since this
 * time. That means that the generic `Threshold{}` query does not provide valid information
 * for existing proposals.
 *
 * This defines the different ways tallies can happen. Every contract should support a
 * subset of these, ideally all.
 *
 * The total_weight used for calculating success as well as the weights of each individual
 * voter used in tallying should be snapshotted at the beginning of the block at which the
 * proposal starts (this is likely the responsibility of a correct cw4 implementation).
 *
 * Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
 * no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
 *
 * This is the simplest format and usually suitable for small multisigs of trusted parties,
 * like 3 of 5. (weight: 3, total_weight: 5)
 *
 * A proposal of this type can pass early as soon as the needed weight of yes votes has been
 * cast.
 *
 * Declares a percentage of the total weight that must cast Yes votes, in order for a
 * proposal to pass. The passing weight is computed over the total weight minus the weight
 * of the abstained votes.
 *
 * This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
 * small set of voters, and participation is required. It is understood that if the voting
 * set (group) changes between different proposals that refer to the same group, each
 * proposal will work with a different set of voter weights (the ones snapshotted at
 * proposal creation), and the passing weight for each proposal will be computed based on
 * the absolute percentage, times the total weights of the members at the time of each
 * proposal creation.
 *
 * Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
 * 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
 * but the `total_weight` of the group has increased to 9. That proposal will then
 * automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
 * with `AbsoluteCount`.
 *
 * In addition to a `threshold`, declares a `quorum` of the total votes that must
 * participate in the election in order for the vote to be considered at all. Within the
 * votes that were cast, it requires `threshold` votes in favor. That is calculated by
 * ignoring the Abstain votes (they count towards `quorum`, but do not influence
 * `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
 * `threshold` to consider if the proposal was passed.
 *
 * It is rather difficult for a proposal of this type to pass early. That can only happen if
 * the required quorum has been already met, and there are already enough Yes votes for the
 * proposal to pass.
 *
 * 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
 * cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
 * 50% no). Once the voting period has passed with no additional votes, that same proposal
 * would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
 * we ignore abstain votes).
 *
 * This type is more common in general elections, where participation is often expected to
 * be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
 * percentages to pass, independently of if there was high participation in the election or
 * not.
 */
export interface ProposalResponseForEmptyThresholdResponse {
  absolute_count?: FluffyAbsoluteCount;
  absolute_percentage?: FluffyAbsolutePercentage;
  threshold_quorum?: FluffyThresholdQuorum;
}

export interface FluffyAbsoluteCount {
  total_weight: number;
  weight: number;
}

export interface FluffyAbsolutePercentage {
  percentage: string;
  total_weight: number;
}

export interface FluffyThresholdQuorum {
  quorum: string;
  threshold: string;
  total_weight: number;
}

/**
 * This defines the different ways tallies can happen. Every contract should support a
 * subset of these, ideally all.
 *
 * The total_weight used for calculating success as well as the weights of each individual
 * voter used in tallying should be snapshotted at the beginning of the block at which the
 * proposal starts (this is likely the responsibility of a correct cw4 implementation).
 *
 * Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
 * no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
 *
 * This is the simplest format and usually suitable for small multisigs of trusted parties,
 * like 3 of 5. (weight: 3, total_weight: 5)
 *
 * A proposal of this type can pass early as soon as the needed weight of yes votes has been
 * cast.
 *
 * Declares a percentage of the total weight that must cast Yes votes, in order for a
 * proposal to pass. The passing weight is computed over the total weight minus the weight
 * of the abstained votes.
 *
 * This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
 * small set of voters, and participation is required. It is understood that if the voting
 * set (group) changes between different proposals that refer to the same group, each
 * proposal will work with a different set of voter weights (the ones snapshotted at
 * proposal creation), and the passing weight for each proposal will be computed based on
 * the absolute percentage, times the total weights of the members at the time of each
 * proposal creation.
 *
 * Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
 * 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
 * but the `total_weight` of the group has increased to 9. That proposal will then
 * automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
 * with `AbsoluteCount`.
 *
 * In addition to a `threshold`, declares a `quorum` of the total votes that must
 * participate in the election in order for the vote to be considered at all. Within the
 * votes that were cast, it requires `threshold` votes in favor. That is calculated by
 * ignoring the Abstain votes (they count towards `quorum`, but do not influence
 * `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
 * `threshold` to consider if the proposal was passed.
 *
 * It is rather difficult for a proposal of this type to pass early. That can only happen if
 * the required quorum has been already met, and there are already enough Yes votes for the
 * proposal to pass.
 *
 * 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
 * cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
 * 50% no). Once the voting period has passed with no additional votes, that same proposal
 * would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
 * we ignore abstain votes).
 *
 * This type is more common in general elections, where participation is often expected to
 * be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
 * percentages to pass, independently of if there was high participation in the election or
 * not.
 */
export interface ThresholdResponse {
  absolute_count?: TentacledAbsoluteCount;
  absolute_percentage?: TentacledAbsolutePercentage;
  threshold_quorum?: TentacledThresholdQuorum;
}

export interface TentacledAbsoluteCount {
  total_weight: number;
  weight: number;
}

export interface TentacledAbsolutePercentage {
  percentage: string;
  total_weight: number;
}

export interface TentacledThresholdQuorum {
  quorum: string;
  threshold: string;
  total_weight: number;
}

export interface VoteResponse {
  vote?: VoteInfo | null;
}

/**
 * Returns the vote (opinion as well as weight counted) as well as the address of the voter
 * who submitted it
 */
export interface VoteInfo {
  proposal_id: number;
  vote: Vote;
  voter: string;
  weight: number;
}

export interface VoterResponse {
  weight?: number | null;
}
