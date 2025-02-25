package e2e

import (
	"context"
	"encoding/base64"
	"testing"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	strategymanager "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-manager"
)

const managerAddr = "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"

const strategyAddr = "bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc"

const delegationAddr = "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"

const slashManagerAddr = "bbn1ht8js7p6y5jxthze8hy3egfxflh8t9mvkl79w75mg6atu4ssfc0ssk77p4"

const factoryAddr = "bbn18h8ncg9szj3v92cz289qz3ndwqk5zema4cr2t4e7amjaehrae52qyw07y9"

const stakerAddr = "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
const tokenAddr = "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"

const homeDir = "../.babylon"
const keyName = "staker1"

const actualStakerAddr = "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
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
	suite.chainIO = suite.container.NewChainIO("../.babylon")

	deployer := &bvs.Deployer{BabylonContainer: suite.container}
	tAddr := suite.container.GenerateAddress("test-address").String()

	suite.container.ImportPrivKey("strategy-manager:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	strategyManager := deployer.DeployStrategyManager(tAddr, tAddr, tAddr, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")

	suite.managerAddr = strategyManager.Address
	suite.container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

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
	strategyManager := api.NewStrategyManager(chainIO, suite.managerAddr)

	resp, err := strategyManager.TwoStepTransferOwnership(context.Background(), ownerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetDelegationManager(context.Background(), delegationAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetSlashManager(context.Background(), slashManagerAddr)
	assert.NoError(t, err, "SetSlashManager")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetStrategyFactory(context.Background(), factoryAddr)
	assert.NoError(t, err, "SetStrategyFactory")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetStrategyWhitelister(context.Background(), ownerAddr)
	assert.NoError(t, err, "SetStrategyWhitelister")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.RemoveStrategiesFromWhitelist(context.Background(), []string{strategyAddr})
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.AddStrategiesToWhitelist(context.Background(), []string{strategyAddr}, []bool{true})
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

	/*resp, err := strategyManager.SetPauser(context.Background(),stakerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.Pause(context.Background())
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetUnpauser(context.Background(),stakerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.Unpause(context.Background())
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

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
	strategyManager := api.NewStrategyManager(chainIO, suite.managerAddr)

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

	resp, err = strategyManager.AddStrategiesToWhitelist(context.Background(),[]string{strategyAddr}, []bool{false})
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

	/*resp, err := strategyManager.SetPauser(context.Background(),stakerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.Pause(context.Background())
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.SetUnpauser(context.Background(),stakerAddr)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = strategyManager.Unpause(context.Background())
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

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

	strategyManager := api.NewStrategyManager(chainIO, suite.managerAddr)

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

	resp, err = strategyManager.IsThirdPartyTransfersForbidden(strategyAddr)
	assert.NoError(t, err, "IsThirdPartyTransfersForbidden")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsThirdPartyTransfersForbidden resp:%+v", resp)

	resp, err = strategyManager.GetNonce(stakerAddr)
	assert.NoError(t, err, "GetNonce")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetNonce resp:%+v", resp)

	resp, err = strategyManager.GetStakerStrategyList(strategyAddr)
	assert.NoError(t, err, "GetStakerStrategyList")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStakerStrategyList resp:%+v", resp)

	resp, err = strategyManager.Owner()
	assert.NoError(t, err, "Owner")
	assert.NotNil(t, resp, "response nil")
	t.Logf("Owner resp:%+v", resp)

	resp, err = strategyManager.IsStrategyWhitelisted(strategyAddr)
	assert.NoError(t, err, "IsStrategyWhitelisted")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsStrategyWhitelisted resp:%+v", resp)

	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account", account)
	staker := account.GetAddress().String()
	publicKey := account.GetPubKey()

	params := strategymanager.QueryDigestHashParams{
		Staker:       staker,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		Strategy:     strategyAddr,
		Token:        tokenAddr,
		Amount:       "10",
		Nonce:        1,
		Expiry:       1,
		ChainID:      "sat-bbn-localnet",
		ContractAddr: managerAddr,
	}

	resp, err = strategyManager.CalculateDigestHash(params)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("CalculateDigestHash resp:%+v", resp)

	resp, err = strategyManager.GetStrategyWhitelister()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStrategyWhitelister resp:%+v", resp)

	resp, err = strategyManager.GetStrategyManagerState()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetStrategyManagerState resp:%+v", resp)

	resp, err = strategyManager.GetDepositTypeHash()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDepositTypeHash resp:%+v", resp)

	resp, err = strategyManager.DomainTypeHash()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("DomainTypeHash resp:%+v", resp)

	resp, err = strategyManager.DomainName()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("DomainName resp:%+v", resp)

	resp, err = strategyManager.DelegationManager()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("DelegationManager resp:%+v", resp)
}

func TestStrategyManagerTestSuite(t *testing.T) {
	suite.Run(t, new(strategyManagerTestSuite))
}
