// This file was automatically generated from guardrail/schema.json.
// DO NOT MODIFY IT BY HAND.

package guardrail

type InstantiateMsg struct {
	Members   []MemberElement `json:"members"`
	Owner     string          `json:"owner"`
	Threshold Threshold       `json:"threshold"`
}

// A group member has a weight associated with them. This may all be equal, or may have
// meaning in the app that makes use of the group (eg. voting power)
type MemberElement struct {
	Addr   string `json:"addr"`
	Weight int64  `json:"weight"`
}

// This defines the different ways tallies can happen.
//
// The total_weight used for calculating success as well as the weights of each individual
// voter used in tallying should be snapshotted at the beginning of the block at which the
// proposal starts (this is likely the responsibility of a correct cw4 implementation). See
// also `ThresholdResponse` in the cw3 spec.
//
// Declares that a fixed weight of Yes votes is needed to pass. See
// `ThresholdResponse.AbsoluteCount` in the cw3 spec for details.
//
// Declares a percentage of the total weight that must cast Yes votes in order for a
// proposal to pass. See `ThresholdResponse.AbsolutePercentage` in the cw3 spec for
// details.
//
// Declares a `quorum` of the total votes that must participate in the election in order for
// the vote to be considered at all. See `ThresholdResponse.ThresholdQuorum` in the cw3 spec
// for details.
type Threshold struct {
	AbsoluteCount      *ThresholdAbsoluteCount      `json:"absolute_count,omitempty"`
	AbsolutePercentage *ThresholdAbsolutePercentage `json:"absolute_percentage,omitempty"`
	ThresholdQuorum    *ThresholdThresholdQuorum    `json:"threshold_quorum,omitempty"`
}

type ThresholdAbsoluteCount struct {
	Weight int64 `json:"weight"`
}

type ThresholdAbsolutePercentage struct {
	Percentage string `json:"percentage"`
}

type ThresholdThresholdQuorum struct {
	Quorum    string `json:"quorum"`
	Threshold string `json:"threshold"`
}

// apply a diff to the existing members. remove is applied after add, so if an address is in
// both, it is removed
type ExecuteMsg struct {
	Propose       *Propose        `json:"propose,omitempty"`
	Vote          *ExecuteMsgVote `json:"vote,omitempty"`
	Close         *Close          `json:"close,omitempty"`
	UpdateMembers *UpdateMembers  `json:"update_members,omitempty"`
}

type Close struct {
	SlashingRequestID string `json:"slashing_request_id"`
}

