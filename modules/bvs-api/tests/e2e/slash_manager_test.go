package e2e

import (
	"context"
	"testing"
	"time"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-api/chainio/api"
	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-api/logger"
	transactionprocess "github.com/satlayer/satlayer-api/metrics/indicators/transaction_process"
)

type slashManagerTestSuite struct {
	suite.Suite
	chainIO                  io.ChainIO
	contrAddr                string
	strategyManagerContrAddr string
}

func (suite *slashManagerTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "slash_manager")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.contrAddr = "bbn1z52hmh7ht0364lzcs8700sgrnns84sa3wr9c8upd80es5n5x65mq2dedfp"
	suite.strategyManagerContrAddr = "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"
}

func (suite *slashManagerTestSuite) Test_SetMinimalSlashSignature() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	minimalSignature := uint64(1)
	txResp, err := slashApi.SetMinimalSlashSignature(context.Background(), minimalSignature)
	assert.NoError(t, err, "SetMinimalSlashSignature failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_SetSlasher() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	slasher := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := slashApi.SetSlasher(context.Background(), slasher, true)
	assert.NoError(t, err, "SetSlasher failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

// func (suite *slashManagerTestSuite) Test_TransferOwnership() {
// 	t := suite.T()
// 	keyName := "testkey"
// 	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
// 	assert.NoError(t, err)

// 	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
// 	slashApi.BindClient(suite.contrAddr)
// 	slashApi.WithGasLimit(300000)

// 	newOwner := "osmo1l3u09t2x6ey8xcrhc5e48ygauqlxy3facyz34p"
// 	txResp, err := slashApi.TransferOwnership(context.Background(), newOwner)
// 	assert.NoError(t, err, "TransferOwnership failed")
// 	assert.NotNil(t, txResp, "response nil")
// 	t.Logf("txResp:%+v", txResp)
// }

func (suite *slashManagerTestSuite) Test_SetDelegationManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	newDelegationManager := "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"
	txResp, err := slashApi.SetDelegationManager(context.Background(), newDelegationManager)
	assert.NoError(t, err, "SetDelegationManager failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_SetPauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	newPauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := slashApi.SetPauser(context.Background(), newPauser)
	assert.NoError(t, err, "SetPauser failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_SetUnpauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	newUnpauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := slashApi.SetUnpauser(context.Background(), newUnpauser)
	assert.NoError(t, err, "SetUnpauser failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_Pause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	txResp, err := slashApi.Pause(context.Background())
	assert.NoError(t, err, "Pause failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_Unpause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	txResp, err := slashApi.Unpause(context.Background())
	assert.NoError(t, err, "Unpause failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_SetSlasherValidator() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	validators := []string{"bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"}
	values := []bool{true}
	txResp, err := slashApi.SetSlasherValidator(context.Background(), validators, values)
	assert.NoError(t, err, "SetSlasherValidator failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_SetStrategyManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	txResp, err := slashApi.SetStrategyManager(context.Background(), suite.strategyManagerContrAddr)
	assert.NoError(t, err, "SetStrategyManager failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

// func (suite *slashManagerTestSuite) Test_SubmitSlashRequest() {
// 	t := suite.T()
// 	keyName := "testkey"
// 	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
// 	assert.NoError(t, err)
// 	account, err := chainIO.GetCurrentAccount()
// 	assert.NoError(t, err)

// 	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
// 	slashApi.BindClient(suite.contrAddr)
// 	slashApi.WithGasLimit(300000)

// 	slashDetails := types.ExecuteSlashDetails{
// 		Slasher:        account.GetAddress().String(),
// 		Operator:       "osmo18872ufnvmc5pg8la8dyfty3n6a8xj9l6nk8sh2",
// 		Share:          "10000",
// 		SlashSignature: 1,
// 		SlashValidator: []string{account.GetAddress().String()},
// 		Reason:         "test",
// 		StartTime:      1729176928,
// 		EndTime:        1827426463,
// 		Status:         true,
// 	}

// 	txResp, err := slashApi.SubmitSlashRequest(context.Background(), slashDetails, []cryptotypes.PubKey{account.GetPubKey()})
// 	assert.NoError(t, err, "SubmitSlashRequest failed")
// 	assert.NotNil(t, txResp, "response nil")
// 	t.Logf("txResp:%+v", txResp)
// }

// func (suite *slashManagerTestSuite) test_ExecuteSlashRequest() {
// 	t := suite.T()
// 	keyName := "testkey"
// 	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
// 	assert.NoError(t, err)
// 	account, err := chainIO.GetCurrentAccount()
// 	assert.NoError(t, err)

// 	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
// 	slashApi.BindClient(suite.contrAddr)
// 	slashApi.WithGasLimit(2000000)

// 	slashHash := "8644527532e4230b12809aade8cf2aa018b24e7c1f1051562d653744cc49bcab"
// 	validatorsPublicKeys := []cryptotypes.PubKey{account.GetPubKey()}

// 	txResp, err := slashApi.ExecuteSlashRequest(context.Background(), slashHash, validatorsPublicKeys)
// 	assert.NoError(t, err, "ExecuteSlashRequest failed")
// 	assert.NotNil(t, txResp, "response nil")
// 	t.Logf("txResp:%+v", txResp)
// }

// func (suite *slashManagerTestSuite) test_CancelSlashRequest() {
// 	t := suite.T()
// 	keyName := "testkey"
// 	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
// 	assert.NoError(t, err)

// 	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
// 	slashApi.BindClient(suite.contrAddr)
// 	slashApi.WithGasLimit(300000)

// 	slashHash := "8644527532e4230b12809aade8cf2aa018b24e7c1f1051562d653744cc49bcab"
// 	txResp, err := slashApi.CancelSlashRequest(context.Background(), slashHash)
// 	assert.NoError(t, err, "CancelSlashRequest failed")
// 	assert.NotNil(t, txResp, "response nil")
// 	t.Logf("txResp:%+v", txResp)
// }

func (suite *slashManagerTestSuite) test_GetSlashDetails() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	slashHash := "8644527532e4230b12809aade8cf2aa018b24e7c1f1051562d653744cc49bcab"

	txResp, err := slashApi.GetSlashDetails(slashHash)
	assert.NoError(t, err, "GetSlashDetails failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_IsValidator() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	validator := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	txResp, err := slashApi.IsValidator(validator)
	assert.NoError(t, err, "IsValidator failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) test_GetMinimalSlashSignature() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	txResp, err := slashApi.GetMinimalSlashSignature()
	assert.NoError(t, err, "GetMinimalSlashSignature failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *slashManagerTestSuite) Test_CalculateSlashHash() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err)

	slashApi := api.NewSlashManagerImpl(chainIO, suite.contrAddr)
	slashApi.BindClient(suite.contrAddr)
	slashApi.WithGasLimit(300000)

	sender := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	slashDetails := types.ExecuteSlashDetails{
		Slasher:        sender,
		Operator:       "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		Share:          "10000",
		SlashSignature: 1,
		SlashValidator: []string{sender},
		Reason:         "test",
		StartTime:      1729176928,
		EndTime:        1827363142,
		Status:         true,
	}

	validatorsPublicKeys := []cryptotypes.PubKey{account.GetPubKey()}

	txResp, err := slashApi.CalculateSlashHash(sender, slashDetails, validatorsPublicKeys)
	assert.NoError(t, err, "CalculateSlashHash failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func TestSlashManagerTestSuite(t *testing.T) {
	suite.Run(t, new(slashManagerTestSuite))
}
