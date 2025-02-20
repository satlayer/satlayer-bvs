package strategyfactory

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (api.StrategyFactory, io.ChainIO) {
	s := NewService()
	chainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	factoryApi := api.NewStrategyFactoryImpl(chainIO).WithGasLimit(2000000)
	factoryApi.BindClient(conf.C.Contract.StrategyFactory)
	return factoryApi, chainIO
}

func CreateStrategy(userKeyName, token, pauser, unpauser string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.DeployNewStrategy(context.Background(), token, pauser, unpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Deploy new strategy success. txn: %s\n", resp.Hash)
}

func UpdateConfig(userKeyName, newOwner string, strategyCodeId int64) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.UpdateConfig(context.Background(), newOwner, strategyCodeId)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Update config success. txn: %s\n", resp.Hash)
}

func Pause(userKeyName string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.Pause(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", resp.Hash)
}

func Unpause(userKeyName string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.Unpause(context.Background())
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", resp.Hash)
}

func SetPauser(userKeyName, pauser string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.SetPauser(context.Background(), pauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", resp.Hash)
}

func SetUnpauser(userKeyName, unpauser string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.SetUnpauser(context.Background(), unpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", resp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.TransferOwnership(context.Background(), newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", resp.Hash)
}

func BlacklistTokens(userKeyName string, tokens []string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.BlacklistTokens(context.Background(), tokens)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Blacklist tokens success. txn: %s\n", resp.Hash)
}

func RemoveStrategiesFromWhitelist(userKeyName string, strategies []string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.RemoveStrategiesFromWhitelist(context.Background(), strategies)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Remove strategies from whitelist success. txn: %s\n", resp.Hash)
}

func SetThirdPartyTransfersForBidden(userKeyName, strategy string, value bool) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.SetThirdPartyTransfersForBidden(context.Background(), strategy, value)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set third party transfers forbidden success. txn: %s\n", resp.Hash)
}

func WhitelistStrategies(userKeyName string, strategies []string, forbiddenValues []bool) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.WhitelistStrategies(context.Background(), strategies, forbiddenValues)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Whitelist strategies success. txn: %s\n", resp.Hash)
}

func SetStrategyManager(userKeyName, newManager string) {
	strategyFactory, _ := newService(userKeyName)
	resp, err := strategyFactory.SetStrategyManager(context.Background(), newManager)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set strategy manager success. txn: %s\n", resp.Hash)
}
