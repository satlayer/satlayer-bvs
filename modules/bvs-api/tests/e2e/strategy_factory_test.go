package e2e

import (
	"context"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-api/chainio/api"
	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-api/logger"
	transactionprocess "github.com/satlayer/satlayer-api/metrics/indicators/transaction_process"
)

type strategyFactoryTestSuite struct {
	suite.Suite
	chainIO   io.ChainIO
	contrAddr string
}

func (suite *strategyFactoryTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "strategy_factory")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.contrAddr = "bbn18h8ncg9szj3v92cz289qz3ndwqk5zema4cr2t4e7amjaehrae52qyw07y9"
}

func (suite *strategyFactoryTestSuite) test_DeployNewStrategy() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(400000)

	token := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	pauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	unpauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.DeployNewStrategy(context.Background(), token, pauser, unpauser)
	assert.NoError(t, err, "DeployNewStrategy failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetThirdPartyTransfersForBidden() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	strategy := "bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"
	txResp, err := factoryApi.SetThirdPartyTransfersForBidden(context.Background(), strategy, true)
	assert.NoError(t, err, "SetThirdPartyTransfersForBidden failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_UpdateConfig() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newOwner := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyCodeID := uint64(10995)
	txResp, err := factoryApi.UpdateConfig(context.Background(), newOwner, strategyCodeID)
	assert.NoError(t, err, "UpdateConfig failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) test_BlacklistTokens() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	tokens := []string{"bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"}
	txResp, err := factoryApi.BlacklistTokens(context.Background(), tokens)
	assert.NoError(t, err, "BlacklistTokens failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) test_RemoveStrategiesFromWhitelist() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	strategies := []string{"bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"}
	txResp, err := factoryApi.RemoveStrategiesFromWhitelist(context.Background(), strategies)
	assert.NoError(t, err, "RemoveStrategiesFromWhitelist failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) test_WhitelistStrategies() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	strategiesToWhitelist := []string{"bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"}
	thirdPartyTransfersForbiddenValues := []bool{true}
	txResp, err := factoryApi.WhitelistStrategies(context.Background(), strategiesToWhitelist, thirdPartyTransfersForbiddenValues)
	assert.NoError(t, err, "WhitelistStrategies failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetStrategyManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newManager := "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"
	txResp, err := factoryApi.SetStrategyManager(context.Background(), newManager)
	assert.NoError(t, err, "SetStrategyManager failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_TransferOwnership() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newOwner := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.TransferOwnership(context.Background(), newOwner)
	assert.NoError(t, err, "TransferOwnership failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetPauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newPauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.SetPauser(context.Background(), newPauser)
	assert.NoError(t, err, "SetPauser failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_Pause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	txResp, err := factoryApi.Pause(context.Background())
	assert.NoError(t, err, "Pause failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetUnpauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newUnpauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.SetUnpauser(context.Background(), newUnpauser)
	assert.NoError(t, err, "SetUnpauser failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_Unpause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	txResp, err := factoryApi.Unpause(context.Background())
	assert.NoError(t, err, "Unpause failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) test_Query_GetStrategy() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)

	token := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	resp, err := factoryApi.GetStrategy(token)
	assert.NoError(t, err, "GetStrategy query failed")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStrategy Response: %+v", resp)
}

func (suite *strategyFactoryTestSuite) Test_Query_IsTokenBlacklisted() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactoryImpl(chainIO, suite.contrAddr)
	factoryApi.BindClient(suite.contrAddr)

	token := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	resp, err := factoryApi.IsTokenBlacklisted(token)
	assert.NoError(t, err, "IsTokenBlacklisted query failed")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsTokenBlacklisted Response: %+v", resp)
}

func TestStrategyFactoryTestSuite(t *testing.T) {
	suite.Run(t, new(strategyFactoryTestSuite))
}
