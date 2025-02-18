package e2e

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond/cw20"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type strategyBaseTestSuite struct {
	suite.Suite
	chainIO             io.ChainIO
	strategyBaseAddr    string
	strategyManagerAddr string
}

func (suite *strategyBaseTestSuite) SetupTest() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")

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

	container.ImportPrivKey("strategy-base:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	// TODO: Circular Deps, StrategyManager should be set via ExecuteMsg and not injected in InitMsg
	strategyBase := container.DeployStrategyBase(token.Address, "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy")

	suite.strategyBaseAddr = strategyBase.Address
	suite.strategyManagerAddr = container.GenerateAddress("throw-away").String()

}

func (suite *strategyBaseTestSuite) Test_ExecuteStrategyBase() {
	t := suite.T()
	keyName := "caller"

	t.Logf("TestExecuteSquaring")
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(suite.strategyBaseAddr)

	resp, err := strategyBase.SetStrategyManager(context.Background(), suite.strategyManagerAddr)
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
	keyName := "wallet1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyBase := api.NewStrategyBase(chainIO)
	strategyBase.BindClient(suite.strategyBaseAddr)

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
