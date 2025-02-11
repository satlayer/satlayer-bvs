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

type ethBVSDirectoryTestSuite struct {
	suite.Suite
	ethChainIO   io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (suite *ethBVSDirectoryTestSuite) SetupTest() {
	endpoint := "https://arb-sepolia.g.alchemy.com/v2/p2SMc5MIkqXr1zWVeTCCz7FHXf24-EZ2"
	keystorePath := "../.eth/keystore"
	abiPath := "../../chainio/abi"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	ethChainIO, err := io.NewETHChainIO(endpoint, keystorePath, logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:                 3,
		RetryInterval:              2 * time.Second,
		ConfirmationTimeout:        15 * time.Second,
		ETHGasFeeCapAdjustmentRate: 2,
		ETHGasLimitAdjustmentRate:  1.1,
		GasLimit:                   1000000000,
	})
	suite.Require().NoError(err)
	contractABI, err := chainioabi.GetContractABI(abiPath, "BVSDirectory")
	suite.Require().NoError(err)

	suite.ethChainIO = ethChainIO
	suite.contractAddr = common.HexToAddress("0xaB7862A971CcA44F883441605134B534B41e5B4F")
	suite.contractABI = contractABI
}

func (suite *ethBVSDirectoryTestSuite) Test_RegisterBVS() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	bvsHash := "226466AF1CF2ECDA66821E7833C325F15037D6BB7CC0CE39A8908587D02C1046"
	bvsContract := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	bvsDirectory := api.NewETHBVSDirectoryImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := bvsDirectory.RegisterBVS(ctx, wallet, bvsHash, bvsContract)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethBVSDirectoryTestSuite) Test_GetBVSInfo() {
	ctx := context.Background()
	bvsHash := "226466AF1CF2ECDA66821E7833C325F15037D6BB7CC0CE39A8908587D02C1046"
	bvsDirectory := api.NewETHBVSDirectoryImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := bvsDirectory.GetBVSInfo(ctx, bvsHash)
	assert.NoError(suite.T(), err)
	suite.T().Logf("bvs info %+v", resp)
}

func (suite *ethBVSDirectoryTestSuite) Test_Owner() {
	ctx := context.Background()
	bvsDirectory := api.NewETHBVSDirectoryImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := bvsDirectory.Owner(ctx)
	assert.NoError(suite.T(), err)
	suite.T().Logf("bvs info %s", resp)
}

func TestETHBVSDirectory(t *testing.T) {
	suite.Run(t, new(ethBVSDirectoryTestSuite))
}
