package slash

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
	ChainIO io.ChainIO
	Slash   api.SlashManager
}

func NewService() *Service {
	conf.InitConfig()
	logger := logger2.NewELKLogger("satlayer-cli")
	logger.SetLogLevel(conf.C.LogLevel)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "slash")
	chainIO, err := io.NewChainIO(conf.C.Chain.ID, conf.C.Chain.RPC, conf.C.Account.KeyDir, conf.C.Account.Bech32Prefix, logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	if conf.C.Contract.Slash == "" {
		panic("Contract address for Slash is empty!")
	}

	slashManager := api.NewSlashManager(chainIO, conf.C.Contract.Slash).WithGasLimit(2000000)
	slashManager.BindClient(conf.C.Contract.Slash)
	return &Service{ChainIO: chainIO, Slash: slashManager}
}
