package e2e

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"golang.org/x/time/rate"

	chainioabi "github.com/satlayer/satlayer-api/chainio/abi"
	"github.com/satlayer/satlayer-api/chainio/api"
	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-api/logger"
	transactionprocess "github.com/satlayer/satlayer-api/metrics/indicators/transaction_process"
)

type ethStateBankTestSuite struct {
	suite.Suite
	ethChainIO   io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (suite *ethStateBankTestSuite) SetupTest() {
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
	contractABI, err := chainioabi.GetContractABI(abiPath, "StateBank")
	suite.Require().NoError(err)

	suite.ethChainIO = ethChainIO
	suite.contractAddr = common.HexToAddress("0xFa5000dA183A9422322483923620bB656902355A")
	suite.contractABI = contractABI
}

func (suite *ethStateBankTestSuite) Test_ETHRegisteredAndSet() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	pwd := "123"
	addr := common.HexToAddress("0xaA0851f2939EF2D8B51971B510383Fcb5c246a17")
	stateBank := api.NewETHStateBankImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	isRegistered, err := stateBank.IsBVSContractRegistered(ctx, addr)
	assert.NoError(suite.T(), err)
	if !isRegistered {
		resp, err := stateBank.SetRegisteredBVSContract(ctx, wallet, addr)
		assert.NoError(suite.T(), err, "set registered BVS contract")
		assert.NotNil(suite.T(), resp, "response nil")
		suite.T().Logf("SetRegisteredBVSContract txhash %s", resp.TxHash)
	}

	key := "testkey"
	value := "222222"
	resp, err := stateBank.Set(context.Background(), wallet, key, value)
	assert.NoError(suite.T(), err, "set key-value")
	assert.NotNil(suite.T(), resp, "response nil")
	suite.T().Logf("Set tx: %+v", resp)
}

func (suite *ethStateBankTestSuite) Test_Get() {
	ctx := context.Background()
	key := "testkey"
	stateBank := api.NewETHStateBankImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	keyStr, err := stateBank.Get(ctx, key)
	assert.NoError(suite.T(), err)
	assert.NotEmpty(suite.T(), keyStr)
	fmt.Println(keyStr)
}

func (suite *ethStateBankTestSuite) Test_ETHStateBankIndexer() {
	ctx := context.Background()
	latestBlock, err := suite.ethChainIO.GetLatestBlockNumber(ctx)
	bvsContract := "0x1CCF4368c82DB1B9eD8daBC76D77D40F24466BA1"
	assert.NoError(suite.T(), err)
	stateBank := api.NewETHStateBankImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	eventTypes := []common.Hash{
		common.HexToHash("0x6c94acf0e05d6bee21156de7d7badca3e844eb6b31df94a1bdf9bbf3cd2847de"),
	}

	idxer := stateBank.Indexer(suite.ethChainIO.GetETHClient(), bvsContract, latestBlock-10, eventTypes, rate.Limit(5), 3)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	processingQueue, err := idxer.Run(ctx)
	if err != nil {
		suite.T().Fatal(err)
	}
	for event := range processingQueue {
		suite.T().Logf("Received event: %+v", event)
	}
	time.Sleep(5 * time.Second)
}

func TestETHStateBank(t *testing.T) {
	suite.Run(t, new(ethStateBankTestSuite))
}
