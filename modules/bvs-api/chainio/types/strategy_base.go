package types

type DepositReq struct {
	Deposit Deposit `json:"deposit"`
}

type Deposit struct {
	Amount string `json:"amount"`
}

type WithdrawReq struct {
	Withdraw Withdraw `json:"withdraw"`
}

type Withdraw struct {
	Recipient    string `json:"recipient"`
	AmountShares string `json:"amount_shares"`
}

type PauseReq struct {
	Pause struct{} `json:"pause"`
}

type UnPauseReq struct {
	UnPause struct{} `json:"unpause"`
}

type SetPauserReq struct {
	SetPauser SetPauser `json:"set_pauser"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetUnpauserReq struct {
	SetUnpauser SetUnpauser `json:"set_unpauser"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TransferStrategyBaseOwnershipReq struct {
	TransferOwnership TransferStrategyBaseOwnership `json:"transfer_ownership"`
}

type TransferStrategyBaseOwnership struct {
	NewOwner string `json:"new_owner"`
}

type GetSharesReq struct {
	GetShares GetShares `json:"get_shares"`
}

type GetShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type SharesToUnderlyingViewReq struct {
	SharesToUnderlyingView SharesToUnderlyingView `json:"shares_to_underlying_view"`
}

type SharesToUnderlyingView struct {
	AmountShares string `json:"amount_shares"`
}

type UnderlyingToShareViewReq struct {
	UnderlyingToShareView UnderlyingToShareView `json:"underlying_to_share_view"`
}

type UnderlyingToShareView struct {
	Amount string `json:"amount"`
}

type UserUnderlyingViewReq struct {
	UserUnderlyingView UserUnderlyingView `json:"user_underlying_view"`
}

type UserUnderlyingView struct {
	User string `json:"user"`
}

type UnderlyingTokenReq struct {
	GetUnderlyingToken struct{} `json:"get_underlying_token"`
}

type BaseSetStrategyManagerReq struct {
	SetStrategyManager BaseSetStrategyManager `json:"set_strategy_manager"`
}

type BaseSetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
}
