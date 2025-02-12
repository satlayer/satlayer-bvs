package uploader

import "github.com/shopspring/decimal"

type Staker struct {
	TotalAmount decimal.Decimal
	Strategies  []Strategy
}

type Strategy struct {
	Strategy string
	Amount   decimal.Decimal
}

type Submission struct {
	Strategy string          `json:"strategy"`
	Token    string          `json:"token"`
	Amount   decimal.Decimal `json:"amount"`
}

type Earner struct {
	Earner           string          `json:"earner"`
	TotalStakeAmount decimal.Decimal `json:"total_stake_amount"`
	Tokens           []*TokenAmount  `json:"tokens"`
}

type TokenAmount struct {
	Strategy     string          `json:"strategy"`
	Token        string          `json:"token"`
	RewardAmount string          `json:"reward_amount"`
	StakeAmount  decimal.Decimal `json:"stake_amount"`
}

type RewardEarner struct {
	Earner    string         `json:"earner"`
	TokenHash string         `json:"token_hash"`
	Tokens    []*RewardToken `json:"tokens"`
}

type RewardToken struct {
	Token  string `json:"token"`
	Amount string `json:"amount"`
}

type RewardUploadRequest struct {
	Timestamp   int64           `json:"timestamp" binding:"required"`
	Signature   string          `json:"signature" binding:"required"`
	PubKey      string          `json:"pub_key" binding:"required"`
	RootIndex   int64           `json:"root_index" binding:"required"`
	RootHash    string          `json:"root_hash" binding:"required"`
	TxnHash     string          `json:"txn_hash" binding:"required"`
	CreateTs    int64           `json:"create_ts" binding:"required"`
	CalcEndTs   int64           `json:"calc_end_ts" binding:"required"`
	ActivatedTs int64           `json:"activated_ts" binding:"required"`
	Earners     []*RewardEarner `json:"earners" binding:"required"`
}

type EarnerTokenRequest struct {
	Timestamp    int64               `json:"timestamp" binding:"required"`
	Signature    string              `json:"signature" binding:"required"`
	PubKey       string              `json:"pub_key" binding:"required"`
	EarnerTokens []EarnerTokenAmount `json:"earner_tokens" binding:"required"`
}

type EarnerTokenAmount struct {
	Earner string `json:"earner"`
	Token  string `json:"token"`
	Amount string `json:"amount"`
}
