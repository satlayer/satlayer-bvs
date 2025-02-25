package e2e

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type strategyFactoryTestSuite struct {
	suite.Suite
	chainIO   io.ChainIO
	contrAddr string
	container *babylond.BabylonContainer
	deployer  *bvs.Deployer
}

func (suite *strategyFactoryTestSuite) SetupSuite() {
	suite.container = babylond.Run(context.Background())
	suite.chainIO = suite.container.NewChainIO("../.babylon")

	// Import And Fund Caller
	suite.container.ImportPrivKey("strategy-factory:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	suite.container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	// Deployment
	suite.deployer = &bvs.Deployer{BabylonContainer: suite.container}
	tAddr := suite.container.GenerateAddress("test-address").String()
	strategyFactory := suite.deployer.DeployStrategyFactory(tAddr, 1)
	suite.contrAddr = strategyFactory.Address
}

func (suite *strategyFactoryTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *strategyFactoryTestSuite) test_DeployNewStrategy() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(400000)

	token := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	pauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	unpauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.DeployNewStrategy(context.Background(), token, pauser, unpauser)
	assert.NoError(t, err, "DeployNewStrategy failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetThirdPartyTransfersForbidden() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	// Setup StrategyManager
	tAddr := suite.container.GenerateAddress("test-address").String()
	suite.container.ImportPrivKey("strategy-manager:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	strategyManager := suite.deployer.DeployStrategyManager(tAddr, tAddr, suite.contrAddr, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	_, err = factoryApi.SetStrategyManager(context.Background(), strategyManager.Address)
	assert.NoError(t, err)

	txResp, err := factoryApi.SetThirdPartyTransfersForbidden(context.Background(), strategyManager.Address, true)
	assert.NoError(t, err, "SetThirdPartyTransfersForbidden failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_UpdateConfig() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newOwner := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyCodeID := int64(10995)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	newOwner := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := factoryApi.TwoStepTransferOwnership(context.Background(), newOwner)
	assert.NoError(t, err, "TransferOwnership failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *strategyFactoryTestSuite) Test_SetPauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	{
		// Setup Pauser
		_, err := factoryApi.SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		suite.Require().NoError(err)
	}

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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)
	factoryApi.WithGasLimit(300000)

	{
		// Setup Pauser
		_, err := factoryApi.SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		suite.Require().NoError(err)
	}

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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)

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

	factoryApi := api.NewStrategyFactory(chainIO, suite.contrAddr)

	token := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	resp, err := factoryApi.IsTokenBlacklisted(token)
	assert.NoError(t, err, "IsTokenBlacklisted query failed")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsTokenBlacklisted Response: %+v", resp)
}

func TestStrategyFactoryTestSuite(t *testing.T) {
	suite.Run(t, new(strategyFactoryTestSuite))
}
