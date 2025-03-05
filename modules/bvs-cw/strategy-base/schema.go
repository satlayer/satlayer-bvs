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
//    sharesResponse, err := UnmarshalSharesResponse(bytes)
//    bytes, err = sharesResponse.Marshal()
//
//    sharesToUnderlyingResponse, err := UnmarshalSharesToUnderlyingResponse(bytes)
//    bytes, err = sharesToUnderlyingResponse.Marshal()
//
//    strategyManagerResponse, err := UnmarshalStrategyManagerResponse(bytes)
//    bytes, err = strategyManagerResponse.Marshal()
//
//    totalSharesResponse, err := UnmarshalTotalSharesResponse(bytes)
//    bytes, err = totalSharesResponse.Marshal()
//
//    underlyingResponse, err := UnmarshalUnderlyingResponse(bytes)
//    bytes, err = underlyingResponse.Marshal()
//
//    underlyingToSharesResponse, err := UnmarshalUnderlyingToSharesResponse(bytes)
//    bytes, err = underlyingToSharesResponse.Marshal()
//
//    underlyingTokenResponse, err := UnmarshalUnderlyingTokenResponse(bytes)
//    bytes, err = underlyingTokenResponse.Marshal()

package strategybase

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

type SharesResponse string

func UnmarshalSharesResponse(data []byte) (SharesResponse, error) {
	var r SharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type SharesToUnderlyingResponse string

func UnmarshalSharesToUnderlyingResponse(data []byte) (SharesToUnderlyingResponse, error) {
	var r SharesToUnderlyingResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SharesToUnderlyingResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type StrategyManagerResponse string

func UnmarshalStrategyManagerResponse(data []byte) (StrategyManagerResponse, error) {
	var r StrategyManagerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyManagerResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type TotalSharesResponse string

func UnmarshalTotalSharesResponse(data []byte) (TotalSharesResponse, error) {
	var r TotalSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TotalSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type UnderlyingResponse string

func UnmarshalUnderlyingResponse(data []byte) (UnderlyingResponse, error) {
	var r UnderlyingResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type UnderlyingToSharesResponse string

func UnmarshalUnderlyingToSharesResponse(data []byte) (UnderlyingToSharesResponse, error) {
	var r UnderlyingToSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingToSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type UnderlyingTokenResponse string

func UnmarshalUnderlyingTokenResponse(data []byte) (UnderlyingTokenResponse, error) {
	var r UnderlyingTokenResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingTokenResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Owner           string `json:"owner"`
	Registry        string `json:"registry"`
	StrategyManager string `json:"strategy_manager"`
	UnderlyingToken string `json:"underlying_token"`
}

type ExecuteMsg struct {
	Deposit           *Deposit           `json:"deposit,omitempty"`
	Withdraw          *Withdraw          `json:"withdraw,omitempty"`
	TransferOwnership *TransferOwnership `json:"transfer_ownership,omitempty"`
}

type Deposit struct {
	Amount string `json:"amount"`
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type Withdraw struct {
	Recipient string `json:"recipient"`
	Shares    string `json:"shares"`
}

type QueryMsg struct {
	Shares             *Shares             `json:"shares,omitempty"`
	Underlying         *Underlying         `json:"underlying,omitempty"`
	SharesToUnderlying *SharesToUnderlying `json:"shares_to_underlying,omitempty"`
	UnderlyingToShares *UnderlyingToShares `json:"underlying_to_shares,omitempty"`
	StrategyManager    *StrategyManager    `json:"strategy_manager,omitempty"`
	UnderlyingToken    *UnderlyingToken    `json:"underlying_token,omitempty"`
	TotalShares        *TotalShares        `json:"total_shares,omitempty"`
}

type Shares struct {
	Staker string `json:"staker"`
}

type SharesToUnderlying struct {
	Shares string `json:"shares"`
}

type StrategyManager struct {
}

type TotalShares struct {
}

type Underlying struct {
	Staker string `json:"staker"`
}

type UnderlyingToShares struct {
	Amount string `json:"amount"`
}

type UnderlyingToken struct {
}
