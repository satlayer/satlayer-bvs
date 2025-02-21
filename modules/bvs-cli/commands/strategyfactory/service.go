package strategyfactory

import (
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	logger2 "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

type Service struct {
	ChainIO         io.ChainIO
	StrategyFactory api.StrategyFactory
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "strategyfactory")
	chainIO, err := io.NewChainIO(conf.C.Chain.ID, conf.C.Chain.RPC, conf.C.Account.KeyDir, conf.C.Account.Bech32Prefix, logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	if conf.C.Contract.StrategyFactory == "" {
		panic("Contract address for Strategy Factory is empty!")
	}

	factoryApi := api.NewStrategyFactory(chainIO).WithGasLimit(2000000)
	factoryApi.BindClient(conf.C.Contract.StrategyFactory)
	return &Service{ChainIO: chainIO, StrategyFactory: factoryApi}
}
