package economic

import (
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/testutil"
	"github.com/stretchr/testify/assert"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/logging"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
)

func TestEconomicCollector(t *testing.T) {
	delegationContrAddr := "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := ""

	logger, err := logging.NewMockZapLogger(logging.Development)
	assert.NoError(t, err)
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "metrics_economic")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "osmo", logger.GetElkLogger(), metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	assert.NoError(t, err)

	delegation := api.NewDelegationImpl(chainIO, delegationContrAddr)
	economicCollector := NewCollector(
		"localbvs",
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		[]string{"bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"},
		delegation,
		logger,
	)

	count := testutil.CollectAndCount(economicCollector, "satlayer_delegated_shares")
	assert.Equal(t, 1, count)
}
