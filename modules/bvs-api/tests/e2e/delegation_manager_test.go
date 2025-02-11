package e2e

import (
	"context"
	"encoding/base64"
	"math/big"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

type delegationTestSuite struct {
	suite.Suite
	chainIO      io.ChainIO
	contrAddr    string
	strategies   []string
	token        string
	slashManager string
}

func (suite *delegationTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "delegation")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    15 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.contrAddr = "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"
	suite.strategies = []string{
		"bbn1326vx56sy7ra2qk4perr2tg8td3ln4qll3s2l4vu8jclxdplzj5scxzahc",
		"bbn1df8tu3pxrxf2cs0s4rjcvdjuhs25he9l8v720yvl59hj7phvgmyqp873ay",
	}
	suite.token = "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	suite.slashManager = "bbn1z52hmh7ht0364lzcs8700sgrnns84sa3wr9c8upd80es5n5x65mq2dedfp"
}

// KeyName needs to be changed every time it is executed
func (suite *delegationTestSuite) test_RegisterAsOperator() {
	t := suite.T()
	keyName := "operator1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")

	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr).WithGasLimit(400000)

	txResp, err := delegation.RegisterAsOperator(
		context.Background(),
		account.GetPubKey(),
		"",
		"0",
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

	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)

	txResp, err := delegation.ModifyOperatorDetails(
		context.Background(),
		"bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv",
		"bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv",
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)

	approverAccount, err := chainIO.QueryAccount("bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.NoError(t, err, "get account")

	txResp, err := delegation.DelegateTo(
		context.Background(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		approverAccount.GetAddress().String(),
		"aggregator",
		approverAccount.GetPubKey(),
	)
	assert.NoError(t, err, "delegate to")
	t.Logf("txResp: %v", txResp)

	// repeat to DelegateTo will be failed
	txResp, err = delegation.DelegateTo(
		context.Background(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		approverAccount.GetAddress().String(),
		"aggregator",
		approverAccount.GetPubKey(),
	)
	assert.Error(t, err, "delegate to no error")
	t.Logf("txResp: %v", txResp)

	recoverResp, err := delegation.UnDelegate(context.Background(), "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "un delegate")
	t.Logf("recoverResp: %v", recoverResp)
}

func (suite *delegationTestSuite) Test_DelegateToBySignatureAndUnDelegate() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)

	stakerAccount, err := chainIO.QueryAccount("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get account")

	approverAccount, err := chainIO.QueryAccount("bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv")
	assert.NoError(t, err, "get account")

	txResp, err := delegation.DelegateToBySignature(
		context.Background(),
		"bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		"bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
		"staker1",
		"bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv",
		"aggregator",
		stakerAccount.GetPubKey(),
		approverAccount.GetPubKey(),
	)
	assert.NoError(t, err, "delegate to by signature")
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
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
		types.Withdrawal{
			Staker:      "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			DelegatedTo: "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
			Withdrawer:  "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Nonce:       nonce,
			StartBlock:  11112958,
			Strategies:  suite.strategies,
			Shares:      []string{"41"},
		},
		[]string{suite.token},
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)

	req := []types.QueuedWithdrawalParams{
		{
			WithDrawer: "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
			Strategies: suite.strategies,
			Shares:     []string{"20"},
		},
		{
			WithDrawer: "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	nonceResp, err := delegation.GetCumulativeWithdrawalsQueuedNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get nonce")
	currentNonce := new(big.Int)
	_, ok := currentNonce.SetString(nonceResp.CumulativeWithdrawals, 10)
	assert.True(t, ok)
	// Note the special case where the resulting value is 0
	withdrawals := []types.Withdrawal{
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
	tokens := [][]string{
		{suite.token},
		{suite.token},
	}

	txResp, err := delegation.CompleteQueuedWithdrawals(context.Background(), withdrawals, tokens, []uint64{0, 0}, []bool{true, true})
	assert.NoError(t, err, "complete queued withdrawals")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_SetMinWithdrawalDelayBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.SetMinWithdrawalDelayBlocks(context.Background(), 10)
	assert.NoError(t, err, "set min withdrawal delay blocks")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_SetStrategyWithdrawalDelayBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.SetStrategyWithdrawalDelayBlocks(context.Background(), suite.strategies, []uint64{5, 5})
	assert.NoError(t, err, "set strategy withdrawal delay blocks")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_DelegateTransferOwnership() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err, "transfer ownership")
	t.Logf("txResp: %v", txResp)

	// not owner to transfer ownership will be failed
	RecoverClient, err := suite.chainIO.SetupKeyring("aggregator", "test")
	assert.NoError(t, err, "create cosmos client")
	recoverDelegation := api.NewDelegationImpl(RecoverClient, suite.contrAddr)
	recoverResp, err := recoverDelegation.TransferOwnership(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.Error(t, err, "transfer ownership failed")
	assert.Nil(t, recoverResp, "transfer ownership resp nil")
}

func (suite *delegationTestSuite) Test_DelegationPause() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	txResp, err := api.NewDelegationImpl(chainIO, suite.contrAddr).Pause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)

	recoverResp, err := api.NewDelegationImpl(chainIO, suite.contrAddr).Unpause(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, recoverResp, "response nil")
	t.Logf("txResp:%+v", recoverResp)
}

