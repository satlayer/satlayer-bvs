package e2e

import (
	"context"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/stretchr/testify/assert"

	"github.com/satlayer/satlayer-bvs/examples/squaring/bvssquaringapi"
)

func testExecuteSquaring(t *testing.T) {
	contractAddr := "bbn1ynpwwfu05ujdurq6rj4rvgkejzmx2mm7jsyxkke3xhxddlvzwp2ssnqv9h"
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../../../.babylond"
	keyName := "wallet1"

	t.Logf("TestExecuteSquaring")
	elkLogger := logger.NewMockELKLogger()
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	assert.NoError(t, err, "failed to create chain IO")
	chainIO, err = chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err, "failed to setup keyring")

	bvsSquaring := bvssquaringapi.NewBVSSquaring(chainIO)
	bvsSquaring.BindClient(contractAddr)

	resp, err := bvsSquaring.CreateNewTask(context.Background(), 10)
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = bvsSquaring.RespondToTask(context.Background(), 10, 100, "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x")
	assert.NoError(t, err, "execute contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
}

func testQuerySquaring(t *testing.T) {
	contractAddr := "bbn1ynpwwfu05ujdurq6rj4rvgkejzmx2mm7jsyxkke3xhxddlvzwp2ssnqv9h"
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../../../.babylond"
	keyName := "wallet1"

	elkLogger := logger.NewMockELKLogger()
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	assert.NoError(t, err, "failed to create chain IO")
	chainIO, err = chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err, "failed to setup keyring")

	bvsSquaring := bvssquaringapi.NewBVSSquaring(chainIO)
	bvsSquaring.BindClient(contractAddr)

	resp, err := bvsSquaring.GetTaskInput(10)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = bvsSquaring.GetTaskResult(10)
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)

	resp, err = bvsSquaring.GetLatestTaskID()
	assert.NoError(t, err, "query contract")
	assert.NotNil(t, resp, "response nil")
	t.Logf("resp:%+v", resp)
}
