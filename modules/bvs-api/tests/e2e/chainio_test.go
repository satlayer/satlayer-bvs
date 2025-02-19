package e2e

import (
	"context"
	"encoding/json"
	"testing"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
)

type ioTestSuite struct {
	suite.Suite
	chainIO   io.ChainIO
	directory *bvs.Contract[directory.InstantiateMsg]
	container *babylond.BabylonContainer
}

func (suite *ioTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	deployer := &bvs.Deployer{BabylonContainer: container}
	tAddr := container.GenerateAddress("throw-away")
	suite.directory = deployer.DeployDirectory(tAddr.String())
}

func (suite *ioTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *ioTestSuite) Test_QueryContract() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")
	queryMsg, err := json.Marshal(types.GetOperatorStatusReq{GetOperatorStatus: types.GetOperatorStatus{
		Operator: account.GetAddress().String(),
		BVS:      account.GetAddress().String(),
	}})
	assert.NoError(t, err, "marshal query msg")
	QueryOptions := types.QueryOptions{
		ContractAddr: suite.directory.Address,
		QueryMsg:     queryMsg,
	}
	resp, err := chainIO.QueryContract(QueryOptions)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
}

func (suite *ioTestSuite) Test_QueryTransaction() {
	t := suite.T()
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)

	executeMsgBytes, _ := json.Marshal(&types.UpdateMetadataURIReq{UpdateMetadataURI: types.UpdateMetadataURI{MetadataURI: "example.com"}})
	assert.NoError(t, err, "marshal execute msg")
	executeOptions := types.ExecuteOptions{
		ContractAddr:  suite.directory.Address,
		ExecuteMsg:    executeMsgBytes,
		Funds:         "",
		GasAdjustment: 1.2,
		GasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		Gas:           200000,
		Memo:          "test query transaction tx",
		Simulate:      true,
	}
	transactionResp, err := chainIO.SendTransaction(context.TODO(), executeOptions)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, transactionResp, "response nil")

	resp, err := chainIO.QueryTransaction(transactionResp.Hash.String())
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	// not exist hash will be failed
	resp, err = chainIO.QueryTransaction("1638873FAC7918A6882CF4D556339286EC8D46D4792B0F9AB5CBECB3BE2AD7E0")
	assert.Error(t, err, "get not exist transaction failed")
	assert.Nil(t, resp, "response not nil")
	t.Logf("resp:%+v", resp)
}

func (suite *ioTestSuite) Test_QueryAccount() {
	t := suite.T()
	resp, err := suite.chainIO.QueryAccount("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	assert.Nil(t, resp.GetPubKey(), "response nil")
	//t.Logf("resp:%s", base64.StdEncoding.EncodeToString(resp.GetPubKey().Bytes()))

	// query not exist address will be failed
	resp, err = suite.chainIO.QueryAccount("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredc")
	assert.Error(t, err, "get not exist address failed")
	assert.Nil(t, resp, "response not nil")
	t.Logf("not exist address resp:%s", resp)
}

func (suite *ioTestSuite) test_GetCurrentAccount() {
	t := suite.T()
	keyName := "coswallet"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "os", "babylon")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")
	t.Log(account.GetAddress().String())
	assert.Equal(t, account.GetAddress().String(), "bbn15t8rczqcyfxk4vjkrjx6cjd4vmw804ufu0tmzx")
}

func TestIoTestSuite(t *testing.T) {
	suite.Run(t, new(ioTestSuite))
}
