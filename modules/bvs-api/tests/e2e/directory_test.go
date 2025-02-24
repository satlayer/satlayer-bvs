package e2e

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

type DirectoryTestSuite struct {
	suite.Suite
	chainIO             io.ChainIO
	contrAddr           string
	delegationContrAddr string
	container           *babylond.BabylonContainer
}

func (s *DirectoryTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	s.chainIO = container.NewChainIO("../.babylon")
	s.container = container

	// Import And Fund Caller
	container.ImportPrivKey("directory:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.ImportPrivKey("directory:initial_owner:replaced", "4D895710FBC2F9B50239FEFBD0747CED0A1C10AEBEEAA21044BAF36244888D2B")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
	container.FundAddressUbbn("bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv", 1e7)

	tAddr := container.GenerateAddress("test-address").String()
	deployer := &bvs.Deployer{BabylonContainer: container}

	// Setup DelegationManager,
	// Setup StrategyManager,
	// Add Operator to DelegationManager
	s.container.FundAddressUbbn("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", 1e8)

	strategyManager := deployer.DeployStrategyManager(tAddr, tAddr, tAddr, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")

	delegationManager := deployer.DeployDelegationManager(
		tAddr, strategyManager.Address, 100, []string{tAddr}, []int64{50},
	)

	chainIO, err := s.chainIO.SetupKeyring("operator1", "test")
	delegationApi := api.NewDelegationManager(chainIO, delegationManager.Address)
	s.Require().NoError(err, "setup keyring")
	accountPubKey := GetPubKeyFromKeychainByUid(chainIO, "operator1")

	txResp, err := delegationApi.RegisterAsOperator(
		context.Background(),
		accountPubKey,
		"",
		"0",
		"",
		0,
	)
	s.Require().NoError(err, "register as operator")
	s.Require().NotNil(txResp, "response nil")

	s.contrAddr = deployer.DeployDirectory(delegationManager.Address).Address
	s.delegationContrAddr = delegationManager.Address
}

func (s *DirectoryTestSuite) TearDownSuite() {
	s.Require().NoError(s.container.Terminate(context.Background()))
}

func (s *DirectoryTestSuite) test_RegisterBvs() {
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(s.T(), err)
	bvsApi := api.NewDirectory(chainIO, s.contrAddr)

	isAccountOnChain := chainIO.IsCurrentAccountOnChain()
	s.T().Logf("isAccountOnChain:%v", isAccountOnChain)
	assert.True(s.T(), isAccountOnChain, "Account %s (%s) is not on chain", keyName, chainIO.GetClientCtx().GetFromAddress())

	txResp, err := bvsApi.RegisterBvs(
		context.Background(),
		"bbn1mzq6xzynh002x090rzt6us37scfexpu8ca4sllc3p3scn5mvsp0q5cs73s",
	)
	assert.NoError(s.T(), err, "TestRegisterBvs")
	assert.NotNil(s.T(), txResp, "response nil")
	s.T().Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_RegisterOperatorAndDeregisterOperator() {
	t := s.T()
	keyName := "operator1"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	s.container.FundAddressUbbn("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", 1e8)

	operatorKey, err := chainIO.GetClientCtx().Keyring.Key("operator1")
	assert.NoError(t, err)
	operatorAddr, err := operatorKey.GetAddress()
	assert.NoError(t, err)
	operatorPubKey, err := operatorKey.GetPubKey()
	assert.NoError(t, err)

	bvsApi := api.NewDirectory(chainIO, s.contrAddr)
	bvsApi.WithGasLimit(500000)
	registerResp, err := bvsApi.RegisterOperator(context.Background(), operatorAddr.String(), operatorPubKey)
	assert.NoError(t, err, "register operator")
	assert.NotNil(t, registerResp, "response nil")
	t.Logf("registerResp:%+v", registerResp)

	// repeat register operator failed
	registerResp, err = bvsApi.RegisterOperator(context.Background(), operatorAddr.String(), operatorPubKey)
	assert.Error(t, err, "register operator not failed")
	assert.Nil(t, registerResp, "response not nil")

	deregisterResp, err := bvsApi.DeregisterOperator(context.Background(), operatorAddr.String())
	assert.NoError(t, err, "deregister operator")
	assert.NotNil(t, deregisterResp, "response nil")
	t.Logf("deregisterResp:%+v", deregisterResp)
}

func (s *DirectoryTestSuite) Test_UpdateMetadataURI() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(s.T(), err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).UpdateMetadataURI(context.Background(), "example.com")
	assert.NoError(t, err, "update metadata uri")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_CancelSalt() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(s.T(), err)
	randomStr, err := utils.GenerateRandomString(16)
	assert.NoError(s.T(), err)
	salt := "salt" + randomStr
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).CancelSalt(context.Background(), salt)
	assert.NoError(t, err, "TestCancelSalt")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_TransferOwnership() {
	t := s.T()
	chainIO, err := s.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)

	updateResp, err := api.NewDirectory(chainIO, s.contrAddr).TransferOwnership(context.Background(), "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.NoError(t, err, "TestTransferOwnership")
	assert.NotNil(t, updateResp, "response nil")
	t.Logf("updateResp:%+v", updateResp)

	// not owner to transfer ownership will be failed
	updateResp, err = api.NewDirectory(chainIO, s.contrAddr).TransferOwnership(context.Background(), "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.Error(t, err, "TestTransferOwnership not failed")
	assert.Nil(t, updateResp, "response not nil")
	t.Logf("TransferOwnership failed Resp:%+v", updateResp)

	recoverClient, err := s.chainIO.SetupKeyring("aggregator", "test")
	assert.NoError(t, err, "create client")
	recoverResp, err := api.NewDirectory(recoverClient, s.contrAddr).TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err, "TestTransferOwnership")
	assert.NotNil(t, recoverResp, "response nil")
	t.Logf("recoverResp:%+v", recoverResp)
}

func (s *DirectoryTestSuite) Test_Pause() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	bvsDirectory := api.NewDirectory(chainIO, s.contrAddr)
	{
		_, err = bvsDirectory.SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		s.Require().NoError(err)
		_, err = bvsDirectory.SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		s.Require().NoError(err)
	}

	txResp, err := bvsDirectory.Pause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

	recoverResp, err := bvsDirectory.Unpause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, recoverResp, "response nil")
	t.Logf("txResp:%+v", recoverResp)
}

