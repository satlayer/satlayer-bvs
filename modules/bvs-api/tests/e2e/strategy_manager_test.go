package e2e

import (
	"context"
	"testing"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

const managerAddr = "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"

const strategyAddr = "bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc"

const delegationAddr = "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"

const slashManagerAddr = "bbn1ht8js7p6y5jxthze8hy3egfxflh8t9mvkl79w75mg6atu4ssfc0ssk77p4"

const stakerAddr = "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
const tokenAddr = "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"

const keyName = "staker1"

const actualStakerKeyName = "staker2"

const ownerAddr = "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"

type strategyManagerTestSuite struct {
	suite.Suite
	chainIO     io.ChainIO
	container   *babylond.BabylonContainer
	managerAddr string
	tokenAddr   string
}

func (suite *strategyManagerTestSuite) SetupSuite() {
	suite.container = babylond.Run(context.Background())
	suite.container.ImportPrivKey("owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	suite.container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	suite.chainIO = suite.container.NewChainIO("../.babylon")

	deployer := &bvs.Deployer{BabylonContainer: suite.container}
	registry := deployer.DeployRegistry(nil)
	strategyManager := deployer.DeployStrategyManager(registry.Address)

	suite.managerAddr = strategyManager.Address

	minter := suite.container.GenerateAddress("cw20:minter")
	token := cw20.DeployCw20(suite.container, cw20.InstantiateMsg{
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
	suite.tokenAddr = token.Address
}

func (suite *strategyManagerTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *strategyManagerTestSuite) Test_Init() {
	t := suite.T()

	t.Logf("TestExecuteStrategyManager")
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)
	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(suite.managerAddr)

	resp, err := strategyManager.TransferOwnership(context.Background(), ownerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetRouting(context.Background(), delegationAddr, slashManagerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.UpdateStrategy(context.Background(), strategyAddr, true)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = api.IncreaseTokenAllowance(context.Background(), chainIO, 100, suite.tokenAddr, suite.managerAddr, sdktypes.NewInt64DecCoin("ubbn", 1))
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	/*resp, err = strategyManager.SetThirdPartyTransfersForbidden(context.Background(),strategyAddr, false)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	/*resp, err := strategyManager.DepositIntoStrategy(context.Background(),strategyAddr, tokenAddr, 30)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	/*account, err := chainIO.QueryAccount(actualStakerAddr)
	assert.NoError(t, err, "get account")
	assert.NoError(t, err, "get account", account)
	publicKey := account.GetPubKey()

	resp, err := strategyManager.DepositIntoStrategyWithSignature(context.Background(),strategyAddr, tokenAddr, 30, actualStakerAddr, publicKey, actualStakerKeyName)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	/*resp, err = strategyManager.RemoveShares(context.Background(),stakerAddr, strategyAddr, 10)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.WithdrawSharesAsTokens(context.Background(),stakerAddr, strategyAddr, 10, tokenAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.AddShares(context.Background(),stakerAddr, tokenAddr, strategyAddr, 10)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/
}

func (suite *strategyManagerTestSuite) test_ExecuteStrategyManager() {
	t := suite.T()

	t.Logf("TestExecuteStrategyManager")
	chainIO, err := suite.chainIO.SetupKeyring(actualStakerKeyName, "test")
	assert.NoError(t, err)
	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(managerAddr)

	/*resp, err := strategyManager.TransferOwnership(context.Background(),stakerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetDelegationManager(context.Background(),delegationAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetStrategyWhitelister(context.Background(),stakerAddr)
	assert.NoError(t, err, "SetStrategyWhitelister")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.RemoveStrategiesFromWhitelist(context.Background(),[]string{strategyAddr})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.UpdateStrategy(context.Background(),[]string{strategyAddr}, []bool{false})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetThirdPartyTransfersForbidden(context.Background(),strategyAddr, false)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	resp, err := strategyManager.DepositIntoStrategy(context.Background(), strategyAddr, tokenAddr, 1000000)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	/*account, err := chainIO.QueryAccount(actualStakerAddr)
	assert.NoError(t, err, "get account")
	assert.NoError(t, err, "get account", account)
	publicKey := account.GetPubKey()

	resp, err := strategyManager.DepositIntoStrategyWithSignature(context.Background(), strategyAddr, tokenAddr, 10, actualStakerAddr, publicKey, actualStakerKeyName)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)*/

	/*
		resp, err = strategyManager.RemoveShares(context.Background(),stakerAddr, strategyAddr, 10)
		assert.NoError(t, err, "execute contract")
		assert.NotNil(t, resp, "response nil")
		t.Logf("resp:%+v", resp)
		/

		resp, err = strategyManager.WithdrawSharesAsTokens(context.Background(),stakerAddr, strategyAddr, 10, tokenAddr)
		assert.NoError(t, err, "execute contract")
		assert.NotNil(t, resp, "response nil")
		t.Logf("resp:%+v", resp)
		/

		resp, err = strategyManager.AddShares(context.Background(),stakerAddr, tokenAddr, strategyAddr, 10)
		assert.NoError(t, err, "execute contract")
		assert.NotNil(t, resp, "response nil")
		t.Logf("resp:%+v", resp)*/
}

func (suite *strategyManagerTestSuite) test_QueryStrategyManager() {
	t := suite.T()

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(managerAddr)

	resp, err := strategyManager.GetDeposits(stakerAddr)
	assert.NoError(t, err, "GetDeposits")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDeposits resp:%+v", resp)

	resp, err = strategyManager.StakerStrategyListLength(stakerAddr)
	assert.NoError(t, err, "StakerStrategyListLength")
	assert.NotNil(t, resp, "response nil")
	t.Logf("StakerStrategyListLength resp:%+v", resp)

	resp, err = strategyManager.GetStakerStrategyShares(stakerAddr, strategyAddr)
	assert.NoError(t, err, "GetStakerStrategyShares")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStakerStrategyShares resp:%+v", resp)

	resp, err = strategyManager.GetStakerStrategyList(strategyAddr)
	assert.NoError(t, err, "GetStakerStrategyList")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStakerStrategyList resp:%+v", resp)

	resp, err = strategyManager.IsStrategyWhitelisted(strategyAddr)
	assert.NoError(t, err, "IsStrategyWhitelisted")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsStrategyWhitelisted resp:%+v", resp)

	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account", account)
}

func TestStrategyManagerTestSuite(t *testing.T) {
	suite.Run(t, new(strategyManagerTestSuite))
}
