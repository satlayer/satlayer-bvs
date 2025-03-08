package e2e

import (
	"context"
	"math/big"
	"testing"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/babylond/cw20"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type delegationTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	contrAddr       string
	strategies      []string
	tokenAddr       string
	strategyManager string
	container       *babylond.BabylonContainer
}

func (suite *delegationTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	suite.container = container

	// Fund Callers
	container.ImportPrivKey("owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
	container.FundAddressUbbn("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x", 1e8)
	container.FundAddressUbbn("bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv", 1e7)
	container.FundAddressUbbn("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk", 1e7)

	// TODO(fuxingloh): operator1, operator2, operator3

	minter := container.GenerateAddress("cw20:minter")
	token := cw20.DeployCw20(container, cw20.InstantiateMsg{
		Decimals: 6,
		InitialBalances: []cw20.Cw20Coin{
			{
				Address: minter.String(),
				Amount:  "1000000000",
			},
		},
		Mint: &cw20.MinterResponse{
			Minter: minter.String(),
		},
		Name:   "Test Token",
		Symbol: "TEST",
	})
	suite.tokenAddr = token.Address

	deployer := &bvs.Deployer{BabylonContainer: container}
	pauser := deployer.DeployPauser(nil)

	tAddr := container.GenerateAddress("test-address").String()
	tAddr1 := container.GenerateAddress("test-address-1").String()
	tAddr2 := container.GenerateAddress("test-address-2").String()

	strategyManager := deployer.DeployStrategyManager(pauser.Address)
	delegationManager := deployer.DeployDelegationManager(pauser.Address, 100, []string{tAddr}, []int64{50})

	suite.strategyManager = strategyManager.Address
	suite.contrAddr = delegationManager.Address
	suite.strategies = []string{
		// Replace with actual strategy addresses
		tAddr1,
		tAddr2,
	}

	// TODO(fuxingloh):
	// tAddr, strategyManager.Address,
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	suite.Require().NoError(err, "setup keyring")

	delegationApi := api.NewDelegationManager(chainIO, delegationManager.Address)
	txResp, err := delegationApi.SetRouting(context.Background(), suite.strategyManager, tAddr)
	suite.Require().NoError(err)
	suite.Require().Equal(uint32(0), txResp.TxResult.Code)

	chainIO, err = suite.chainIO.SetupKeyring("operator1", "test")
	delegationApi = api.NewDelegationManager(chainIO, delegationManager.Address)
	suite.Require().NoError(err, "setup keyring")
	txResp, err = delegationApi.RegisterAsOperator(context.Background(), "", 0)
	suite.Require().NoError(err, "register as operator")
	suite.Require().NotNil(txResp, "tx resp is nil")

}

func (suite *delegationTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

// KeyName needs to be changed every time it is executed
func (suite *delegationTestSuite) test_RegisterAsOperator() {
	t := suite.T()
	keyName := "operator1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	delegation := api.NewDelegationManager(chainIO, suite.contrAddr).WithGasLimit(400000)

	txResp, err := delegation.RegisterAsOperator(
		context.Background(),
		"",
		0,
	)
	assert.NoError(t, err, "register as operator")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_ModifyOperatorDetails() {
	t := suite.T()
	keyName := "operator1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)

	txResp, err := delegation.ModifyOperatorDetails(
		context.Background(),
		0,
	)
	assert.NoError(t, err, "modify operator details")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_UpdateOperatorMetadataURI() {
	t := suite.T()
	keyName := "operator1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.UpdateOperatorMetadataURI(context.Background(), "metadata.uri")
	assert.NoError(t, err, "update operator metadata uri")
	assert.NotNil(t, txResp, "tx resp is nil")
	t.Logf("%+v", txResp)
}

func (suite *delegationTestSuite) Test_DelegateToAndUnDelegate() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)

	txResp, err := delegation.DelegateTo(
		context.Background(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
	)
	assert.NoError(t, err, "delegate to")
	t.Logf("txResp: %v", txResp)

	// repeat to DelegateTo will be failed
	txResp, err = delegation.DelegateTo(
		context.Background(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
	)
	assert.Error(t, err, "delegate to no error")
	t.Logf("txResp: %v", txResp)

	recoverResp, err := delegation.UnDelegate(context.Background(), "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "un delegate")
	t.Logf("recoverResp: %v", recoverResp)
}

func (suite *delegationTestSuite) test_CompleteQueuedWithdrawal() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	nonceResp, err := delegation.GetCumulativeWithdrawalsQueuedNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get nonce")
	currentNonce := new(big.Int)
	_, ok := currentNonce.SetString(nonceResp.CumulativeWithdrawals, 10)
	assert.True(t, ok)
	nonce := currentNonce.String()
	if currentNonce.Cmp(big.NewInt(0)) == 1 {
		nonce = currentNonce.Sub(currentNonce, big.NewInt(1)).String()
	}
	txResp, err := delegation.CompleteQueuedWithdrawal(
		context.Background(),
		delegationmanager.WithdrawalElement{
			Staker:      "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			DelegatedTo: "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
			Withdrawer:  "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Nonce:       nonce,
			StartBlock:  11112958,
			Strategies:  suite.strategies,
			Shares:      []string{"41"},
		},
		0,
		true,
	)
	assert.NoError(t, err, "complete queued withdrawal")
	t.Logf("txResp: %v", txResp)
}
func (suite *delegationTestSuite) test_QueueWithdrawals() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)

	req := []delegationmanager.QueuedWithdrawalParams{
		{
			Withdrawer: "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Strategies: suite.strategies,
			Shares:     []string{"20"},
		},
		{
			Withdrawer: "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Strategies: suite.strategies,
			Shares:     []string{"20"},
		},
	}

	txResp, err := delegation.QueueWithdrawals(context.Background(), req)
	assert.NoError(t, err, "queue withdrawals")
	t.Logf("txResp: %v", txResp)
}
func (suite *delegationTestSuite) test_CompleteQueuedWithdrawals() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	nonceResp, err := delegation.GetCumulativeWithdrawalsQueuedNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get nonce")
	currentNonce := new(big.Int)
	_, ok := currentNonce.SetString(nonceResp.CumulativeWithdrawals, 10)
	assert.True(t, ok)
	// Note the special case where the resulting value is 0
	withdrawals := []delegationmanager.WithdrawalElement{
		{
			Staker:      "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			DelegatedTo: "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
			Withdrawer:  "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Nonce:       "0",
			StartBlock:  11140602,
			Strategies:  suite.strategies,
			Shares:      []string{"20"},
		},
		{
			Staker:      "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			DelegatedTo: "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
			Withdrawer:  "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Nonce:       "1",
			StartBlock:  11140602,
			Strategies:  suite.strategies,
			Shares:      []string{"20"},
		},
	}

	txResp, err := delegation.CompleteQueuedWithdrawals(context.Background(), withdrawals, []int64{0, 0}, []bool{true, true})
	assert.NoError(t, err, "complete queued withdrawals")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_SetMinWithdrawalDelayBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.SetMinWithdrawalDelayBlocks(context.Background(), 10)
	assert.NoError(t, err, "set min withdrawal delay blocks")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_SetStrategyWithdrawalDelayBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.SetStrategyWithdrawalDelayBlocks(context.Background(), suite.strategies, []int64{5, 5})
	assert.NoError(t, err, "set strategy withdrawal delay blocks")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_DelegateTransferOwnership() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.TransferOwnership(context.Background(), "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.NoError(t, err, "transfer ownership")
	t.Logf("txResp: %v", txResp)

	// not owner to transfer ownership will be failed
	txResp, err = delegation.TransferOwnership(context.Background(), "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.Error(t, err, "transfer ownership not failed")
	t.Logf("not owner to transfer ownership txResp: %v", txResp)

	RecoverClient, err := suite.chainIO.SetupKeyring("aggregator", "test")
	assert.NoError(t, err, "create cosmos client")
	recoverDelegation := api.NewDelegationManager(RecoverClient, suite.contrAddr)
	recoverResp, err := recoverDelegation.TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err, "transfer ownership")
	t.Logf("recoverResp: %v", recoverResp)
}

func (suite *delegationTestSuite) Test_SetRouting() {
	chainIO, err := suite.chainIO.SetupKeyring("caller", "test")
	suite.NoError(err)

	tAddr := suite.container.GenerateAddress("test-address").String()
	delegationApi := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegationApi.SetRouting(context.Background(), suite.strategyManager, tAddr)
	suite.NoError(err)
	suite.NotNil(txResp, "response nil")
}

func (suite *delegationTestSuite) Test_IsDelegated() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.IsDelegated("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "check delegation")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_IsOperator() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.IsOperator("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "check operator")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_OperatorDetails() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.OperatorDetails("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "operator details")
	t.Logf("txResp: %v", txResp)

	// not exist operator will be failed
	txResp, err = delegation.OperatorDetails("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430d")
	assert.Error(t, err, "operator details")
	assert.Nil(t, txResp, "operator details")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_StakerOptOutWindowBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.StakerOptOutWindowBlocks("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "staker opt out window blocks")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetOperatorShares() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.GetOperatorShares(
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		suite.strategies,
	)
	assert.NoError(t, err, "get operator shares")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetOperatorStakers() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.GetOperatorStakers("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "get operator stakers")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetDelegatableShares() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.GetDelegatableShares("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get delegatable shares")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetWithdrawalDelay() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.GetWithdrawalDelay(suite.strategies)
	assert.NoError(t, err, "get withdrawal delay")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_CalculateWithdrawalRoot() {
	t := suite.T()
	keyName := "staker2"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)

	req := delegationmanager.CalculateWithdrawalRootWithdrawal{
		Staker:      "bbn14x6wy9kfj8hxqh03c8zwmzy9xsn4yh55xxf9qu",
		DelegatedTo: "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		Withdrawer:  "bbn14x6wy9kfj8hxqh03c8zwmzy9xsn4yh55xxf9qu",
		Nonce:       "0",
		StartBlock:  10968238,
		Strategies:  suite.strategies,
		Shares:      []string{"20"},
	}

	txResp, err := delegation.CalculateWithdrawalRoot(req)
	assert.NoError(t, err, "calculate withdrawal root")
	t.Logf("txResp:%v", txResp)
}

func (suite *delegationTestSuite) Test_GetCumulativeWithdrawalsQueuedNonce() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationManager(chainIO, suite.contrAddr)
	txResp, err := delegation.GetCumulativeWithdrawalsQueuedNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get withdrawal queued nonce")
	t.Logf("txResp: %+v", txResp)
}

func TestDelegationTestSuite(t *testing.T) {
	suite.Run(t, new(delegationTestSuite))
}