type Propose struct {
	Expiration        ProposeExpiration `json:"expiration"`
	Reason            string            `json:"reason"`
	SlashingRequestID string            `json:"slashing_request_id"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type ProposeExpiration struct {
	AtHeight *int64       `json:"at_height,omitempty"`
	AtTime   *string      `json:"at_time,omitempty"`
	Never    *PurpleNever `json:"never,omitempty"`
}

type PurpleNever struct {
}

type UpdateMembers struct {
	Add    []AddElement `json:"add"`
	Remove []string     `json:"remove"`
}

// A group member has a weight associated with them. This may all be equal, or may have
// meaning in the app that makes use of the group (eg. voting power)
type AddElement struct {
	Addr   string `json:"addr"`
	Weight int64  `json:"weight"`
}

type ExecuteMsgVote struct {
	SlashingRequestID string `json:"slashing_request_id"`
	Vote              Vote   `json:"vote"`
}

type QueryMsg struct {
	Threshold                   *ThresholdClass              `json:"threshold,omitempty"`
	Proposal                    *Proposal                    `json:"proposal,omitempty"`
	ProposalBySlashingRequestID *ProposalBySlashingRequestID `json:"proposal_by_slashing_request_id,omitempty"`
	ListProposals               *ListProposals               `json:"list_proposals,omitempty"`
	Vote                        *QueryMsgVote                `json:"vote,omitempty"`
	VoteBySlashingRequestID     *VoteBySlashingRequestID     `json:"vote_by_slashing_request_id,omitempty"`
	ListVotes                   *ListVotes                   `json:"list_votes,omitempty"`
	Voter                       *Voter                       `json:"voter,omitempty"`
	ListVoters                  *ListVoters                  `json:"list_voters,omitempty"`
}

type ListProposals struct {
	Limit      *int64 `json:"limit"`
	StartAfter *int64 `json:"start_after"`
}

type ListVoters struct {
	Limit      *int64  `json:"limit"`
	StartAfter *string `json:"start_after"`
}

type ListVotes struct {
	Limit      *int64  `json:"limit"`
	ProposalID int64   `json:"proposal_id"`
	StartAfter *string `json:"start_after"`
}

type Proposal struct {
	ProposalID int64 `json:"proposal_id"`
}

type ProposalBySlashingRequestID struct {
	SlashingRequestID string `json:"slashing_request_id"`
}

type ThresholdClass struct {
	Height *int64 `json:"height"`
}

type QueryMsgVote struct {
	ProposalID int64  `json:"proposal_id"`
	Voter      string `json:"voter"`
}

type VoteBySlashingRequestID struct {
	SlashingRequestID string `json:"slashing_request_id"`
	Voter             string `json:"voter"`
}

type Voter struct {
	Address string `json:"address"`
	Height  *int64 `json:"height"`
}

type ProposalListResponseForEmpty struct {
	Proposals []ProposalElement `json:"proposals"`
}

// Note, if you are storing custom messages in the proposal, the querier needs to know what
// possible custom message types those are in order to parse the response
type ProposalElement struct {
	Deposit     *ProposalDepositInfo        `json:"deposit"`
	Description string                      `json:"description"`
	Expires     ProposalExpiration          `json:"expires"`
	ID          int64                       `json:"id"`
	Msgs        []ProposalCosmosMsgForEmpty `json:"msgs"`
	Proposer    string                      `json:"proposer"`
	Status      Status                      `json:"status"`
	// This is the threshold that is applied to this proposal. Both the rules of the voting
	// contract, as well as the total_weight of the voting group may have changed since this
	// time. That means that the generic `Threshold{}` query does not provide valid information
	// for existing proposals.
	Threshold ProposalThresholdResponse `json:"threshold"`
	Title     string                    `json:"title"`
}

// Information about the deposit required to create a proposal.
type ProposalDepositInfo struct {
	// The number tokens required for payment.
	Amount string `json:"amount"`
	// The denom of the deposit payment.
	Denom PurpleDenom `json:"denom"`
	// Should failed proposals have their deposits refunded?
	RefundFailedProposals bool `json:"refund_failed_proposals"`
}

// The denom of the deposit payment.
type PurpleDenom struct {
	Native *string `json:"native,omitempty"`
	Cw20   *string `json:"cw20,omitempty"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type ProposalExpiration struct {
	AtHeight *int64       `json:"at_height,omitempty"`
	AtTime   *string      `json:"at_time,omitempty"`
	Never    *FluffyNever `json:"never,omitempty"`
}

type FluffyNever struct {
}

// `CosmosMsg::Any` is the replaces the "stargate message" – a message wrapped in a
// [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any) that is supported by
// the chain. It behaves the same as `CosmosMsg::Stargate` but has a better name and
// slightly improved syntax.
//
// This is feature-gated at compile time with `cosmwasm_2_0` because a chain running
// CosmWasm < 2.0 cannot process this.
type ProposalCosmosMsgForEmpty struct {
	Bank   *PurpleBankMsg `json:"bank,omitempty"`
	Custom *PurpleEmpty   `json:"custom,omitempty"`
	Any    *PurpleAnyMsg  `json:"any,omitempty"`
	WASM   *PurpleWASMMsg `json:"wasm,omitempty"`
}

// A message encoded the same way as a protobuf
// [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
// This is the same structure as messages in `TxBody` from
// [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
type PurpleAnyMsg struct {
	TypeURL string `json:"type_url"`
	Value   string `json:"value"`
}

// The message types of the bank module.
//
// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
//
// Sends native tokens from the contract to the given address.
//
// This is translated to a
// [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
// `from_address` is automatically filled with the current contract's address.
//
// This will burn the given coins from the contract's account. There is no Cosmos SDK
// message that performs this, but it can be done by calling the bank keeper. Important if a
// contract controls significant token supply that must be retired.
type PurpleBankMsg struct {
	Send *PurpleSend `json:"send,omitempty"`
	Burn *PurpleBurn `json:"burn,omitempty"`
}

type PurpleBurn struct {
	Amount []PurpleCoin `json:"amount"`
}

type PurpleCoin struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}

