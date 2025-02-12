package reward

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (api.RewardsCoordinator, io.ChainIO) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	reward := api.NewRewardsCoordinator(newChainIO).WithGasLimit(400000)
	reward.BindClient(conf.C.Contract.RewardCoordinator)
	return reward, newChainIO
}

func SetClaimer(userKeyName, claimer string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetClaimerFor(ctx, claimer)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set claimer success. txn: %s\n", resp.Hash.String())
}

func SetActivationDelay(userKeyName string, newActivationDelay uint32) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetActivationDelay(ctx, newActivationDelay)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set activation delay success. txn: %s\n", resp.Hash.String())
}

func SetGlobalOperatorCommission(userKeyName string, newCommissionBips uint16) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetGlobalOperatorCommission(ctx, newCommissionBips)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set global operator commission success. txn: %s\n", resp.Hash.String())
}

func SetRewardUpdater(userKeyName, rewardUpdater string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetRewardsUpdater(ctx, rewardUpdater)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set reward updater success. txn: %s\n", resp.Hash.String())
}

func Pause(userKeyName string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.Pause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", resp.Hash.String())
}

func Unpause(userKeyName string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.Unpause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", resp.Hash.String())
}

func SetPauser(userKeyName, newPauser string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetPauser(ctx, newPauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", resp.Hash.String())
}

func SetUnpauser(userKeyName, unpauser string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.SetUnpauser(ctx, unpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", resp.Hash.String())
}

func TransferOwner(userKeyName, newOwner string) {
	ctx := context.Background()
	reward, _ := newService(userKeyName)
	resp, err := reward.TransferOwnership(ctx, newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", resp.Hash.String())
}
