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
	strategyBase := api.NewStrategyBase(chainIO, conf.C.Contract.StrategyBase)
	return strategyBase, chainIO
}

func Deposit(userKeyName string, amount uint64) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.Deposit(context.Background(), amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Deposit success. txn: %s\n", resp.Hash)
}

func Withdraw(userKeyName, recipient string, amount uint64) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.Withdraw(context.Background(), recipient, amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Withdraw success. txn: %s\n", resp.Hash)
}

func Pause(uerKeyName string) {
	strategyBase, _ := newService(uerKeyName)
	resp, err := strategyBase.Pause(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", resp.Hash)
}

func Unpause(userKeyName string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.Unpause(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", resp.Hash)
}

func SetPauser(userKeyName, pauser string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.SetPauser(context.Background(), pauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", resp.Hash)
}

func SetUnpauser(userKeyName, unpauser string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.SetUnpauser(context.Background(), unpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", resp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	strategyBase, _ := newService(userKeyName)
	resp, err := strategyBase.TransferOwnership(context.Background(), newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", resp.Hash)
}
