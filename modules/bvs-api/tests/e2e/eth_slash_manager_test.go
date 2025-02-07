package e2e

import (
	"context"
	"math/big"
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

type ethSlashManagerTestSuite struct {
	suite.Suite
	ethChainIO   io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (suite *ethSlashManagerTestSuite) SetupTest() {
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
	contractABI, err := chainioabi.GetContractABI(abiPath, "SlashManager")
	suite.Require().NoError(err)

	suite.ethChainIO = ethChainIO
	suite.contractAddr = common.HexToAddress("0x66E371A5dfF96fcC88864F292ecfC85a5B853176")
	suite.contractABI = contractABI
}

func (suite *ethSlashManagerTestSuite) Test_CancelSlashRequest() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	slashHash := "8033994ebff512d0c140140ee6900064c9b98ff732b02320ae384ad3a7976966"
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.CancelSlashRequest(ctx, wallet, slashHash)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_ExecuteSlashRequest() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	slashHash := "638789a8cd83d13edab39fcf89b7044e693a5e96ee56348328f2405442ec6d09"
	signatures := []string{"m2uENcWcVtC+JTVs5duyxS0tPJLH00s9wlKON/3fRdAhzsgS/AcdwGbHxUzbT5b/WlfANGHMDvmZy5eVyFnL9g=="}
	validatorsPublicKeys := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}

	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.ExecuteSlashRequest(ctx, wallet, slashHash, signatures, validatorsPublicKeys)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_SetMinimalSlashSignature() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	minimalSlashSignature := big.NewInt(1)
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.SetMinimalSlashSignature(ctx, wallet, minimalSlashSignature)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_SetSlashValidator() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	validators := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}
	values := []bool{true}
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.SetSlashValidator(ctx, wallet, validators, values)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_SetSlasher() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	slasher := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.SetSlasher(ctx, wallet, slasher)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_SubmitSlashRequest() {
	ctx := context.Background()
	fromAddr := common.HexToAddress("0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
	pwd := "123"
	executeSlashDetails := types.SlashDetails{
		Slasher:         "bbn1m4gtpe3wfhlmvwultl678rxzexyduy60jjm6ty",
		Operator:        "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		Share:           big.NewInt(10),
		SlashSignature:  1,
		SlashValidators: []string{"bbn1m4gtpe3wfhlmvwultl678rxzexyduy60jjm6ty"},
		Reason:          "test",
		StartTime:       big.NewInt(1736144991696),
		EndTime:         big.NewInt(1736144995296),
		Status:          true,
	}
	validatorsPublicKeys := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      pwd,
	}

	resp, err := slashManager.SubmitSlashRequest(ctx, wallet, executeSlashDetails, validatorsPublicKeys)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.TxHash)
}

func (suite *ethSlashManagerTestSuite) Test_MinimalSlashSignature() {
	ctx := context.Background()
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := slashManager.MinimalSlashSignature(ctx)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.String())
}

func (suite *ethSlashManagerTestSuite) Test_Slasher() {
	ctx := context.Background()
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := slashManager.Slasher(ctx)
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %s", resp.String())
}

func (suite *ethSlashManagerTestSuite) Test_Validators() {
	ctx := context.Background()
	slashManager := api.NewETHSlashManagerImpl(suite.ethChainIO, suite.contractAddr, suite.contractABI)

	resp, err := slashManager.Validators(ctx, "abc")
	assert.NoError(suite.T(), err)
	suite.T().Logf("txhash %t", resp)
}

func TestETHSlashManager(t *testing.T) {
	suite.Run(t, new(ethSlashManagerTestSuite))
}
