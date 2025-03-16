// This file was automatically generated from strategy-base/schema.json.
// DO NOT MODIFY IT BY HAND.

package strategybase

type SharesResponse string

type SharesToUnderlyingResponse string

type StrategyManagerResponse string

type TotalSharesResponse string

type UnderlyingResponse string

type UnderlyingToSharesResponse string

type UnderlyingTokenResponse string

type InstantiateMsg struct {
	Owner           string `json:"owner"`
	Pauser          string `json:"pauser"`
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
	Sender string `json:"sender"`
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
