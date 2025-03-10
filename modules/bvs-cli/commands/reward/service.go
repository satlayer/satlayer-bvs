package reward

import (
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

type Service struct {
	Reward  *api.RewardsCoordinator
	ChainIO io.ChainIO
}

func NewService() *Service {
	conf.InitConfig()
	chainIO, err := io.NewChainIO(conf.C.Chain.ID, conf.C.Chain.RPC, conf.C.Account.KeyDir, conf.C.Account.Bech32Prefix, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}
	reward := api.NewRewardsCoordinator(chainIO)
	reward.BindClient(conf.C.Contract.RewardCoordinator)

	return &Service{Reward: reward, ChainIO: chainIO}
}
