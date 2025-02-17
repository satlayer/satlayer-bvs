package e2e

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"golang.org/x/time/rate"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-cw/wasm"
)

type stateBankTestSuite struct {
	suite.Suite
	chainIO    io.ChainIO
	contrAddr  string
	callerAddr string
}

func (suite *stateBankTestSuite) SetupTest() {
	container, err := babylond.Run(context.Background())
	suite.Require().NoError(err)

	wasmByteCode, err := wasm.ReadWasmFile("cw-state-bank")
	suite.Require().NoError(err)

	initJson := fmt.Sprintf(`{"initial_owner": "%s"}`, container.GenerateAddress("initial_owner").String())
	contract, err := container.DeployWasmCode(wasmByteCode, []byte(initJson), "BVS State Bank", "genesis")
	suite.Require().NoError(err)

	suite.contrAddr = contract.Address
	suite.callerAddr = "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	_, err = container.FundAccountUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)
	suite.Require().NoError(err)

	chainIO, err := container.NewChainIO("../.babylon")
	suite.Require().NoError(err)
	suite.chainIO = chainIO
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
