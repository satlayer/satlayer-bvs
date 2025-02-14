package babylond

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"time"
)

func (d *BabylonContainer) GetChainIO() (io.ChainIO, error) {
	homeDir := "../.babylon"
	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	return io.NewChainIO(ChainId, d.GetRpcUri(), homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
}
