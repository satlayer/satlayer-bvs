package e2e

import (
	"context"
	"testing"

	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"

	"github.com/satlayer/satlayer-bvs/babylond/bvs"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type slashManagerTestSuite struct {
	suite.Suite
	chainIO                  io.ChainIO
	contrAddr                string
	strategyManagerContrAddr string
	container                *babylond.BabylonContainer
}

func (suite *slashManagerTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.container = container
	suite.chainIO = container.NewChainIO("../.babylon")

	// Import And Fund Caller
	container.ImportPrivKey("slash-manager:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	tAddr := container.GenerateAddress("test-address").String()
	deployer := &bvs.Deployer{BabylonContainer: container}
	slashManager := deployer.DeploySlashManager(tAddr, tAddr)
	suite.contrAddr = slashManager.Address
	suite.strategyManagerContrAddr = tAddr
}

func (suite *slashManagerTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *slashManagerTestSuite) Test_SetMinimalSlashSignature() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
	slashApi.WithGasLimit(300000)

	{ // Setup SetSlasher
		_, err = slashApi.SetSlasher(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", true)
		suite.Require().NoError(err)
	}

	minimalSignature := int64(1)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

// 	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
	slashApi.WithGasLimit(300000)

	{ // Setup SetPauser
		_, err = slashApi.SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		suite.Require().NoError(err)
	}

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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
	slashApi.WithGasLimit(300000)

	{ // Setup SetPauser
		_, err = slashApi.SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
		suite.Require().NoError(err)
	}

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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
	slashApi.WithGasLimit(300000)

	{ // Setup SetSlasher
		_, err = slashApi.SetSlasher(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", true)
		suite.Require().NoError(err)
	}

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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

// 	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

// 	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

// 	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
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

	slashApi := api.NewSlashManager(chainIO, suite.contrAddr)
	slashApi.WithGasLimit(300000)

	sender := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	slashDetails := slashmanager.CalculateSlashHashSlashDetails{
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

	pubKey := chainIO.GetCurrentAccountPubKey()
	validatorsPublicKeys := []cryptotypes.PubKey{pubKey}

	txResp, err := slashApi.CalculateSlashHash(sender, slashDetails, validatorsPublicKeys)
	assert.NoError(t, err, "CalculateSlashHash failed")
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func TestSlashManagerTestSuite(t *testing.T) {
	suite.Run(t, new(slashManagerTestSuite))
}
