package e2e

import (
	"context"
	"fmt"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond/bvs"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"golang.org/x/time/rate"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

type stateBankTestSuite struct {
	suite.Suite
	chainIO    io.ChainIO
	contrAddr  string
	callerAddr string
	container  *babylond.BabylonContainer
}

func (suite *stateBankTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	suite.container = container

	deployer := &bvs.Deployer{BabylonContainer: container}
	suite.contrAddr = deployer.DeployStateBank().Address
	suite.callerAddr = "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"

	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
}

func (suite *stateBankTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *stateBankTestSuite) Test_ExecuteStateBank() {
	t := suite.T()
	keyName := "caller"

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)

	stateBank := api.NewStateBankImpl(chainIO)
	stateBank.BindClient(suite.contrAddr)

	resp, err := stateBank.SetRegisteredBVSContract(context.Background(), suite.callerAddr)
	assert.NoError(t, err, "set registered BVS contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("SetRegisteredBVSContract resp: %+v", resp)

	key := "count"
	value := "11"
	resp, err = stateBank.Set(context.Background(), key, value)
	assert.NoError(t, err, "set key-value")
	assert.NotNil(t, resp, "response nil")
	t.Logf("Set resp: %+v", resp)
}

func (suite *stateBankTestSuite) test_StateBankIndexer() {
	t := suite.T()
	keyName := "caller"
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	stateBankApi := api.NewStateBankImpl(chainIO)
	res, err := chainIO.QueryNodeStatus(context.Background())
	if err != nil {
		panic(err)
	}
	latestBlock := res.SyncInfo.LatestBlockHeight
	idxer := stateBankApi.Indexer(chainIO.GetClientCtx(), suite.contrAddr, suite.callerAddr, latestBlock-10, []string{"wasm-UpdateState"}, rate.Limit(5), 3)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	processingQueue, err := idxer.Run(ctx)
	if err != nil {
		t.Fatal(err)
	}
	go func() {
		stateBankApi.EventHandler(processingQueue)
	}()
	stateMap := stateBankApi.GetStateMap()
	i := 0
	for {
		v, ok := stateMap.Load("count")
		if ok {
			fmt.Printf("key:%s val:%v\n", "count", v)
			i++
		}
		if i > 1 {
			break
		}
	}
}

func TestStateBank(t *testing.T) {
	suite.Run(t, new(stateBankTestSuite))
}