type PurpleSend struct {
	Amount    []PurpleCoin `json:"amount"`
	ToAddress string       `json:"to_address"`
}

// An empty struct that serves as a placeholder in different places, such as contracts that
// don't set a custom message.
//
// It is designed to be expressible in correct JSON and JSON Schema but contains no
// meaningful data. Previously we used enums without cases, but those cannot represented as
// valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
type PurpleEmpty struct {
}

// The message types of the wasm module.
//
// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
//
// Dispatches a call to another contract at a known address (with known ABI).
//
// This is translated to a
// [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
// `sender` is automatically filled with the current contract's address.
//
// Instantiates a new contracts from previously uploaded Wasm code.
//
// The contract address is non-predictable. But it is guaranteed that when emitting the same
// Instantiate message multiple times, multiple instances on different addresses will be
// generated. See also Instantiate2.
//
// This is translated to a
// [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
// `sender` is automatically filled with the current contract's address.
//
// Instantiates a new contracts from previously uploaded Wasm code using a predictable
// address derivation algorithm implemented in [`cosmwasm_std::instantiate2_address`].
//
// This is translated to a
// [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
// `sender` is automatically filled with the current contract's address. `fix_msg` is
// automatically set to false.
//
// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
// customize behavior.
//
// Only the contract admin (as defined in wasmd), if any, is able to make this call.
//
// This is translated to a
// [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
// `sender` is automatically filled with the current contract's address.
//
// Sets a new admin (for migrate) on the given contract. Fails if this contract is not
// currently admin of the target contract.
//
// Clears the admin on the given contract, so no more migration possible. Fails if this
// contract is not currently admin of the target contract.
type PurpleWASMMsg struct {
	Execute      *PurpleExecute      `json:"execute,omitempty"`
	Instantiate  *PurpleInstantiate  `json:"instantiate,omitempty"`
	Instantiate2 *PurpleInstantiate2 `json:"instantiate2,omitempty"`
	Migrate      *PurpleMigrate      `json:"migrate,omitempty"`
	UpdateAdmin  *PurpleUpdateAdmin  `json:"update_admin,omitempty"`
	ClearAdmin   *PurpleClearAdmin   `json:"clear_admin,omitempty"`
}

type PurpleClearAdmin struct {
	ContractAddr string `json:"contract_addr"`
}

type PurpleExecute struct {
	ContractAddr string       `json:"contract_addr"`
	Funds        []PurpleCoin `json:"funds"`
	// msg is the json-encoded ExecuteMsg struct (as raw Binary)
	Msg string `json:"msg"`
}

type PurpleInstantiate struct {
	Admin  *string      `json:"admin"`
	CodeID int64        `json:"code_id"`
	Funds  []PurpleCoin `json:"funds"`
	// A human-readable label for the contract.
	//
	// Valid values should: - not be empty - not be bigger than 128 bytes (or some
	// chain-specific limit) - not start / end with whitespace
	Label string `json:"label"`
	// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
	Msg string `json:"msg"`
}

