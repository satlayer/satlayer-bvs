package types

import (
	"database/sql"
	"database/sql/driver"
	"fmt"
	"strings"

	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func ToString(value sql.NullString) string {
	if value.Valid {
		return value.String
	}
	return ""
}

func ToNullString(value string) sql.NullString {
	value = strings.TrimSpace(value)
	return sql.NullString{
		Valid:  value != "",
		String: value,
	}
}

func RemoveEmpty(s []string) []string {
	var r []string
	for _, str := range s {
		if str != "" {
			r = append(r, str)
		}
	}
	return r
}

// _________________________________________________________

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

// Equal tells whether coin and d represent the same coin with the same amount
func (coin DBCoin) Equal(d DBCoin) bool {
	return coin.Denom == d.Denom && coin.Amount == d.Amount
}

// Value implements driver.Valuer
func (coin *DBCoin) Value() (driver.Value, error) {
	return fmt.Sprintf("(%s,%s)", coin.Denom, coin.Amount), nil
}

// Scan implements sql.Scanner
func (coin *DBCoin) Scan(src interface{}) error {
	strValue := string(src.([]byte))
	strValue = strings.ReplaceAll(strValue, `"`, "")
	strValue = strings.ReplaceAll(strValue, "{", "")
	strValue = strings.ReplaceAll(strValue, "}", "")
	strValue = strings.ReplaceAll(strValue, "(", "")
	strValue = strings.ReplaceAll(strValue, ")", "")

	values := strings.Split(strValue, ",")

	*coin = DBCoin{Denom: values[0], Amount: values[1]}
	return nil
}

// ToCoin converts this DbCoin to sdk.Coin
func (coin DBCoin) ToCoin() sdk.Coin {
	amount, _ := math.NewIntFromString(coin.Amount)
	return sdk.NewCoin(coin.Denom, amount)
}

// _________________________________________________________

// DBCoins represents an array of coins
type DBCoins []*DBCoin

// type DBCoins []*DBCoin

// NewDBCoins build a new DBCoins object starting from an array of coins
func NewDBCoins(coins sdk.Coins) DBCoins {
	dbCoins := make([]*DBCoin, 0)
	for _, coin := range coins {
		dbCoins = append(dbCoins, &DBCoin{Amount: coin.Amount.String(), Denom: coin.Denom})
	}
	return dbCoins
}

// Equal tells whether c and d contain the same items in the same order
func (coins DBCoins) Equal(d *DBCoins) bool {
	if d == nil {
		return false
	}

	if len(coins) != len(*d) {
		return false
	}

	for index, coin := range coins {
		if !coin.Equal(*(*d)[index]) {
			return false
		}
	}

	return true
}

// func (coins DBCoins) Equal(d *DBCoins) bool {
// 	if d == nil {
// 		return false
// 	}
//
// 	if len(coins) != len(*d) {
// 		return false
// 	}
//
// 	for index, coin := range coins {
// 		if !coin.Equal(*(*d)[index]) {
// 			return false
// 		}
// 	}
//
// 	return true
// }

// Scan implements sql.Scanner
func (coins *DBCoins) Scan(src interface{}) error {
	strValue := string(src.([]byte))
	strValue = strings.ReplaceAll(strValue, `"`, "")
	strValue = strings.ReplaceAll(strValue, "{", "")
	strValue = strings.ReplaceAll(strValue, "}", "")
	strValue = strings.ReplaceAll(strValue, "),(", ") (")
	strValue = strings.ReplaceAll(strValue, "(", "")
	strValue = strings.ReplaceAll(strValue, ")", "")

	values := RemoveEmpty(strings.Split(strValue, " "))

	coinsV := make(DBCoins, len(values))
	for index, value := range values {
		v := strings.Split(value, ",") // Split the values

		coin := DBCoin{Denom: v[0], Amount: v[1]}
		coinsV[index] = &coin
	}

	*coins = coinsV
	return nil
}

// ToCoins converts this DbCoins to sdk.Coins
func (coins DBCoins) ToCoins() sdk.Coins {
	sdkCoins := make([]sdk.Coin, len(coins))
	for index := range coins {
		sdkCoins[index] = coins[index].ToCoin()
	}
	return sdkCoins
}

// --------------------------------------------------------------------------------------------------------------------

// DBDecCoin represents the information stored inside the database about a single coin
type DBDecCoin struct {
	Denom  string
	Amount string
}

// NewDbDecCoin builds a DbDecCoin starting from an SDK Coin
func NewDBDecCoin(coin sdk.DecCoin) DBDecCoin {
	return DBDecCoin{
		Denom:  coin.Denom,
		Amount: coin.Amount.String(),
	}
}

// Equal tells whether coin and d represent the same coin with the same amount
func (coin DBDecCoin) Equal(d DBDecCoin) bool {
	return coin.Denom == d.Denom && coin.Amount == d.Amount
}

// Value implements driver.Valuer
func (coin *DBDecCoin) Value() (driver.Value, error) {
	return fmt.Sprintf("(%s,%s)", coin.Denom, coin.Amount), nil
}

// Scan implements sql.Scanner
func (coin *DBDecCoin) Scan(src interface{}) error {
	strValue := string(src.([]byte))
	strValue = strings.ReplaceAll(strValue, `"`, "")
	strValue = strings.ReplaceAll(strValue, "{", "")
	strValue = strings.ReplaceAll(strValue, "}", "")
	strValue = strings.ReplaceAll(strValue, "(", "")
	strValue = strings.ReplaceAll(strValue, ")", "")

	values := strings.Split(strValue, ",")
	*coin = DBDecCoin{Denom: values[0], Amount: values[1]}
	return nil
}

// ToDecCoin converts this DbDecCoin to sdk.DecCoin
func (coin DBDecCoin) ToDecCoin() sdk.DecCoin {
	amount, _ := math.LegacyNewDecFromStr(coin.Amount)
	return sdk.NewDecCoinFromDec(coin.Denom, amount)
}

// _________________________________________________________

// DbDecCoins represents an array of coins
type DBDecCoins []*DBDecCoin

// NewDbDecCoins build a new DbDecCoins object starting from an array of coins
func NewDBDecCoins(coins sdk.DecCoins) DBDecCoins {
	DbDecCoins := make([]*DBDecCoin, 0)
	for _, coin := range coins {
		DbDecCoins = append(DbDecCoins, &DBDecCoin{Amount: coin.Amount.String(), Denom: coin.Denom})
	}
	return DbDecCoins
}

// Equal tells whether c and d contain the same items in the same order
func (coins DBDecCoins) Equal(d *DBDecCoins) bool {
	if d == nil {
		return false
	}

	if len(coins) != len(*d) {
		return false
	}

	for index, coin := range coins {
		if !coin.Equal(*(*d)[index]) {
			return false
		}
	}

	return true
}

// Scan implements sql.Scanner
func (coins *DBDecCoins) Scan(src interface{}) error {
	strValue := string(src.([]byte))
	strValue = strings.ReplaceAll(strValue, `"`, "")
	strValue = strings.ReplaceAll(strValue, "{", "")
	strValue = strings.ReplaceAll(strValue, "}", "")
	strValue = strings.ReplaceAll(strValue, "),(", ") (")
	strValue = strings.ReplaceAll(strValue, "(", "")
	strValue = strings.ReplaceAll(strValue, ")", "")

	values := RemoveEmpty(strings.Split(strValue, " "))

	coinsV := make(DBDecCoins, len(values))
	for index, value := range values {
		v := strings.Split(value, ",") // Split the values

		coin := DBDecCoin{Denom: v[0], Amount: v[1]}
		coinsV[index] = &coin
	}

	*coins = coinsV
	return nil
}

// ToDecCoins converts this DBDecCoins to sdk.DecCoins
func (coins DBDecCoins) ToDecCoins() sdk.DecCoins {
	sdkCoins := make([]sdk.DecCoin, len(coins))
	for index := range coins {
		sdkCoins[index] = coins[index].ToDecCoin()
	}
	return sdkCoins
}