func (suite *delegationTestSuite) Test_DelegationSetPauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDelegationImpl(chainIO, suite.contrAddr).SetPauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *delegationTestSuite) Test_DelegationSetUnpauser() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDelegationImpl(chainIO, suite.contrAddr).SetUnpauser(context.Background(), "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *delegationTestSuite) Test_SetSlashManager() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	txResp, err := api.NewDelegationImpl(chainIO, suite.contrAddr).SetSlashManager(context.Background(), suite.slashManager)
	assert.NoError(t, err)
	assert.NotNil(t, txResp, "response nil")
	t.Logf("txResp:%+v", txResp)
}

func (suite *delegationTestSuite) Test_IsDelegated() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.IsDelegated("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "check delegation")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_IsOperator() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.IsOperator("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "check operator")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_OperatorDetails() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.OperatorDetails("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "operator details")
	t.Logf("txResp: %v", txResp)

	// not exist operator will be failed
	txResp, err = delegation.OperatorDetails("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430d")
	assert.Error(t, err, "operator details")
	assert.Nil(t, txResp, "operator details")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_DelegationApprover() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.DelegationApprover("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "delegation approver")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_StakerOptOutWindowBlocks() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.StakerOptOutWindowBlocks("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "staker opt out window blocks")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetOperatorShares() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
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
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.GetOperatorStakers("bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "get operator stakers")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetDelegatableShares() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.GetDelegatableShares("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get delegatable shares")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetWithdrawalDelay() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.GetWithdrawalDelay(suite.strategies)
	assert.NoError(t, err, "get withdrawal delay")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_CalculateWithdrawalRoot() {
	t := suite.T()
	keyName := "staker2"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)

	req := types.Withdrawal{
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

func (suite *delegationTestSuite) Test_CalculateCurrentStakerDelegationDigestHash() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	stakerAccount, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")
	nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err, "query node status")
	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
	req := types.CurrentStakerDigestHashParams{
		Staker:          "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
		Operator:        "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		StakerPublicKey: base64.StdEncoding.EncodeToString(stakerAccount.GetPubKey().Bytes()),
		Expiry:          expiry,
		CurrentNonce:    "0",
		ContractAddr:    suite.contrAddr,
	}

	txResp, err := delegation.CalculateCurrentStakerDelegationDigestHash(req)
	assert.NoError(t, err, "get current staker delegation hash")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_StakerDelegationDigestHash() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	stakerAccount, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")
	nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err, "query node status")
	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
	req := types.StakerDigestHashParams{
		Staker:          "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
		StakerNonce:     "0",
		Operator:        "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		StakerPublicKey: base64.StdEncoding.EncodeToString(stakerAccount.GetPubKey().Bytes()),
		Expiry:          expiry,
		ContractAddr:    suite.contrAddr,
	}

	txResp, err := delegation.StakerDelegationDigestHash(req)
	assert.NoError(t, err, "get staker delegation hash")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_DelegationApprovalDigestHash() {
	t := suite.T()
	keyName := "aggregator"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	nodeStatus, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err, "query node status")

	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)
	randomStr, err := utils.GenerateRandomString(16)
	assert.NoError(suite.T(), err)
	salt := "salt" + randomStr

	approverAccount, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err, "get account")

	req := types.ApproverDigestHashParams{
		Staker:            "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk",
		Operator:          "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x",
		Approver:          "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv",
		ApproverPublicKey: base64.StdEncoding.EncodeToString(approverAccount.GetPubKey().Bytes()),
		ApproverSalt:      base64.StdEncoding.EncodeToString([]byte(salt)),
		Expiry:            expiry,
		ContractAddr:      suite.contrAddr,
	}

	txResp, err := delegation.DelegationApprovalDigestHash(req)
	assert.NoError(t, err, "get delegation approver hash")
	t.Logf("txResp: %v", txResp)
}

func (suite *delegationTestSuite) Test_GetStakerNonce() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.GetStakerNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get withdrawal delay")
	t.Logf("txResp: %+v", txResp)
}

func (suite *delegationTestSuite) Test_GetCumulativeWithdrawalsQueuedNonce() {
	t := suite.T()
	keyName := "staker1"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	delegation := api.NewDelegationImpl(chainIO, suite.contrAddr)
	txResp, err := delegation.GetCumulativeWithdrawalsQueuedNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
	assert.NoError(t, err, "get withdrawal queued nonce")
	t.Logf("txResp: %+v", txResp)
}

func TestDelegationTestSuite(t *testing.T) {
	suite.Run(t, new(delegationTestSuite))
}