type PurpleInstantiate2 struct {
	Admin  *string      `json:"admin"`
	CodeID int64        `json:"code_id"`
	Funds  []PurpleCoin `json:"funds"`
	// A human-readable label for the contract.
	//
	// Valid values should: - not be empty - not be bigger than 128 bytes (or some
	// chain-specific limit) - not start / end with whitespace
	Label string `json:"label"`
	// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
	Msg  string `json:"msg"`
	Salt string `json:"salt"`
}

type PurpleMigrate struct {
	ContractAddr string `json:"contract_addr"`
	// msg is the json-encoded MigrateMsg struct that will be passed to the new code
	Msg string `json:"msg"`
	// the code_id of the new logic to place in the given contract
	NewCodeID int64 `json:"new_code_id"`
}

type PurpleUpdateAdmin struct {
	Admin        string `json:"admin"`
	ContractAddr string `json:"contract_addr"`
}

// This is the threshold that is applied to this proposal. Both the rules of the voting
// contract, as well as the total_weight of the voting group may have changed since this
// time. That means that the generic `Threshold{}` query does not provide valid information
// for existing proposals.
//
// This defines the different ways tallies can happen. Every contract should support a
// subset of these, ideally all.
//
// The total_weight used for calculating success as well as the weights of each individual
// voter used in tallying should be snapshotted at the beginning of the block at which the
// proposal starts (this is likely the responsibility of a correct cw4 implementation).
//
// Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
// no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
//
// This is the simplest format and usually suitable for small multisigs of trusted parties,
// like 3 of 5. (weight: 3, total_weight: 5)
//
// A proposal of this type can pass early as soon as the needed weight of yes votes has been
// cast.
//
// Declares a percentage of the total weight that must cast Yes votes, in order for a
// proposal to pass. The passing weight is computed over the total weight minus the weight
// of the abstained votes.
//
// This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
// small set of voters, and participation is required. It is understood that if the voting
// set (group) changes between different proposals that refer to the same group, each
// proposal will work with a different set of voter weights (the ones snapshotted at
// proposal creation), and the passing weight for each proposal will be computed based on
// the absolute percentage, times the total weights of the members at the time of each
// proposal creation.
//
// Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
// 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
// but the `total_weight` of the group has increased to 9. That proposal will then
// automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
// with `AbsoluteCount`.
//
// In addition to a `threshold`, declares a `quorum` of the total votes that must
// participate in the election in order for the vote to be considered at all. Within the
// votes that were cast, it requires `threshold` votes in favor. That is calculated by
// ignoring the Abstain votes (they count towards `quorum`, but do not influence
// `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
// `threshold` to consider if the proposal was passed.
//
// It is rather difficult for a proposal of this type to pass early. That can only happen if
// the required quorum has been already met, and there are already enough Yes votes for the
// proposal to pass.
//
// 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
// cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
// 50% no). Once the voting period has passed with no additional votes, that same proposal
// would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
// we ignore abstain votes).
//
// This type is more common in general elections, where participation is often expected to
// be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
// percentages to pass, independently of if there was high participation in the election or
// not.
type ProposalThresholdResponse struct {
	AbsoluteCount      *PurpleAbsoluteCount      `json:"absolute_count,omitempty"`
	AbsolutePercentage *PurpleAbsolutePercentage `json:"absolute_percentage,omitempty"`
	ThresholdQuorum    *PurpleThresholdQuorum    `json:"threshold_quorum,omitempty"`
}

type PurpleAbsoluteCount struct {
	TotalWeight int64 `json:"total_weight"`
	Weight      int64 `json:"weight"`
}

type PurpleAbsolutePercentage struct {
	Percentage  string `json:"percentage"`
	TotalWeight int64  `json:"total_weight"`
}

type PurpleThresholdQuorum struct {
	Quorum      string `json:"quorum"`
	Threshold   string `json:"threshold"`
	TotalWeight int64  `json:"total_weight"`
}

type VoterListResponse struct {
	Voters []VoterDetail `json:"voters"`
}

type VoterDetail struct {
	Addr   string `json:"addr"`
	Weight int64  `json:"weight"`
}

