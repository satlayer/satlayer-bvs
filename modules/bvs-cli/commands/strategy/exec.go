package strategy

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (api.StrategyManager, io.ChainIO) {
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

func SetDelegationManager(userKeyName, delegationManager string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.SetDelegationManager(ctx, delegationManager)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set delegation manager success. txn: %s\n", txResp.Hash)
}

func SetStrategyWhitelist(userKeyName, strategyWhitelist string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.SetStrategyWhitelister(ctx, strategyWhitelist)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set strategy whitelist success. txn: %s\n", txResp.Hash)
}
func AddStrategyWhitelist(userKeyName string, strategies []string, values []bool) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.AddStrategiesToWhitelist(ctx, strategies, values)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Add strategy whitelist success. txn: %s\n", txResp.Hash)
}
func RemoveStrategyWhitelist(userKeyName string, strategies []string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.RemoveStrategiesFromWhitelist(ctx, strategies)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Remove strategy whitelist success. txn: %s\n", txResp.Hash)
}

func SetThirdTransferForbidden(userKeyName, strategyAddress string, value bool) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.SetThirdPartyTransfersForbidden(ctx, strategyAddress, value)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set third party transfers forbidden success. txn: %s\n", txResp.Hash)
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

func Pause(userKeyName string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.Pause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", txResp.Hash)
}

func Unpause(userKeyName string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.Unpause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", txResp.Hash)
}
func SetPauser(userKeyName, newPauser string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.SetPauser(ctx, newPauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", txResp.Hash)
}
func SetUnpauser(userKeyName, newUnpauser string) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.SetUnpauser(ctx, newUnpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", txResp.Hash)
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

func WithdrawSharesAsTokens(userKeyName, recipient, strategyAddress, tokenAddress string, shares uint64) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.WithdrawSharesAsTokens(ctx, recipient, strategyAddress, shares, tokenAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Withdraw shares as tokens success. txn: %s\n", txResp.Hash)
}

func AddShares(userKeyName, stakerAddress, tokenAddress, strategyAddress string, shares uint64) {
	ctx := context.Background()
	strategy, _ := newService(userKeyName)
	txResp, err := strategy.AddShares(ctx, stakerAddress, tokenAddress, strategyAddress, shares)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Add shares success. txn: %s\n", txResp.Hash)
}
