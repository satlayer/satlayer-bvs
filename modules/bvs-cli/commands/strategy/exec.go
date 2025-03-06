package strategy

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (*api.StrategyManager, io.ChainIO) {
	s := NewService()
	chainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	strategy := api.NewStrategyManager(chainIO).WithGasLimit(4000000)
	strategy.BindClient(conf.C.Contract.Strategy)
	return strategy, chainIO
}

func TransferOwner(userKeyName, newOwner string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.TransferOwnership(ctx, newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", txResp.Hash)
}

func UpdateStrategy(userKeyName string, strategy string, whitelisted bool) {
	ctx := context.Background()
	strategyManager, _ := newService(userKeyName)
	txResp, err := strategyManager.UpdateStrategy(ctx, strategy, whitelisted)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Add strategy whitelist success. txn: %s\n", txResp.Hash)
}

func DepositStrategy(userKeyName, strategyAddress, tokenAddress string, amount uint64) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.DepositIntoStrategy(ctx, strategyAddress, tokenAddress, amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Deposit strategy success. txn: %s\n", txResp.Hash)
}

func RemoveShares(userKeyName, stakerAddress, strategyAddress string, shares uint64) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.RemoveShares(ctx, stakerAddress, strategyAddress, shares)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Remove shares success. txn: %s\n", txResp.Hash)
}

func WithdrawSharesAsTokens(userKeyName, recipient, strategyAddress, shares string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.WithdrawSharesAsTokens(ctx, recipient, strategyAddress, shares)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Withdraw shares as tokens success. txn: %s\n", txResp.Hash)
}