type VoteListResponse struct {
	Votes []VoteElement `json:"votes"`
}

// Returns the vote (opinion as well as weight counted) as well as the address of the voter
// who submitted it
type VoteElement struct {
	ProposalID int64  `json:"proposal_id"`
	Vote       Vote   `json:"vote"`
	Voter      string `json:"voter"`
	Weight     int64  `json:"weight"`
}

// Note, if you are storing custom messages in the proposal, the querier needs to know what
// possible custom message types those are in order to parse the response
type ProposalResponseForEmpty struct {
	Deposit     *ProposalResponseForEmptyDepositInfo        `json:"deposit"`
	Description string                                      `json:"description"`
	Expires     ProposalResponseForEmptyExpiration          `json:"expires"`
	ID          int64                                       `json:"id"`
	Msgs        []ProposalResponseForEmptyCosmosMsgForEmpty `json:"msgs"`
	Proposer    string                                      `json:"proposer"`
	Status      Status                                      `json:"status"`
	// This is the threshold that is applied to this proposal. Both the rules of the voting
	// contract, as well as the total_weight of the voting group may have changed since this
	// time. That means that the generic `Threshold{}` query does not provide valid information
	// for existing proposals.
	Threshold ProposalResponseForEmptyThresholdResponse `json:"threshold"`
	Title     string                                    `json:"title"`
}

// Information about the deposit required to create a proposal.
type ProposalResponseForEmptyDepositInfo struct {
	// The number tokens required for payment.
	Amount string `json:"amount"`
	// The denom of the deposit payment.
	Denom FluffyDenom `json:"denom"`
	// Should failed proposals have their deposits refunded?
	RefundFailedProposals bool `json:"refund_failed_proposals"`
}

