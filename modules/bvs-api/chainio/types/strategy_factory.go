package types

// Execute messages

type DeployNewStrategyReq struct {
	DeployNewStrategy DeployNewStrategy `json:"deploy_new_strategy"`
}

type DeployNewStrategy struct {
	Token    string `json:"token"`
	Pauser   string `json:"pauser"`
	Unpauser string `json:"unpauser"`
}

type UpdateConfigReq struct {
	UpdateConfig UpdateConfig `json:"update_config"`
}

type UpdateConfig struct {
	NewOwner       string `json:"new_owner"`
	StrategyCodeId uint64 `json:"strategy_code_id"`
}

type BlacklistTokensReq struct {
	BlacklistTokens BlacklistTokens `json:"blacklist_tokens"`
}

type BlacklistTokens struct {
	Tokens []string `json:"tokens"`
}

type SetThirdPartyTransfersForBiddenReq struct {
	SetThirdPartyTransfersForBidden SetThirdPartyTransfersForBidden `json:"set_third_party_transfers_for_bidden"`
}

type SetThirdPartyTransfersForBidden struct {
	Strategy string `json:"strategy"`
	Value    bool   `json:"value"`
}

type WhitelistStrategiesReq struct {
	WhitelistStrategies WhitelistStrategies `json:"whitelist_strategies"`
}

type WhitelistStrategies struct {
	StrategiesToWhitelist              []string `json:"strategies_to_whitelist"`
	ThirdPartyTransfersForbiddenValues []bool   `json:"third_party_transfers_forbidden_values"`
}

type SetStrategyManagerReq struct {
	SetStrategyManager SetStrategyManager `json:"set_strategy_manager"`
}

type SetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}

type TransferOwnershipReq struct {
	TransferOwnership TransferOwnership `json:"transfer_ownership"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type PauseFactoryReq struct {
	Pause struct{} `json:"pause"`
}

type UnpauseFactoryReq struct {
	Unpause struct{} `json:"unpause"`
}

type SetPauserFactoryReq struct {
	SetPauser SetPauser `json:"set_pauser"`
}

type SetPauserFactory struct {
	NewPauser string `json:"new_pauser"`
}

type SetUnpauserFactoryReq struct {
	SetUnpauser SetUnpauser `json:"set_unpauser"`
}

type SetUnpauserFactory struct {
	NewUnpauser string `json:"new_unpauser"`
}

// Query messages

type GetStrategyReq struct {
	GetStrategy GetStrategy `json:"get_strategy"`
}

type GetStrategy struct {
	Token string `json:"token"`
}

type IsTokenBlacklistedReq struct {
	IsTokenBlacklisted IsTokenBlacklisted `json:"is_token_blacklisted"`
}

type IsTokenBlacklisted struct {
	Token string `json:"token"`
}

// Responses

type GetStrategyResponse struct {
	Strategy string `json:"strategy"`
}

type BlacklistStatusResponse struct {
	IsBlacklisted bool `json:"is_blacklisted"`
}
