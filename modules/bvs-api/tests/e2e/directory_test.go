package e2e

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/directory"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
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

	// Import And Fund Caller
	container.ImportPrivKey("owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.ImportPrivKey("owner:replaced", "4D895710FBC2F9B50239FEFBD0747CED0A1C10AEBEEAA21044BAF36244888D2B")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
	container.FundAddressUbbn("bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv", 1e7)
	container.FundAddressUbbn("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", 1e8)
	container.FundAddressUbbn("bbn1fd9kt5nmzd6jxwecemuad4pyg3hhefd8hxuhnz", 1e8)

	s.chainIO = container.NewChainIO("../.babylon")
	s.container = container

	tAddr := container.GenerateAddress("test-address").String()
	deployer := &bvs.Deployer{BabylonContainer: container}
	pauser := deployer.DeployPauser(nil)

	// Setup DelegationManager,
	// Setup StrategyManager,
	// Add Operator to DelegationManager
	strategyManager := deployer.DeployStrategyManager(pauser.Address)
	delegationManager := deployer.DeployDelegationManager(pauser.Address, 100, []string{tAddr}, []int64{50})

	s.contrAddr = deployer.DeployDirectory(pauser.Address).Address
	s.delegationContrAddr = delegationManager.Address

	chainIO, err := s.chainIO.SetupKeyring("caller", "test")
	delegationApi := api.NewDelegationManager(chainIO, delegationManager.Address)
	txResp, err := delegationApi.SetRouting(context.Background(), strategyManager.Address, tAddr)
	s.Require().NoError(err)
	s.Require().Equal(uint32(0), txResp.TxResult.Code)

	chainIO, err = s.chainIO.SetupKeyring("operator1", "test")
	delegationApi = api.NewDelegationManager(chainIO, delegationManager.Address)
	txResp, err = delegationApi.RegisterAsOperator(
		context.Background(),
		"",
		0,
	)
	s.Require().NoError(err, "register as operator")
	s.Require().NotNil(txResp, "response nil")

	chainIO, err = s.chainIO.SetupKeyring("caller", "test")
	s.Require().NoError(err)
	txResp, err = api.NewDirectory(chainIO, s.contrAddr).SetRouting(context.Background(), s.delegationContrAddr)
	s.Require().NoError(err)
	s.Require().Equal(uint32(0), txResp.TxResult.Code)
}

func (s *DirectoryTestSuite) TearDownSuite() {
	s.Require().NoError(s.container.Terminate(context.Background()))
}

func (s *DirectoryTestSuite) Test_ServiceRegister() {
	chainIO, err := s.chainIO.SetupKeyring("operator3", "test")
	s.NoError(err)
	directoryApi := api.NewDirectory(chainIO, s.contrAddr)

	txResp, err := directoryApi.ServiceRegister(
		context.Background(),
		directory.ServiceMetadata{},
	)
	s.NoError(err)
	s.NotNil(txResp)
}

func (s *DirectoryTestSuite) Test_RegisterOperatorAndDeregisterOperator() {
	// Operator (Already Registered)
	chainIO, err := s.chainIO.SetupKeyring("operator1", "test")
	s.NoError(err)
	operatorAddr := chainIO.GetClientCtx().GetFromAddress()
	operatorClient := api.NewDirectory(chainIO, s.contrAddr).WithGasLimit(500000)

	// Service (BVS)
	chainIO, err = s.chainIO.SetupKeyring("caller", "test")
	s.NoError(err)
	serviceAddr := chainIO.GetClientCtx().GetFromAddress()
	serviceClient := api.NewDirectory(chainIO, s.contrAddr).WithGasLimit(500000)

	status, _ := serviceClient.QueryStatus(operatorAddr.String(), serviceAddr.String())
	s.Equal(directory.StatusResponse(0), *status)

	res, err := serviceClient.ServiceRegister(context.Background(), directory.ServiceMetadata{})
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)
	status, _ = serviceClient.QueryStatus(operatorAddr.String(), serviceAddr.String())
	s.Equal(directory.StatusResponse(0), *status)

	// Register Operator to Service
	res, err = operatorClient.OperatorRegisterService(context.Background(), serviceAddr.String())
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	status, _ = serviceClient.QueryStatus(operatorAddr.String(), serviceAddr.String())
	s.Equal(directory.StatusResponse(2), *status)

	// Register Service to Operator
	res, err = serviceClient.ServiceRegisterOperator(context.Background(), operatorAddr.String())
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	status, _ = serviceClient.QueryStatus(operatorAddr.String(), serviceAddr.String())
	s.Equal(directory.StatusResponse(1), *status)
}

func (s *DirectoryTestSuite) Test_UpdateMetadataURI() {
	t := s.T()
	keyName := "caller"
	chainIO, err := s.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(s.T(), err)

	name := "Example BVS"
	uri := "https://example.com"
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).ServiceUpdateMetadata(context.Background(), directory.ServiceMetadata{
		Name: &name,
		URI:  &uri,
	})
	assert.NoError(t, err, "update metadata uri")
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

func (s *DirectoryTestSuite) Test_SetRouting() {
	t := s.T()
	chainIO, err := s.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)
	txResp, err := api.NewDirectory(chainIO, s.contrAddr).SetRouting(context.Background(), s.delegationContrAddr)
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (s *DirectoryTestSuite) Test_QueryStatus() {
	chainIO, err := s.chainIO.SetupKeyring("caller", "test")
	s.Require().NoError(err)
	directoryApi := api.NewDirectory(chainIO, s.contrAddr)

	status, err := directoryApi.QueryStatus("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	s.NoError(err)
	s.Equal(directory.StatusResponse(0), *status)
}

func TestDirectory(t *testing.T) {
	suite.Run(t, new(DirectoryTestSuite))
}
