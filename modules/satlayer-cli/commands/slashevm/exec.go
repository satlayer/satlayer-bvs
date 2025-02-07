package slashevm

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

func SetSlasher(userAddr, password, slasher string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	slasherAddr := common.HexToAddress(slasher)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.Slash.SetSlasher(ctx, wallet, slasherAddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set slasher success. txn: %s\n", txResp.TxHash)
}

func SetMinimalSlashSignature(userAddr, password string, minimalSignature int64) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	minimal := big.NewInt(minimalSignature)
	txResp, err := s.Slash.SetMinimalSlashSignature(ctx, wallet, minimal)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set minimal slash signature success. txn: %s\n", txResp.TxHash)
}

func SetSlasherValidator(userAddr, password string, validators []string, values []bool) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	txResp, err := s.Slash.SetSlashValidator(ctx, wallet, validators, values)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set slasher validator success. txn: %s\n", txResp.TxHash)
}

func Pause(userAddr, password string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	txResp, err := s.Slash.Pause(ctx, wallet)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", txResp.TxHash)
}

func Unpause(userAddr, password string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	txResp, err := s.Slash.Unpause(ctx, wallet)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", txResp.TxHash)
}

func TransferOwnership(userAddr, password string, newOwner string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	toAddr := common.HexToAddress(newOwner)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	txResp, err := s.Slash.TransferOwnership(ctx, wallet, toAddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", txResp.TxHash)
}

func SubmitSlashRequest(userAddr, password string, slasher string, operator string, share int64, slashSignature uint16, slashValidators []string, reason string, startTime int64, endTime int64, status bool, validatorsPublicKeys []string) {
	s := NewService()
	ctx := context.Background()
	slashDetails := types.SlashDetails{
		Slasher:         slasher,
		Operator:        operator,
		Share:           big.NewInt(share),
		SlashSignature:  slashSignature,
		SlashValidators: slashValidators,
		Reason:          reason,
		StartTime:       big.NewInt(startTime),
		EndTime:         big.NewInt(endTime),
		Status:          status,
	}
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.Slash.SubmitSlashRequest(ctx, wallet, slashDetails, validatorsPublicKeys)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Submit slash request success. txn: %s\n", txResp.TxHash)
}

func ExecuteSlashRequest(userAddr, password string, signatures []string, validatorsPublicKeys []string, slashHash string) {
	s := NewService()
	ctx := context.Background()

	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.Slash.ExecuteSlashRequest(ctx, wallet, slashHash, signatures, validatorsPublicKeys)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Execute slash request success. txn: %s\n", txResp.TxHash)
}

func CancelSlashRequest(userAddr, password string, slashHash string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	txResp, err := s.Slash.CancelSlashRequest(ctx, wallet, slashHash)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Cancel slash request success. txn: %s\n", txResp.TxHash)
}
