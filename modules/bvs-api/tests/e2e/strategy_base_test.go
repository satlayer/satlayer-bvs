package e2e

import (
	"context"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
)

type strategyBaseTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	contrAddr       string
	strategyManager string
}

func (suite *strategyBaseTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "strategy_base")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.strategyManager = "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"
}

func (suite *strategyBaseTestSuite) Test_ExecuteStrategyBase() {
	t := suite.T()
	contrAddr := "bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc"
	keyName := "caller"

	t.Logf("TestExecuteSquaring")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(contrAddr)

	resp, err := strategyBase.SetStrategyManager(context.Background(), suite.strategyManager)
	assert.NoError(t, err)
	assert.NotNil(t, resp)

	// these functions only invoked by strategy manager contract
	/*resp, err := strategyBase.Deposit(context.Background(),10)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyBase.Withdraw(context.Background(),"osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7", 5)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/
}

func (suite *strategyBaseTestSuite) Test_QueryStrategyBase() {
	t := suite.T()
	contrAddr := "bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc"
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(contrAddr)

	/*resp, err := strategyBase.GetShares("osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7", "osmo1p4ee54wcu54vcxht5spk5dpklr39qjpxxk38rm9p36c48rlgyawstwl3q8")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	resp, err := strategyBase.SharesToUnderlyingView(1)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyBase.UnderlyingToShareView(1)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	/*resp, err = strategyBase.UnderlyingView("osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/
}

func TestStrategyTestSuite(t *testing.T) {
	suite.Run(t, new(strategyBaseTestSuite))
}
