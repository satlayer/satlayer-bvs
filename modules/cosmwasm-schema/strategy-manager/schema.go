// This file was automatically generated from strategy-manager/schema.json.
// DO NOT MODIFY IT BY HAND.

package strategymanager

type IsStrategyWhitelistedResponse bool

type StakerDepositListResponse []StrategyShare

type StakerStrategyListResponse []string

type StakerStrategySharesResponse string

type InstantiateMsg struct {
	Owner  string `json:"owner"`
	Pauser string `json:"pauser"`
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
