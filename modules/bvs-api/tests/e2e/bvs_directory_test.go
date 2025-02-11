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
	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

type bvsDirectoryTestSuite struct {
	suite.Suite
	chainIO             io.ChainIO
	chainID             string
	contrAddr           string
	delegationContrAddr string
}

func (suite *bvsDirectoryTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "bvs_directory")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    15 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.chainID = chainID
	suite.contrAddr = "bbn1fysu8t36542tpegm8msl6enez2f56gtvzry698lkf9wpkf7cr89svh2t0q"
	suite.delegationContrAddr = "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"
}

func (suite *bvsDirectoryTestSuite) Test_RegisterBVS() {
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(suite.T(), err)
	bvsApi := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr)

	isAccountOnChain := chainIO.IsCurrentAccountOnChain()
	suite.T().Logf("isAccountOnChain:%v", isAccountOnChain)
	assert.True(suite.T(), isAccountOnChain, "Account %s (%s) is not on chain", keyName, chainIO.GetClientCtx().GetFromAddress())

	txResp, err := bvsApi.RegisterBVS(
		context.Background(),
		"bbn1mzq6xzynh002x090rzt6us37scfexpu8ca4sllc3p3scn5mvsp0q5cs73s",
		"babylon",
		suite.chainID,
	)
	assert.NoError(suite.T(), err, "TestRegisterBVS")
	assert.NotNil(suite.T(), txResp, "response nil")
	suite.T().Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_RegisterOperatorAndDeregisterOperator() {
	t := suite.T()
	keyName := "operator1"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err)
	bvsApi := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr)
	bvsApi.WithGasLimit(500000)

	pubKey := chainIO.GetCurrentAccountPubKey()

	registerResp, err := bvsApi.RegisterOperator(context.Background(), account.GetAddress().String(), pubKey)
	assert.NoError(t, err, "register operator")
	assert.NotNil(t, registerResp, "response nil")
	t.Logf("registerResp:%+v", registerResp)

	// repeat register operator failed
	registerResp, err = bvsApi.RegisterOperator(context.Background(), account.GetAddress().String(), pubKey)
	assert.Error(t, err, "register operator not failed")
	assert.Nil(t, registerResp, "response not nil")

	deregisterResp, err := bvsApi.DeregisterOperator(context.Background(), account.GetAddress().String())
	assert.NoError(t, err, "deregister operator")
	assert.NotNil(t, deregisterResp, "response nil")
	t.Logf("deregisterResp:%+v", deregisterResp)
}

func (suite *bvsDirectoryTestSuite) Test_UpdateMetadataURI() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(suite.T(), err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).UpdateMetadataURI(context.Background(), "test.cn")
	assert.NoError(t, err, "update metadata uri")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_CancelSalt() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(suite.T(), err)
	randomStr, err := utils.GenerateRandomString(16)
	assert.NoError(suite.T(), err)
	salt := "salt" + randomStr
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).CancelSalt(context.Background(), salt)
	assert.NoError(t, err, "TestCancelSalt")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_TransferOwnership() {
	t := suite.T()
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)

	updateResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err, "TestTransferOwnership")
	assert.NotNil(t, updateResp, "response nil")
	t.Logf("updateResp:%+v", updateResp)

	// not owner to transfer ownership will be failed
	recoverClient, err := suite.chainIO.SetupKeyring("aggregator", "test")
	assert.NoError(t, err, "create client")
	recoverResp, err := api.NewBVSDirectoryImpl(recoverClient, suite.contrAddr).TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.Error(t, err, "TestTransferOwnership failed")
	assert.Nil(t, recoverResp, "response nil")
}

func (suite *bvsDirectoryTestSuite) Test_Pause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).Pause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

	recoverResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).Unpause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, recoverResp, "response nil")
	t.Logf("txResp:%+v", recoverResp)
}

func (suite *bvsDirectoryTestSuite) Test_SetPauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_SetUnpauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_SetDelegationManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).SetDelegationManager(context.Background(), suite.delegationContrAddr)
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_QueryOperator() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).QueryOperator("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "TestQueryOperator")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp.Status)
}

func (suite *bvsDirectoryTestSuite) Test_CalculateDigestHash() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err, "query node status")
	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
	randomStr, err := utils.GenerateRandomString(16)
	assert.NoError(suite.T(), err)
	salt := "salt" + randomStr
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")
	msgHashResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).CalculateDigestHash(
		account.GetPubKey(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		salt,
		expiry,
	)
	assert.NoError(t, err, "TestCalculateDigestHash")
	assert.NotNil(t, msgHashResp, "response nil")
	t.Logf("msgHashResp:%+v", msgHashResp)
}

func (suite *bvsDirectoryTestSuite) Test_IsSaltSpent() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).IsSaltSpent("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "c2FsdDEyMzE4NTgyNzI1NDIyMDc5NDI4")
	assert.NoError(t, err, "TestIsSaltSpent")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_GetDelegationManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetDelegationManager()
	assert.NoError(t, err, "get delegation manager")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

}

func (suite *bvsDirectoryTestSuite) Test_GetOwner() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetOwner()
	assert.NoError(t, err, "GetOwner")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_GetOperatorBVSRegistrationTypeHash() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetOperatorBVSRegistrationTypeHash()
	assert.NoError(t, err, "get operator bvs registration type hash")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_GetDomainTypeHash() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetDomainTypeHash()
	assert.NoError(t, err, "TestGetDomainTypeHash")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) Test_GetDomainName() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetDomainName()
	assert.NoError(t, err, "GetDomainName")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *bvsDirectoryTestSuite) test_GetBVSInfo() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetBVSInfo("af0a809a3b8de8656c9b1af0389174a0ee49bf7094b84102171d4fe9f1d24770")
	assert.NoError(t, err, "TestGetBVSInfo")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

	// get non exist bvsHash will nil
	txResp, err = api.NewBVSDirectoryImpl(chainIO, suite.contrAddr).GetBVSInfo("d2f665ee5cbc6e4d2fa992defb065fb1d51539db35654ed28feae24fcfa7cdbf")
	assert.Error(t, err, "TestGetBVSInfo not failed")
	assert.Nil(t, txResp, "response not nil")
	t.Logf("Get not exist bvsHash txResp:%+v", txResp)
}

func TestBVSDirectoryTestSuite(t *testing.T) {
	suite.Run(t, new(bvsDirectoryTestSuite))
}
