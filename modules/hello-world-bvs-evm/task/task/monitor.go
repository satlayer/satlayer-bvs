package task

import (
	"context"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	chainioabi "github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/task/core"
)

type Monitor struct {
	bvsContract common.Address
	chainIO     io.ETHChainIO
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
	ethChainIO, err := io.NewETHChainIO(core.C.Chain.RPC, core.C.Owner.KeyDir, elkLogger, metricsIndicators, types.TxManagerParams{
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
	contractABI, err := chainioabi.GetContractABI("../data/abi", "BVSDirectory")
	directoryContract := common.HexToAddress(core.C.Chain.BVSDirectory)
	ctx := context.Background()
	directory := api.NewETHBVSDirectoryImpl(ethChainIO, directoryContract, contractABI)
	txResp, err := directory.GetBVSInfo(ctx, core.C.Chain.BVSHash)
	if err != nil {
		panic(err)
	}
	// create BVSDirectory
	return &Monitor{
		bvsContract: txResp.BVSContract,
		chainIO:     ethChainIO,
	}
}

// Run runs the event indexer and monitors for new task created and task responded events.
//
// No parameters.
// No return values.
func (m *Monitor) Run() {
	ctx := context.Background()

	contractABI, err := chainioabi.GetContractABI("../data/abi", "BVSSquaring")
	if err != nil {
		panic(err)
	}
	latestBlock, err := m.chainIO.GetLatestBlockNumber(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Println("latestBlock: ", latestBlock)

	// create BVSContract indexer
	evtIndexer := indexer.NewETHIndexer(
		m.chainIO.GetETHClient(),
		contractABI,
		m.bvsContract,
		latestBlock,
		[]common.Hash{common.HexToHash("0x781467e43ac868ad03a8d5d6c9c8a133d42b05abc31986a209b22a1d409ab9f8"), common.HexToHash("0xfafacb1d7ea2169296baa8efcf9be75f0ff319910ea33328535a2bf4a0980be2")},
		1,
		5)
	evtChain, err := evtIndexer.Run(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Println("chain: ", evtChain)

	// monitor the indexer event
	for evt := range evtChain {
		//fmt.Printf("evt: %v", evt)
		switch evt.EventType {
		case "CreateNewTask":
			blockHeight := evt.BlockHeight
			txnHash := evt.TxHash
			taskId := evt.AttrMap["taskId"]
			taskInput := evt.AttrMap["input"]
			fmt.Printf("[NewTaskCreated] blockHeight: %d, txnHash: %s, taskId: %s, taskInput: %s\n", blockHeight, txnHash, taskId, taskInput)
		case "TaskResponded":
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
