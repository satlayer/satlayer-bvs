package strategybase

import (
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	logger2 "github.com/satlayer/satlayer-bvs/bvs-api/logger"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

type Service struct {
	StrategyBase *api.StrategyBase
	ChainIO      io.ChainIO
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	chainIO, err := io.NewChainIO(conf.C.Chain.ID, conf.C.Chain.RPC, conf.C.Account.KeyDir, conf.C.Account.Bech32Prefix, logger, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}
	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(conf.C.Contract.StrategyBase)
	return &Service{StrategyBase: strategyBase, ChainIO: chainIO}
}
