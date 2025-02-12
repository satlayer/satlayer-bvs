package task

import (
	"context"
	"fmt"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/examples/squaring/task/core"
)

type Monitor struct {
	bvsContract string
	chainIO     io.ChainIO
}

// RunMonitor runs the monitor.
//
// No parameters.
// No return values.
func RunMonitor() {
	m := NewMonitor()
	m.Run()
}

// NewMonitor creates a new instance of the Monitor struct.
//
// No parameters.
// Returns a pointer to the newly created Monitor struct.
func NewMonitor() *Monitor {
	// init chain and log
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")

	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Owner.KeyDir, core.C.Owner.Bech32Prefix, elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	// setup running key
	client, err := chainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	if err != nil {
		panic(err)
	}

	// create BVSDirectory
	txResp, err := api.NewBVSDirectoryImpl(client, core.C.Chain.BVSDirectory).GetBVSInfo(core.C.Chain.BVSHash)
	if err != nil {
		panic(err)
	}
	return &Monitor{
		bvsContract: txResp.BVSContract,
		chainIO:     client,
	}
}

// Run runs the event indexer and monitors for new task created and task responded events.
//
// No parameters.
// No return values.
func (m *Monitor) Run() {
	ctx := context.Background()

	res, err := m.chainIO.QueryNodeStatus(ctx)
	if err != nil {
		panic(err)
	}
	latestBlock := res.SyncInfo.LatestBlockHeight
	fmt.Println("latestBlock: ", latestBlock)

	// create BVSContract indexer
	evtIndexer := indexer.NewEventIndexer(
		m.chainIO.GetClientCtx(),
		m.bvsContract,
		latestBlock,
		[]string{"wasm-NewTaskCreated", "wasm-TaskResponded"},
		1,
		5)
	evtChain, err := evtIndexer.Run(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Println("chain: ", evtChain)

	// monitor the indexer event
	for evt := range evtChain {
		switch evt.EventType {
		case "wasm-NewTaskCreated":
			blockHeight := evt.BlockHeight
			txnHash := evt.TxHash
			taskId := evt.AttrMap["taskId"]
			taskInput := evt.AttrMap["input"]
			fmt.Printf("[NewTaskCreated] blockHeight: %d, txnHash: %s, taskId: %s, taskInput: %s\n", blockHeight, txnHash, taskId, taskInput)
		case "wasm-TaskResponded":
			blockHeight := evt.BlockHeight
			txnHash := evt.TxHash
			taskId := evt.AttrMap["taskId"]
			taskResult := evt.AttrMap["result"]
			taskOperators := evt.AttrMap["operators"]
			fmt.Printf("[TaskResponded] blockHeight: %d, txnHash: %s, taskId: %s, taskResult: %s, taskOperators: %s\n", blockHeight, txnHash, taskId, taskResult, taskOperators)
		default:
			fmt.Printf("Unknown event type. evt: %+v\n", evt)
		}
	}
}