// The denom of the deposit payment.
type FluffyDenom struct {
	Native *string `json:"native,omitempty"`
	Cw20   *string `json:"cw20,omitempty"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type ProposalResponseForEmptyExpiration struct {
	AtHeight *int64          `json:"at_height,omitempty"`
	AtTime   *string         `json:"at_time,omitempty"`
	Never    *TentacledNever `json:"never,omitempty"`
}

type TentacledNever struct {
}

// `CosmosMsg::Any` is the replaces the "stargate message" – a message wrapped in a
// [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any) that is supported by
// the chain. It behaves the same as `CosmosMsg::Stargate` but has a better name and
// slightly improved syntax.
//
// This is feature-gated at compile time with `cosmwasm_2_0` because a chain running
// CosmWasm < 2.0 cannot process this.
type ProposalResponseForEmptyCosmosMsgForEmpty struct {
	Bank   *FluffyBankMsg `json:"bank,omitempty"`
	Custom *FluffyEmpty   `json:"custom,omitempty"`
	Any    *FluffyAnyMsg  `json:"any,omitempty"`
	WASM   *FluffyWASMMsg `json:"wasm,omitempty"`
}

// A message encoded the same way as a protobuf
// [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
// This is the same structure as messages in `TxBody` from
// [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
type FluffyAnyMsg struct {
	TypeURL string `json:"type_url"`
	Value   string `json:"value"`
}

// The message types of the bank module.
//
// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
//
// Sends native tokens from the contract to the given address.
//
// This is translated to a
// [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
// `from_address` is automatically filled with the current contract's address.
//
// This will burn the given coins from the contract's account. There is no Cosmos SDK
// message that performs this, but it can be done by calling the bank keeper. Important if a
// contract controls significant token supply that must be retired.
type FluffyBankMsg struct {
	Send *FluffySend `json:"send,omitempty"`
	Burn *FluffyBurn `json:"burn,omitempty"`
}

type FluffyBurn struct {
	Amount []FluffyCoin `json:"amount"`
}

type FluffyCoin struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}

type FluffySend struct {
	Amount    []FluffyCoin `json:"amount"`
	ToAddress string       `json:"to_address"`
}

// An empty struct that serves as a placeholder in different places, such as contracts that
// don't set a custom message.
//
// It is designed to be expressible in correct JSON and JSON Schema but contains no
// meaningful data. Previously we used enums without cases, but those cannot represented as
// valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
type FluffyEmpty struct {
}

// The message types of the wasm module.
//
// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
//
// Dispatches a call to another contract at a known address (with known ABI).
//
// This is translated to a
// [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
// `sender` is automatically filled with the current contract's address.
//
// Instantiates a new contracts from previously uploaded Wasm code.
//
// The contract address is non-predictable. But it is guaranteed that when emitting the same
// Instantiate message multiple times, multiple instances on different addresses will be
// generated. See also Instantiate2.
//
// This is translated to a
// [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
// `sender` is automatically filled with the current contract's address.
//
// Instantiates a new contracts from previously uploaded Wasm code using a predictable
// address derivation algorithm implemented in [`cosmwasm_std::instantiate2_address`].
//
// This is translated to a
// [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
// `sender` is automatically filled with the current contract's address. `fix_msg` is
// automatically set to false.
//
// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
// customize behavior.
//
// Only the contract admin (as defined in wasmd), if any, is able to make this call.
//
// This is translated to a
// [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
// `sender` is automatically filled with the current contract's address.
//
// Sets a new admin (for migrate) on the given contract. Fails if this contract is not
// currently admin of the target contract.
//
// Clears the admin on the given contract, so no more migration possible. Fails if this
// contract is not currently admin of the target contract.
type FluffyWASMMsg struct {
	Execute      *FluffyExecute      `json:"execute,omitempty"`
	Instantiate  *FluffyInstantiate  `json:"instantiate,omitempty"`
	Instantiate2 *FluffyInstantiate2 `json:"instantiate2,omitempty"`
	Migrate      *FluffyMigrate      `json:"migrate,omitempty"`
	UpdateAdmin  *FluffyUpdateAdmin  `json:"update_admin,omitempty"`
	ClearAdmin   *FluffyClearAdmin   `json:"clear_admin,omitempty"`
}

type FluffyClearAdmin struct {
	ContractAddr string `json:"contract_addr"`
}

type FluffyExecute struct {
	ContractAddr string       `json:"contract_addr"`
	Funds        []FluffyCoin `json:"funds"`
	// msg is the json-encoded ExecuteMsg struct (as raw Binary)
	Msg string `json:"msg"`
}

type FluffyInstantiate struct {
	Admin  *string      `json:"admin"`
	CodeID int64        `json:"code_id"`
	Funds  []FluffyCoin `json:"funds"`
	// A human-readable label for the contract.
	//
	// Valid values should: - not be empty - not be bigger than 128 bytes (or some
	// chain-specific limit) - not start / end with whitespace
	Label string `json:"label"`
	// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
	Msg string `json:"msg"`
}

type FluffyInstantiate2 struct {
	Admin  *string      `json:"admin"`
	CodeID int64        `json:"code_id"`
	Funds  []FluffyCoin `json:"funds"`
	// A human-readable label for the contract.
	//
	// Valid values should: - not be empty - not be bigger than 128 bytes (or some
	// chain-specific limit) - not start / end with whitespace
	Label string `json:"label"`
	// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
	Msg  string `json:"msg"`
	Salt string `json:"salt"`
}

type FluffyMigrate struct {
	ContractAddr string `json:"contract_addr"`
	// msg is the json-encoded MigrateMsg struct that will be passed to the new code
	Msg string `json:"msg"`
	// the code_id of the new logic to place in the given contract
	NewCodeID int64 `json:"new_code_id"`
}

type FluffyUpdateAdmin struct {
	Admin        string `json:"admin"`
	ContractAddr string `json:"contract_addr"`
}

// This is the threshold that is applied to this proposal. Both the rules of the voting
// contract, as well as the total_weight of the voting group may have changed since this
// time. That means that the generic `Threshold{}` query does not provide valid information
// for existing proposals.
//
// This defines the different ways tallies can happen. Every contract should support a
// subset of these, ideally all.
//
// The total_weight used for calculating success as well as the weights of each individual
// voter used in tallying should be snapshotted at the beginning of the block at which the
// proposal starts (this is likely the responsibility of a correct cw4 implementation).
//
// Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
// no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
//
// This is the simplest format and usually suitable for small multisigs of trusted parties,
// like 3 of 5. (weight: 3, total_weight: 5)
//
// A proposal of this type can pass early as soon as the needed weight of yes votes has been
// cast.
//
// Declares a percentage of the total weight that must cast Yes votes, in order for a
// proposal to pass. The passing weight is computed over the total weight minus the weight
// of the abstained votes.
//
// This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
// small set of voters, and participation is required. It is understood that if the voting
// set (group) changes between different proposals that refer to the same group, each
// proposal will work with a different set of voter weights (the ones snapshotted at
// proposal creation), and the passing weight for each proposal will be computed based on
// the absolute percentage, times the total weights of the members at the time of each
// proposal creation.
//
// Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
// 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
// but the `total_weight` of the group has increased to 9. That proposal will then
// automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
// with `AbsoluteCount`.
//
// In addition to a `threshold`, declares a `quorum` of the total votes that must
// participate in the election in order for the vote to be considered at all. Within the
// votes that were cast, it requires `threshold` votes in favor. That is calculated by
// ignoring the Abstain votes (they count towards `quorum`, but do not influence
// `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
// `threshold` to consider if the proposal was passed.
//
// It is rather difficult for a proposal of this type to pass early. That can only happen if
// the required quorum has been already met, and there are already enough Yes votes for the
// proposal to pass.
//
// 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
// cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
// 50% no). Once the voting period has passed with no additional votes, that same proposal
// would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
// we ignore abstain votes).
//
// This type is more common in general elections, where participation is often expected to
// be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
// percentages to pass, independently of if there was high participation in the election or
// not.
type ProposalResponseForEmptyThresholdResponse struct {
	AbsoluteCount      *FluffyAbsoluteCount      `json:"absolute_count,omitempty"`
	AbsolutePercentage *FluffyAbsolutePercentage `json:"absolute_percentage,omitempty"`
	ThresholdQuorum    *FluffyThresholdQuorum    `json:"threshold_quorum,omitempty"`
}

type FluffyAbsoluteCount struct {
	TotalWeight int64 `json:"total_weight"`
	Weight      int64 `json:"weight"`
}

type FluffyAbsolutePercentage struct {
	Percentage  string `json:"percentage"`
	TotalWeight int64  `json:"total_weight"`
}

type FluffyThresholdQuorum struct {
	Quorum      string `json:"quorum"`
	Threshold   string `json:"threshold"`
	TotalWeight int64  `json:"total_weight"`
}

// This defines the different ways tallies can happen. Every contract should support a
// subset of these, ideally all.
//
// The total_weight used for calculating success as well as the weights of each individual
// voter used in tallying should be snapshotted at the beginning of the block at which the
// proposal starts (this is likely the responsibility of a correct cw4 implementation).
//
// Declares that a fixed weight of yes votes is needed to pass. It does not matter how many
// no votes are cast, or how many do not vote, as long as `weight` yes votes are cast.
//
// This is the simplest format and usually suitable for small multisigs of trusted parties,
// like 3 of 5. (weight: 3, total_weight: 5)
//
// A proposal of this type can pass early as soon as the needed weight of yes votes has been
// cast.
//
// Declares a percentage of the total weight that must cast Yes votes, in order for a
// proposal to pass. The passing weight is computed over the total weight minus the weight
// of the abstained votes.
//
// This is useful for similar circumstances as `AbsoluteCount`, where we have a relatively
// small set of voters, and participation is required. It is understood that if the voting
// set (group) changes between different proposals that refer to the same group, each
// proposal will work with a different set of voter weights (the ones snapshotted at
// proposal creation), and the passing weight for each proposal will be computed based on
// the absolute percentage, times the total weights of the members at the time of each
// proposal creation.
//
// Example: we set `percentage` to 51%. Proposal 1 starts when there is a `total_weight` of
// 5. This will require 3 weight of Yes votes in order to pass. Later, the Proposal 2 starts
// but the `total_weight` of the group has increased to 9. That proposal will then
// automatically require 5 Yes of 9 to pass, rather than 3 yes of 9 as would be the case
// with `AbsoluteCount`.
//
// In addition to a `threshold`, declares a `quorum` of the total votes that must
// participate in the election in order for the vote to be considered at all. Within the
// votes that were cast, it requires `threshold` votes in favor. That is calculated by
// ignoring the Abstain votes (they count towards `quorum`, but do not influence
// `threshold`). That is, we calculate `Yes / (Yes + No + Veto)` and compare it with
// `threshold` to consider if the proposal was passed.
//
// It is rather difficult for a proposal of this type to pass early. That can only happen if
// the required quorum has been already met, and there are already enough Yes votes for the
// proposal to pass.
//
// 30% Yes votes, 10% No votes, and 20% Abstain would pass early if quorum <= 60% (who has
// cast votes) and if the threshold is <= 37.5% (the remaining 40% voting no => 30% yes +
// 50% no). Once the voting period has passed with no additional votes, that same proposal
// would be considered successful if quorum <= 60% and threshold <= 75% (percent in favor if
// we ignore abstain votes).
//
// This type is more common in general elections, where participation is often expected to
// be low, and `AbsolutePercentage` would either be too high to pass anything, or allow low
// percentages to pass, independently of if there was high participation in the election or
// not.
type ThresholdResponse struct {
	AbsoluteCount      *TentacledAbsoluteCount      `json:"absolute_count,omitempty"`
	AbsolutePercentage *TentacledAbsolutePercentage `json:"absolute_percentage,omitempty"`
	ThresholdQuorum    *TentacledThresholdQuorum    `json:"threshold_quorum,omitempty"`
}

type TentacledAbsoluteCount struct {
	TotalWeight int64 `json:"total_weight"`
	Weight      int64 `json:"weight"`
}

type TentacledAbsolutePercentage struct {
	Percentage  string `json:"percentage"`
	TotalWeight int64  `json:"total_weight"`
}

type TentacledThresholdQuorum struct {
	Quorum      string `json:"quorum"`
	Threshold   string `json:"threshold"`
	TotalWeight int64  `json:"total_weight"`
}

type VoteResponse struct {
	Vote *VoteInfo `json:"vote"`
}

// Returns the vote (opinion as well as weight counted) as well as the address of the voter
// who submitted it
type VoteInfo struct {
	ProposalID int64  `json:"proposal_id"`
	Vote       Vote   `json:"vote"`
	Voter      string `json:"voter"`
	Weight     int64  `json:"weight"`
}

type VoterResponse struct {
	Weight *int64 `json:"weight"`
}

// Marks support for the proposal.
//
// Marks opposition to the proposal.
//
// Marks participation but does not count towards the ratio of support / opposed
//
// Veto is generally to be treated as a No vote. Some implementations may allow certain
// voters to be able to Veto, or them to be counted stronger than No in some way.
type Vote string

const (
	Abstain Vote = "abstain"
	No      Vote = "no"
	Veto    Vote = "veto"
	Yes     Vote = "yes"
)

// proposal was created, but voting has not yet begun for whatever reason
//
// you can vote on this
//
// voting is over and it did not pass
//
// voting is over and it did pass, but has not yet executed
//
// voting is over it passed, and the proposal was executed
type Status string

const (
	Executed Status = "executed"
	Open     Status = "open"
	Passed   Status = "passed"
	Pending  Status = "pending"
	Rejected Status = "rejected"
)
