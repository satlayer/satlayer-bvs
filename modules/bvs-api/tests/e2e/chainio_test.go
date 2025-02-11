package e2e

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
)

type ioTestSuite struct {
	suite.Suite
	chainIO io.ChainIO
}

func (suite *ioTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    15 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
}

// --------- babylon ---------
func (suite *ioTestSuite) Test_QueryContract() {
	t := suite.T()
	bvsDirContrAddr := "bbn1f803xuwl6l7e8jm9ld0kynvvjfhfs5trax8hmrn4wtnztglpzw0sm72xua"
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
		ContractAddr: bvsDirContrAddr,
		QueryMsg:     queryMsg,
	}
	resp, err := chainIO.QueryContract(QueryOptions)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
}

func (suite *ioTestSuite) Test_QueryTransaction() {
	t := suite.T()
	bvsDirContrAddr := "bbn1f803xuwl6l7e8jm9ld0kynvvjfhfs5trax8hmrn4wtnztglpzw0sm72xua"
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	assert.NoError(t, err)

	executeMsgBytes, _ := json.Marshal(&types.UpdateMetadataURIReq{UpdateMetadataURI: types.UpdateMetadataURI{MetadataURI: "test.cn"}})
	assert.NoError(t, err, "marshal execute msg")
	executeOptions := types.ExecuteOptions{
		ContractAddr:  bvsDirContrAddr,
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
	t.Logf("resp:%s", base64.StdEncoding.EncodeToString(resp.GetPubKey().Bytes()))

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

// --------- eth ---------
type ethIOTestSuite struct {
	suite.Suite
	ethChainIO io.ETHChainIO
	abiPath    string
}

func (suite *ethIOTestSuite) SetupTest() {
	endpoint := "https://arb-sepolia.g.alchemy.com/v2/p2SMc5MIkqXr1zWVeTCCz7FHXf24-EZ2"
	keystorePath := "../.eth/keystore"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	ethChainIO, err := io.NewETHChainIO(endpoint, keystorePath, logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:                 3,
		RetryInterval:              2 * time.Second,
		ConfirmationTimeout:        60 * time.Second,
		ETHGasFeeCapAdjustmentRate: 2,
		ETHGasLimitAdjustmentRate:  1.1,
		GasLimit:                   1000000000,
	})
	suite.Require().NoError(err)
	suite.ethChainIO = ethChainIO
	suite.abiPath = "../../chainio/abi"
}

func (suite *ethIOTestSuite) Test_GetLatestBlockNumber() {
	block, err := suite.ethChainIO.GetLatestBlockNumber(context.Background())
	assert.NoError(suite.T(), err, "get latest block number")
	suite.T().Log(block)
}

func (suite *ethIOTestSuite) Test_GetChainID() {
	chainID, err := suite.ethChainIO.GetChainID(context.Background())
	assert.NoError(suite.T(), err, "get chain id")
	suite.T().Log(chainID)
}

func (suite *ethIOTestSuite) test_CreateAccount() {
	account, err := suite.ethChainIO.CreateAccount("123")
	assert.NoError(suite.T(), err, "create account")
	suite.T().Logf("address %s url %s", account.Address.String(), account.URL)
}

func (suite *ethIOTestSuite) test_ImportKey() {
	privateKeyHex := "xxxx"
	account, err := suite.ethChainIO.ImportKey(privateKeyHex, "123")
	assert.NoError(suite.T(), err, "import key")
	suite.T().Logf("address %s url %s", account.Address.String(), account.URL)
}

func (suite *ethIOTestSuite) Test_SignHash() {
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	ct, err := hex.DecodeString("638789a8cd83d13edab39fcf89b7044e693a5e96ee56348328f2405442ec6d09")
	assert.NoError(suite.T(), err)
	sig, err := suite.ethChainIO.SignHash(types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}, ct)
	assert.NoError(suite.T(), err)
	suite.T().Log(sig)
}

func (suite *ethIOTestSuite) Test_ListAccounts() {
	accounts := suite.ethChainIO.ListAccounts()
	for _, account := range accounts {
		suite.T().Logf("address %s url %s", account.Address.String(), account.URL)
	}
}

func (suite *ethIOTestSuite) Test_ExecuteContract() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	contractAddr := common.HexToAddress("0x1CCF4368c82DB1B9eD8daBC76D77D40F24466BA1")
	contractABI, err := abi.GetContractABI(suite.abiPath, "StateBank")
	assert.NoError(suite.T(), err, "get contract abi")

	var addressBytes [20]byte
	_, err = rand.Read(addressBytes[:])
	assert.NoError(suite.T(), err)
	addr := common.BytesToAddress(addressBytes[:])

	resp, err := suite.ethChainIO.ExecuteContract(ctx, types.ETHExecuteOptions{
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: contractAddr,
			ContractABI:  contractABI,
			Method:       "addRegisteredBvsContract",
			Args:         []interface{}{addr},
		},
		ETHWallet: types.ETHWallet{
			FromAddr: fromAddr,
			PWD:      pwd,
		},
	})
	assert.NoError(suite.T(), err, "execute contract")
	suite.T().Logf("resp:%s", resp.Hash())
}

func (suite *ethIOTestSuite) Test_CallContract() {
	ctx := context.Background()
	contractAddr := common.HexToAddress("0x1CCF4368c82DB1B9eD8daBC76D77D40F24466BA1")
	contractABI, err := abi.GetContractABI(suite.abiPath, "StateBank")
	assert.NoError(suite.T(), err, "get contract abi")
	var testVal interface{}
	err = suite.ethChainIO.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: contractAddr,
		ContractABI:  contractABI,
		Method:       "get",
		Args:         []interface{}{"testkey"},
	}, &testVal)
	assert.NoError(suite.T(), err, "execute contract")
	suite.T().Logf("resp:%+v", testVal)
}

func TestIoTestSuite(t *testing.T) {
	suite.Run(t, new(ioTestSuite))
	suite.Run(t, new(ethIOTestSuite))
}
