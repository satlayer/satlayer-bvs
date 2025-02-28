package economic

import (
	"context"
	"testing"

	"github.com/prometheus/client_golang/prometheus/testutil"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/stretchr/testify/assert"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/logging"
)

func TestEconomicCollector(t *testing.T) {
	container := babylond.Run(context.Background())
	chainIO := container.NewChainIO("../.babylon")

	logger, err := logging.NewMockZapLogger(logging.Development)
	assert.NoError(t, err)

	deployer := &bvs.Deployer{BabylonContainer: container}
	registry := deployer.DeployRegistry(nil)

	tAddr := container.GenerateAddress("test-address").String()
	delegationManager := deployer.DeployDelegationManager(
		registry.Address,
		tAddr, tAddr, 100, []string{tAddr}, []int64{50},
	)

	delegation := api.NewDelegationManager(chainIO, delegationManager.Address)
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
