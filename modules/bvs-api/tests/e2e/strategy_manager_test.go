package e2e

import (
	"context"
	"encoding/base64"
	"testing"
	"time"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
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
	chainIO   io.ChainIO
	contrAddr string
}

func (suite *strategyManagerTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "state_manager")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
}

func (suite *strategyManagerTestSuite) Test_Init() {
	t := suite.T()

	t.Logf("TestExecuteStrategyManager")
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)
	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(managerAddr)

	resp, err := strategyManager.TransferOwnership(context.Background(), ownerAddr)
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

	resp, err = api.IncreaseTokenAllowance(context.Background(), chainIO, 100, tokenAddr, managerAddr, sdktypes.NewInt64DecCoin("ubbn", 1))
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

	strategyManager := api.NewStrategyManager(chainIO)
	strategyManager.BindClient(managerAddr)

	resp, err := strategyManager.GetDeposits(stakerAddr, strategyAddr)
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

	resp, err = strategyManager.GetOwner()
	assert.NoError(t, err, "GetOwner")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetOwner resp:%+v", resp)

	resp, err = strategyManager.IsStrategyWhitelisted(strategyAddr)
	assert.NoError(t, err, "IsStrategyWhitelisted")
	assert.NotNil(t, resp, "response nil")
	t.Logf("IsStrategyWhitelisted resp:%+v", resp)

	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account", account)
	staker := account.GetAddress().String()
	publicKey := account.GetPubKey()

	params := types.DigestHashParams{
		Staker:       staker,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		Strategy:     strategyAddr,
		Token:        tokenAddr,
		Amount:       "10",
		Nonce:        1,
		Expiry:       1,
		ChainId:      "sat-bbn-testnet1",
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

	resp, err = strategyManager.GetDepositTypehash()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDepositTypehash resp:%+v", resp)

	resp, err = strategyManager.GetDomainTypehash()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDomainTypehash resp:%+v", resp)

	resp, err = strategyManager.GetDomainName()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDomainName resp:%+v", resp)

	resp, err = strategyManager.GetDelegationManager()
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetDelegationManager resp:%+v", resp)
}

func TestStrategyManagerTestSuite(t *testing.T) {
	suite.Run(t, new(strategyManagerTestSuite))
}
