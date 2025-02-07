package e2e

import (
	"context"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	chainioabi "github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
)

type ethBVSDriverTestSuite struct {
	suite.Suite
	ethChainIO   io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (suite *ethBVSDriverTestSuite) SetupTest() {
	endpoint := "https://arb-sepolia.g.alchemy.com/v2/p2SMc5MIkqXr1zWVeTCCz7FHXf24-EZ2"
	keystorePath := "../.eth/keystore"
	abiPath := "../../chainio/abi"

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
	contractABI, err := chainioabi.GetContractABI(abiPath, "BVSDriver")
	suite.Require().NoError(err)

	suite.ethChainIO = ethChainIO
	suite.contractAddr = common.HexToAddress("0x1ED3f2CD59d2B5a3Eb0caa6b9D4529947dddadD2")
	suite.contractABI = contractABI
}

func (suite *ethBVSDriverTestSuite) Test_AddRegisteredBvsContract() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	contractAddr := common.HexToAddress("0x1ED3f2CD59d2B5a3Eb0caa6b9D4529947dddadD2")
	bvsDriver := api.NewETHBVSDriverImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	isRegistered, err := bvsDriver.IsBVSContractRegistered(ctx, contractAddr)
	suite.Require().NoError(err)
	if !isRegistered {
		resp, err := bvsDriver.AddRegisteredBvsContract(ctx, wallet, contractAddr)
		assert.NoError(suite.T(), err)
		suite.T().Logf("txhash %s", resp.TxHash)
	} else {
		suite.T().Logf("address already registered %s", contractAddr)
	}
}

func (suite *ethBVSDriverTestSuite) Test_ExecuteBVSOffChain() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	taskID := "1"
	bvsDriver := api.NewETHBVSDriverImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := bvsDriver.ExecuteBVSOffChain(ctx, wallet, taskID)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethBVSDriverTestSuite) Test_IsBVSContractRegistered() {
	ctx := context.Background()
	bvsContract := common.HexToAddress("0x1ED3f2CD59d2B5a3Eb0caa6b9D4529947dddadD2")
	bvsDriver := api.NewETHBVSDriverImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := bvsDriver.IsBVSContractRegistered(ctx, bvsContract)
	assert.NoError(suite.T(), err)
	suite.T().Logf("result %t", resp)
}

func TestETHBVSDriver(t *testing.T) {
	suite.Run(t, new(ethBVSDriverTestSuite))
}
