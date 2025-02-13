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

package strategy_base

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

type InstantiateMsg struct {
	InitialOwner        string `json:"initial_owner"`
	InitialPausedStatus int64  `json:"initial_paused_status"`
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
	Pause              *Pause              `json:"pause,omitempty"`
	Unpause            *Unpause            `json:"unpause,omitempty"`
	SetPauser          *SetPauser          `json:"set_pauser,omitempty"`
	SetUnpauser        *SetUnpauser        `json:"set_unpauser,omitempty"`
}

type Deposit struct {
	Amount string `json:"amount"`
}

type Pause struct {
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
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