func (s *DirectoryTestSuite) Test_SetPauser() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_SetUnpauser() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_SetDelegationManager() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).SetDelegationManager(context.Background(), s.delegationContrAddr)
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_QueryOperator() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).QueryOperator("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "TestQueryOperator")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp.Status)
}

func (s *DirectoryTestSuite) Test_CalculateDigestHash() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err, "query node status")
	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000
	randomStr, err := utils.GenerateRandomString(16)
	assert.NoError(t, err, "GenerateRandomString")
	salt := "salt" + randomStr

	key, err := chainIO.GetClientCtx().Keyring.Key("caller")
	assert.NoError(t, err, "get key")
	pubKey, err := key.GetPubKey()
	assert.NoError(t, err, "get account")
	msgHashResp, err := api.NewDirectory(chainIO, s.contrAddr).CalculateDigestHash(
		pubKey,
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		salt,
		expiry,
	)
	assert.NoError(t, err, "TestCalculateDigestHash")
	assert.NotNil(t, msgHashResp, "response nil")
	t.Logf("msgHashResp:%+v", msgHashResp)
}

func (s *DirectoryTestSuite) Test_IsSaltSpent() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).IsSaltSpent("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "c2FsdDEyMzE4NTgyNzI1NDIyMDc5NDI4")
	assert.NoError(t, err, "TestIsSaltSpent")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_GetDelegationManager() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetDelegationManager()
	assert.NoError(t, err, "get delegation manager")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

}

func (s *DirectoryTestSuite) Test_GetOwner() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetOwner()
	assert.NoError(t, err, "GetOwner")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_GetOperatorBvsRegistrationTypeHash() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetOperatorBvsRegistrationTypeHash()
	assert.NoError(t, err, "get operator bvs registration type hash")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_GetDomainTypeHash() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetDomainTypeHash()
	assert.NoError(t, err, "TestGetDomainTypeHash")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_GetDomainName() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetDomainName()
	assert.NoError(t, err, "GetDomainName")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) test_GetBvsInfo() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).GetBvsInfo("af0a809a3b8de8656c9b1af0389174a0ee49bf7094b84102171d4fe9f1d24770")
	assert.NoError(t, err, "TestGetBvsInfo")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

	// get non exist bvsHash will nil
	txResp, err = api.NewDirectory(chainIO, s.contrAddr).GetBvsInfo("d2f665ee5cbc6e4d2fa992defb065fb1d51539db35654ed28feae24fcfa7cdbf")
	assert.Error(t, err, "TestGetBvsInfo not failed")
	assert.Nil(t, txResp, "response not nil")
	t.Logf("Get not exist bvsHash txResp:%+v", txResp)
}

func TestBVSDirectoryTestSuite(t *testing.T) {
	suite.Run(t, new(DirectoryTestSuite))
}
