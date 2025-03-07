package strategybase

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (*api.StrategyBase, io.ChainIO) {
	s := NewService()
	chainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(conf.C.Contract.StrategyBase)
	return strategyBase, chainIO
}

// Deposit TODO: Deprecated (only called by strategy_manager)
func Deposit(userKeyName string, amount uint64) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.Deposit(context.Background(), amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Deposit success. txn: %s\n", resp.Hash)
}

// Withdraw TODO: Deprecated (only called by strategy_manager)
func Withdraw(userKeyName, recipient string, amount string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.Withdraw(context.Background(), recipient, amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Withdraw success. txn: %s\n", resp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.TransferOwnership(context.Background(), newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", resp.Hash)
}
