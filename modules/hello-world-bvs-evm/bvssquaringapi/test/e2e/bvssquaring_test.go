package e2e

import (
	"context"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"

	"github.com/prometheus/client_golang/prometheus"
	chainioabi "github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/stretchr/testify/assert"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/bvssquaringapi"
)

var contractAddr = "0xA7b87A0a82e234b0396D337a76E43C762453a884"
var rpcURI = "https://arb-sepolia.g.alchemy.com/v2/p2SMc5MIkqXr1zWVeTCCz7FHXf24-EZ2"
var keyDir = "../../../.eth/keystore"
var userAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"
var password = "123"

func TestExecuteSquaring(t *testing.T) {
	ctx := context.Background()
	t.Logf("TestExecuteSquaring")
	elkLogger := logger.NewELKLogger("bvs_demo_eth")
	elkLogger.SetLogLevel("info")

	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo_eth")
	ethChainIO, err := io.NewETHChainIO(rpcURI, keyDir, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:                 3,
		RetryInterval:              2 * time.Second,
		ConfirmationTimeout:        60 * time.Second,
		ETHGasFeeCapAdjustmentRate: 2,
		ETHGasLimitAdjustmentRate:  1.1,
		GasLimit:                   1000000000,
	})
	if err != nil {
		panic(err)
	}
	contractABI, err := chainioabi.GetContractABI("../../../data/abi", "BVSSquaring")
	if err != nil {
		panic(err)
	}
	contractAddr := common.HexToAddress(contractAddr)
	bvsSquaring := bvssquaringapi.NewBVSSquaringImpl(ethChainIO, contractAddr, contractABI)
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}
	resp, err := bvsSquaring.CreateNewTask(context.Background(), wallet, 10)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("createNewTask txnHash: %+v", resp.TxHash)

	//resp, err = bvsSquaring.RespondToTask(context.Background(), wallet, 10, 100, "0xdC61f809a1313C1bc93055aadaD63844A2aeF843")
	//assert.NoError(t, err, "execute contract")
	//assert.NotNil(t, resp, "response nil")
	//t.Logf("responseToTask txnHash: %+v", resp)

	resp1, err := bvsSquaring.GetTaskInput(ctx, 10)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetTaskInput result: %+v", resp1)

	resp2, err := bvsSquaring.GetTaskResult(ctx, 10)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetTaskResult: %+v", resp2)

	resp3, err := bvsSquaring.GetLatestTaskID(ctx)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("GetLastestTaskId result:%+v", resp3)
}
