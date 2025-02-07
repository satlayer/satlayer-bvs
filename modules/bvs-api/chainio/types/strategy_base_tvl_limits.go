package types

type GetTVLLimitsReq struct {
	GetTVLLimits struct{} `json:"get_t_v_l_limits"`
}

type TVLLimitsResponse struct {
	MaxPerDeposit    string `json:"max_per_deposit"`
	MaxTotalDeposits string `json:"max_total_deposits"`
}

type SetTVLLimitsReq struct {
	SetTVLLimits SetTVLLimits `json:"set_t_v_l_limits"`
}

type SetTVLLimits struct {
	MaxPerDeposit    string `json:"max_per_deposit"`
	MaxTotalDeposits string `json:"max_total_deposits"`
}

type GetTotalSharesReq struct {
	GetTotalShares struct{} `json:"get_total_shares"`
}

type GetTotalSharesResponse struct {
	TotalShares string `json:"total_shares"`
}

type ExplanationReq struct {
	Explanation struct{} `json:"explanation"`
}

type ExplanationResponse struct {
	Explanation string `json:"explanation"`
}

type GetStrategyStateReq struct {
	GetStrategyState struct{} `json:"get_strategy_state"`
}

type GetStrategyStateResponse struct {
	StrategyManager string `json:"strategy_manager"`
	UnderlyingToken string `json:"underlying_token"`
	TotalShares     string `json:"total_shares"`
}

type UnderlyingToSharesReq struct {
	UnderlyingToShares UnderlyingToShares `json:"underlying_to_shares"`
}

type UnderlyingToShares struct {
	AmountUnderlying string `json:"amount_underlying"`
}

type UnderlyingToSharesResponse struct {
	ShareToSend string `json:"share_to_send"`
}

type GetStrategyManagerReq struct {
	GetStrategyManager struct{} `json:"get_strategy_manager"`
}

type GetStrategyManagerResponse struct {
	StrategyManager string `json:"strate_manager_addr"`
}

type BaseTVLSetStrategyManagerReq struct {
	SetStrategyManager BaseTVLSetStrategyManager `json:"set_strategy_manager"`
}

type BaseTVLSetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}
