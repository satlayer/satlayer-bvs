package strategybasetvl

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (*api.StrategyBaseTvlLimits, io.ChainIO) {
	s := NewService()
	chainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	StrategyBaseTVL := api.NewStrategyBaseTVLLimits(chainIO, conf.C.Contract.StrategyBaseTVL)
	return StrategyBaseTVL, chainIO
}
func Deposit(userKeyName string, amount uint64) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.Deposit(context.Background(), amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Deposit success. txn: %s\n", resp.Hash)
}

func Withdraw(userKeyName, recipient string, amount uint64) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.Withdraw(context.Background(), recipient, amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Withdraw success. txn: %s\n", resp.Hash)
}

func Pause(userKeyName string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.PauseAll(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", resp.Hash)
}

func Unpause(userKeyName string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.UnpauseAll(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", resp.Hash)
}

func SetPauser(userKeyName, pauser string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.SetPauser(context.Background(), pauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", resp.Hash)
}

func SetUnpauser(userKeyName, unpauser string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.SetUnpauser(context.Background(), unpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", resp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.TransferOwnership(context.Background(), newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", resp.Hash)
}

func SetTvlLimits(userKeyName, maxPerDeposit string, maxTotalDeposits string) {
	StrategyBaseTVL, _ := newService(userKeyName)
	resp, err := StrategyBaseTVL.SetTvlLimits(context.Background(), maxPerDeposit, maxTotalDeposits)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set TVL limits success. txn: %s\n", resp.Hash)
}
