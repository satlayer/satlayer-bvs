package types

import "math/big"

type SlashDetails struct {
	Slasher         string   `json:"slasher"`
	Operator        string   `json:"operator"`
	Share           *big.Int `json:"share"`
	SlashSignature  uint16   `json:"slashSignature"`
	SlashValidators []string `json:"slashValidators"`
	Reason          string   `json:"reason"`
	StartTime       *big.Int `json:"startTime"`
	EndTime         *big.Int `json:"endTime"`
	Status          bool     `json:"status"`
}
