package e2e

import (
	"context"
	"math/big"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type strategyBaseTVLLimitsTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	contrAddr       string
	strategyManager string
	container       *babylond.BabylonContainer
}

func (suite *strategyBaseTVLLimitsTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	suite.container = container
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	deployer := &bvs.Deployer{BabylonContainer: container}

	minter := container.GenerateAddress("cw20:minter")
	token := cw20.DeployCw20(container, cw20.InstantiateMsg{
		Decimals: 6,
		InitialBalances: []cw20.Cw20Coin{
			{
				Address: minter.String(),
				Amount:  "1000000000",
			},
		},
		Mint: &cw20.MinterResponse{
			Minter: minter.String(),
		},
		Name:   "Test Token",
		Symbol: "TEST",
	})
	tAddr := container.GenerateAddress("test-address").String()

	strategyManager := deployer.DeployStrategyManager(tAddr, tAddr, tAddr, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	container.ImportPrivKey("strategy-base-tvl-limits:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	suite.contrAddr = deployer.DeployStrategyBaseTvlLimits(strategyManager.Address, token.Address, big.NewInt(1e8), big.NewInt(1e8)).Address
	suite.strategyManager = strategyManager.Address
}

func (suite *strategyBaseTVLLimitsTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *strategyBaseTVLLimitsTestSuite) Test_ExecuteStrategyTVLLimits() {
	t := suite.T()
	keyName := "caller"

	t.Logf("TestExecuteStrategyTVLLimits")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyTVLLimits := api.NewStrategyBaseTVLLimits(chainIO)
	strategyTVLLimits.BindClient(suite.contrAddr)

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

	// resp, err := strategyTVLLimits.SetTvlLimits(context.Background(), "11000000", "13000000")
	// assert.NoError(t, err, "SetTvlLimits failed")
	// assert.NotNil(t, resp, "response is nil")
	// t.Logf("SetTvlLimits Response: %+v", resp)

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
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyTVLLimits := api.NewStrategyBaseTVLLimits(chainIO)
	strategyTVLLimits.BindClient(suite.contrAddr)

	tvlLimitsResp, err := strategyTVLLimits.GetTvlLimits()
	assert.NoError(t, err, "GetTvlLimits failed")
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
