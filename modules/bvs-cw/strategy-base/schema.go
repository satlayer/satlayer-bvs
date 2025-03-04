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
//    strategyManagerResponse, err := UnmarshalStrategyManagerResponse(bytes)
//    bytes, err = strategyManagerResponse.Marshal()
//
//    strategyState, err := UnmarshalStrategyState(bytes)
//    bytes, err = strategyState.Marshal()
//
//    totalSharesResponse, err := UnmarshalTotalSharesResponse(bytes)
//    bytes, err = totalSharesResponse.Marshal()
//
//    underlyingTokenResponse, err := UnmarshalUnderlyingTokenResponse(bytes)
//    bytes, err = underlyingTokenResponse.Marshal()
//
//    sharesToUnderlyingResponse, err := UnmarshalSharesToUnderlyingResponse(bytes)
//    bytes, err = sharesToUnderlyingResponse.Marshal()
//
//    underlyingToShareResponse, err := UnmarshalUnderlyingToShareResponse(bytes)
//    bytes, err = underlyingToShareResponse.Marshal()
//
//    underlyingToSharesResponse, err := UnmarshalUnderlyingToSharesResponse(bytes)
//    bytes, err = underlyingToSharesResponse.Marshal()
//
//    userUnderlyingResponse, err := UnmarshalUserUnderlyingResponse(bytes)
//    bytes, err = userUnderlyingResponse.Marshal()

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

func UnmarshalSharesResponse(data []byte) (SharesResponse, error) {
	var r SharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStrategyManagerResponse(data []byte) (StrategyManagerResponse, error) {
	var r StrategyManagerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyManagerResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStrategyState(data []byte) (StrategyState, error) {
	var r StrategyState
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyState) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalTotalSharesResponse(data []byte) (TotalSharesResponse, error) {
	var r TotalSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TotalSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalUnderlyingTokenResponse(data []byte) (UnderlyingTokenResponse, error) {
	var r UnderlyingTokenResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingTokenResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalSharesToUnderlyingResponse(data []byte) (SharesToUnderlyingResponse, error) {
	var r SharesToUnderlyingResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *SharesToUnderlyingResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalUnderlyingToShareResponse(data []byte) (UnderlyingToShareResponse, error) {
	var r UnderlyingToShareResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingToShareResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalUnderlyingToSharesResponse(data []byte) (UnderlyingToSharesResponse, error) {
	var r UnderlyingToSharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UnderlyingToSharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalUserUnderlyingResponse(data []byte) (UserUnderlyingResponse, error) {
	var r UserUnderlyingResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *UserUnderlyingResponse) Marshal() ([]byte, error) {
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
	GetShares              *GetShares              `json:"get_shares,omitempty"`
	SharesToUnderlyingView *SharesToUnderlyingView `json:"shares_to_underlying_view,omitempty"`
	UnderlyingToShareView  *UnderlyingToShareView  `json:"underlying_to_share_view,omitempty"`
	UserUnderlyingView     *UserUnderlyingView     `json:"user_underlying_view,omitempty"`
	GetStrategyManager     *GetStrategyManager     `json:"get_strategy_manager,omitempty"`
	GetUnderlyingToken     *GetUnderlyingToken     `json:"get_underlying_token,omitempty"`
	GetTotalShares         *GetTotalShares         `json:"get_total_shares,omitempty"`
	UnderlyingToShares     *UnderlyingToShares     `json:"underlying_to_shares,omitempty"`
	GetStrategyState       *GetStrategyState       `json:"get_strategy_state,omitempty"`
}

type GetShares struct {
	Staker string `json:"staker"`
}

type GetStrategyManager struct {
}

type GetStrategyState struct {
}

type GetTotalShares struct {
}

type GetUnderlyingToken struct {
}

type SharesToUnderlyingView struct {
	AmountShares string `json:"amount_shares"`
}

type UnderlyingToShareView struct {
	Amount string `json:"amount"`
}

type UnderlyingToShares struct {
	AmountUnderlying string `json:"amount_underlying"`
}

type UserUnderlyingView struct {
	User string `json:"user"`
}

type SharesResponse struct {
	TotalShares string `json:"total_shares"`
}

type StrategyManagerResponse struct {
	StrategyManagerAddr string `json:"strategy_manager_addr"`
}

type StrategyState struct {
	TotalShares     string `json:"total_shares"`
	UnderlyingToken string `json:"underlying_token"`
}

type TotalSharesResponse struct {
	TotalShares string `json:"total_shares"`
}

type UnderlyingTokenResponse struct {
	UnderlyingTokenAddr string `json:"underlying_token_addr"`
}

type SharesToUnderlyingResponse struct {
	AmountToSend string `json:"amount_to_send"`
}

type UnderlyingToShareResponse struct {
	ShareToSend string `json:"share_to_send"`
}

type UnderlyingToSharesResponse struct {
	ShareToSend string `json:"share_to_send"`
}

type UserUnderlyingResponse struct {
	AmountToSend string `json:"amount_to_send"`
}
