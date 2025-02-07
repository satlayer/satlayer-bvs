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

type strategyBaseTVLLimitsTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	contrAddr       string
	strategyManager string
}

func (suite *strategyBaseTVLLimitsTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "strategy_tvl_limits")
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

func (suite *strategyBaseTVLLimitsTestSuite) Test_ExecuteStrategyTVLLimits() {
	t := suite.T()
	contrAddr := "bbn18czayk3z6k2zk6yjqpcagdvzgw6rjhy65g88vqdzjg4l5k4rhngs7v7geg"
	keyName := "caller"

	t.Logf("TestExecuteStrategyTVLLimits")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyTVLLimits := api.NewStrategyBaseTVLLimits(chainIO)
	strategyTVLLimits.BindClient(contrAddr)

	resp, err := strategyTVLLimits.SetStrategyManager(context.Background(), suite.strategyManager)
	assert.NoError(t, err)
	assert.NotNil(t, resp)

	// these functions only invoked by strategy manager contract
	/*resp, err := strategyTVLLimits.Deposit(context.Background(), 100)
	assert.NoError(t, err, "Deposit failed")
	assert.NotNil(t, resp, "response is nil")
	t.Logf("Deposit Response: %+v", resp)

	resp, err = strategyTVLLimits.Withdraw(context.Background(), "osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7", 7)
	assert.NoError(t, err, "Withdraw failed")
	assert.NotNil(t, resp, "response is nil")
	t.Logf("Withdraw Response: %+v", resp)*/

	// resp, err := strategyTVLLimits.SetTVLLimits(context.Background(), "11000000", "13000000")
	// assert.NoError(t, err, "SetTVLLimits failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("SetTVLLimits Response: %+v", resp)

	// resp, err = strategyTVLLimits.TransferOwnership(context.Background(), "osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7")
	// assert.NoError(t, err, "TransferOwnership failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("TransferOwnership Response: %+v", resp)

	// resp, err = strategyTVLLimits.SetPauser(context.Background(), "osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7")
	// assert.NoError(t, err, "SetPauser failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("SetPauser Response: %+v", resp)

	// resp, err = strategyTVLLimits.SetUnpauser(context.Background(), "osmo1fxqtqvcsglst7pmnd0a9ftytsxt8g75r6cugv7")
	// assert.NoError(t, err, "SetUnpauser failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("SetUnpauser Response: %+v", resp)

	// resp, err = strategyTVLLimits.Pause(context.Background())
	// assert.NoError(t, err, "Pause failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("Pause Response: %+v", resp)

	// resp, err = strategyTVLLimits.Unpause(context.Background())
	// assert.NoError(t, err, "Unpause failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("Unpause Response: %+v", resp)
}

func (suite *strategyBaseTVLLimitsTestSuite) Test_QueryStrategyTVLLimits() {
	t := suite.T()
	contrAddr := "bbn108l2c6l5aw0pv68mhq764kq9344h4prefft4uufelmweasfstfzsxv0w5p"
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyTVLLimits := api.NewStrategyBaseTVLLimits(chainIO)
	strategyTVLLimits.BindClient(contrAddr)

	tvlLimitsResp, err := strategyTVLLimits.GetTVLLimits()
	assert.NoError(t, err, "GetTVLLimits failed")
	assert.NotNil(t, tvlLimitsResp, "response is nil")
	t.Logf("TVL Limits: %+v", tvlLimitsResp)

	totalDepositsResp, err := strategyTVLLimits.GetTotalDeposits()
	assert.NoError(t, err, "GetTotalDeposits failed")
	assert.NotNil(t, totalDepositsResp, "response is nil")
	t.Logf("Total Deposits: %+v", totalDepositsResp)

	explanationResp, err := strategyTVLLimits.Explanation()
	assert.NoError(t, err, "Explanation query failed")
	assert.NotNil(t, explanationResp, "response is nil")
	t.Logf("Explanation: %+v", explanationResp)

	strategyStateResp, err := strategyTVLLimits.GetStrategyState()
	assert.NoError(t, err, "GetStrategyState query failed")
	assert.NotNil(t, strategyStateResp, "response is nil")
	t.Logf("Strategy State: %+v", strategyStateResp)

	resp, err := strategyTVLLimits.SharesToUnderlyingView(1000000)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyTVLLimits.UnderlyingToShareView(1000000)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	underlyingViewResp, err := strategyTVLLimits.UnderlyingView("bbn1huw8yau3aqdsp9lr2f85v5plfd46tu026wylaj")
	assert.NoError(t, err, "UnderlyingView failed")
	assert.NotNil(t, underlyingViewResp, "response is nil")
	t.Logf("Underlying View: %+v", underlyingViewResp)

	underlyingTokenResp, err := strategyTVLLimits.UnderlyingToken()
	assert.NoError(t, err, "UnderlyingToken query failed")
	assert.NotNil(t, underlyingTokenResp, "response is nil")
	t.Logf("Underlying Token: %+v", underlyingTokenResp)

	resp, err = strategyTVLLimits.GetShares("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk", "bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	underlyingToShareResp, err := strategyTVLLimits.UnderlyingToShares("1000000")
	assert.NoError(t, err, "UnderlyingToShare failed")
	assert.NotNil(t, underlyingToShareResp, "response is nil")
	t.Logf("UnderlyingToShare Response: %+v", underlyingToShareResp)

	strategyManagerResp, err := strategyTVLLimits.GetStrategyManager()
	assert.NoError(t, err, "GetStrategyManager failed")
	assert.NotNil(t, strategyManagerResp, "response is nil")
	t.Logf("Strategy Manager: %+v", strategyManagerResp)
}

func TestStrategyBaseTVLLimitsTestSuite(t *testing.T) {
	suite.Run(t, new(strategyBaseTVLLimitsTestSuite))
}
