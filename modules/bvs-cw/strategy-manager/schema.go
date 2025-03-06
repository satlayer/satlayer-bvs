// This file was generated from JSON Schema using quicktype, do not modify it directly.
// To parse and unparse this JSON data, add this code to your project and do:
//
//    instantiateMsg, err := UnmarshalInstantiateMsg(bytes)
//    bytes, err = instantiateMsg.Marshal()
//
//    executeMsg, err := UnmarshalExecuteMsg(bytes)
//    bytes, err = executeMsg.Marshal()
//
//    queryMsg, err := UnmarshalQueryMsg(bytes)
//    bytes, err = queryMsg.Marshal()
//
//    isStrategyWhitelistedResponse, err := UnmarshalIsStrategyWhitelistedResponse(bytes)
//    bytes, err = isStrategyWhitelistedResponse.Marshal()
//
//    stakerDepositListResponse, err := UnmarshalStakerDepositListResponse(bytes)
//    bytes, err = stakerDepositListResponse.Marshal()
//
//    stakerStrategyListResponse, err := UnmarshalStakerStrategyListResponse(bytes)
//    bytes, err = stakerStrategyListResponse.Marshal()
//
//    stakerStrategySharesResponse, err := UnmarshalStakerStrategySharesResponse(bytes)
//    bytes, err = stakerStrategySharesResponse.Marshal()

package strategymanager

import "encoding/json"

func UnmarshalInstantiateMsg(data []byte) (InstantiateMsg, error) {
	var r InstantiateMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *InstantiateMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalExecuteMsg(data []byte) (ExecuteMsg, error) {
	var r ExecuteMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ExecuteMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalQueryMsg(data []byte) (QueryMsg, error) {
	var r QueryMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *QueryMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type IsStrategyWhitelistedResponse bool

func UnmarshalIsStrategyWhitelistedResponse(data []byte) (IsStrategyWhitelistedResponse, error) {
	var r IsStrategyWhitelistedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsStrategyWhitelistedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type StakerDepositListResponse []StrategyShare

func UnmarshalStakerDepositListResponse(data []byte) (StakerDepositListResponse, error) {
	var r StakerDepositListResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerDepositListResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type StakerStrategyListResponse []string

func UnmarshalStakerStrategyListResponse(data []byte) (StakerStrategyListResponse, error) {
	var r StakerStrategyListResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategyListResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type StakerStrategySharesResponse string

func UnmarshalStakerStrategySharesResponse(data []byte) (StakerStrategySharesResponse, error) {
	var r StakerStrategySharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategySharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Owner    string `json:"owner"`
	Registry string `json:"registry"`
}

type ExecuteMsg struct {
	DepositIntoStrategy    *DepositIntoStrategy    `json:"deposit_into_strategy,omitempty"`
	WithdrawSharesAsTokens *WithdrawSharesAsTokens `json:"withdraw_shares_as_tokens,omitempty"`
	AddShares              *AddShares              `json:"add_shares,omitempty"`
	RemoveShares           *RemoveShares           `json:"remove_shares,omitempty"`
	TransferOwnership      *TransferOwnership      `json:"transfer_ownership,omitempty"`
	SetRouting             *SetRouting             `json:"set_routing,omitempty"`
	AddStrategy            *AddStrategy            `json:"add_strategy,omitempty"`
	UpdateStrategy         *UpdateStrategy         `json:"update_strategy,omitempty"`
}

type AddShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type AddStrategy struct {
	Strategy    string `json:"strategy"`
	Whitelisted bool   `json:"whitelisted"`
}

type DepositIntoStrategy struct {
	Amount   string `json:"amount"`
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
}

type RemoveShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type SetRouting struct {
	DelegationManager string `json:"delegation_manager"`
	SlashManager      string `json:"slash_manager"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type UpdateStrategy struct {
	Strategy    string `json:"strategy"`
	Whitelisted bool   `json:"whitelisted"`
}

type WithdrawSharesAsTokens struct {
	Recipient string `json:"recipient"`
	Shares    string `json:"shares"`
	Strategy  string `json:"strategy"`
}

type QueryMsg struct {
	StakerDepositList     *StakerDepositList    `json:"staker_deposit_list,omitempty"`
	StakerStrategyShares  *StakerStrategyShares `json:"staker_strategy_shares,omitempty"`
	StakerStrategyList    *StakerStrategyList   `json:"staker_strategy_list,omitempty"`
	IsStrategyWhitelisted *string               `json:"is_strategy_whitelisted,omitempty"`
}

type StakerDepositList struct {
	Staker string `json:"staker"`
}

type StakerStrategyList struct {
	Staker string `json:"staker"`
}

type StakerStrategyShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type StrategyShare struct {
	Shares   string `json:"shares"`
	Strategy string `json:"strategy"`
}
