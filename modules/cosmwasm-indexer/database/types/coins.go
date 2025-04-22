package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DBCoin represents the information stored inside the database about a single coin
type DBCoin struct {
	Denom  string
	Amount string
}

// NewDBCoin builds a DbCoin starting from an SDK Coin
func NewDBCoin(coin sdk.Coin) DBCoin {
	return DBCoin{
		Denom:  coin.Denom,
		Amount: coin.Amount.String(),
	}
}

// DBCoins represents an array of coins
type DBCoins []*DBCoin

// NewDBCoins build a new DBCoins object starting from an array of coins
func NewDBCoins(coins sdk.Coins) DBCoins {
	dbCoins := make([]*DBCoin, 0)
	for _, coin := range coins {
		dbCoins = append(dbCoins, &DBCoin{Amount: coin.Amount.String(), Denom: coin.Denom})
	}
	return dbCoins
}
