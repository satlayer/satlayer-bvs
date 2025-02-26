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
//    explanationResponse, err := UnmarshalExplanationResponse(bytes)
//    bytes, err = explanationResponse.Marshal()
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
//    tvlLimitsResponse, err := UnmarshalTvlLimitsResponse(bytes)
//    bytes, err = tvlLimitsResponse.Marshal()
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

package strategybasetvllimits

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

func UnmarshalExplanationResponse(data []byte) (ExplanationResponse, error) {
	var r ExplanationResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ExplanationResponse) Marshal() ([]byte, error) {
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

func UnmarshalTvlLimitsResponse(data []byte) (TvlLimitsResponse, error) {
	var r TvlLimitsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TvlLimitsResponse) Marshal() ([]byte, error) {
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
	InitialOwner        string `json:"initial_owner"`
	InitialPausedStatus int64  `json:"initial_paused_status"`
	MaxPerDeposit       string `json:"max_per_deposit"`
	MaxTotalDeposits    string `json:"max_total_deposits"`
	Pauser              string `json:"pauser"`
	StrategyManager     string `json:"strategy_manager"`
	UnderlyingToken     string `json:"underlying_token"`
	Unpauser            string `json:"unpauser"`
}

type ExecuteMsg struct {
	Deposit            *Deposit            `json:"deposit,omitempty"`
	Withdraw           *Withdraw           `json:"withdraw,omitempty"`
	SetStrategyManager *SetStrategyManager `json:"set_strategy_manager,omitempty"`
	TransferOwnership  *TransferOwnership  `json:"transfer_ownership,omitempty"`
	PauseAll           *PauseAll           `json:"pause_all,omitempty"`
	UnpauseAll         *UnpauseAll         `json:"unpause_all,omitempty"`
	PauseBit           *PauseBit           `json:"pause_bit,omitempty"`
	UnpauseBit         *UnpauseBit         `json:"unpause_bit,omitempty"`
	SetPauser          *SetPauser          `json:"set_pauser,omitempty"`
	SetUnpauser        *SetUnpauser        `json:"set_unpauser,omitempty"`
	SetTvlLimits       *SetTvlLimits       `json:"set_tvl_limits,omitempty"`
}

type Deposit struct {
	Amount string `json:"amount"`
}

type PauseAll struct {
}

type UnpauseAll struct {
}

type PauseBit struct {
	Index uint8 `json:"index"`
}

type UnpauseBit struct {
	Index uint8 `json:"index"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}

type SetTvlLimits struct {
	MaxPerDeposit    string `json:"max_per_deposit"`
	MaxTotalDeposits string `json:"max_total_deposits"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type Withdraw struct {
	AmountShares string `json:"amount_shares"`
	Recipient    string `json:"recipient"`
	Token        string `json:"token"`
}

type QueryMsg struct {
	GetShares              *GetShares              `json:"get_shares,omitempty"`
	SharesToUnderlyingView *SharesToUnderlyingView `json:"shares_to_underlying_view,omitempty"`
	UnderlyingToShareView  *UnderlyingToShareView  `json:"underlying_to_share_view,omitempty"`
	UserUnderlyingView     *UserUnderlyingView     `json:"user_underlying_view,omitempty"`
	GetStrategyManager     *GetStrategyManager     `json:"get_strategy_manager,omitempty"`
	GetUnderlyingToken     *GetUnderlyingToken     `json:"get_underlying_token,omitempty"`
	GetTotalShares         *GetTotalShares         `json:"get_total_shares,omitempty"`
	Explanation            *Explanation            `json:"explanation,omitempty"`
	UnderlyingToShares     *UnderlyingToShares     `json:"underlying_to_shares,omitempty"`
	GetStrategyState       *GetStrategyState       `json:"get_strategy_state,omitempty"`
	GetTvlLimits           *GetTvlLimits           `json:"get_tvl_limits,omitempty"`
}

type Explanation struct {
}

type GetShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type GetStrategyManager struct {
}

type GetStrategyState struct {
}

type GetTotalShares struct {
}

type GetTvlLimits struct {
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

type ExplanationResponse struct {
	Explanation string `json:"explanation"`
}

type SharesResponse struct {
	TotalShares string `json:"total_shares"`
}

type StrategyManagerResponse struct {
	StrategyManagerAddr string `json:"strategy_manager_addr"`
}

type StrategyState struct {
	StrategyManager string `json:"strategy_manager"`
	TotalShares     string `json:"total_shares"`
	UnderlyingToken string `json:"underlying_token"`
}

type TotalSharesResponse struct {
	TotalShares string `json:"total_shares"`
}

type TvlLimitsResponse struct {
	MaxPerDeposit    string `json:"max_per_deposit"`
	MaxTotalDeposits string `json:"max_total_deposits"`
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
