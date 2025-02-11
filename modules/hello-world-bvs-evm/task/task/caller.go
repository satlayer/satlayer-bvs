package task

import (
	"context"
	"fmt"
	"math/rand"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	chainioabi "github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/bvssquaringapi"
	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/task/core"
)

type Caller struct {
	bvsContract common.Address
	chainIO     io.ETHChainIO
}

// RunCaller runs the caller by creating a new caller and executing its Run method.
//
// No parameters.
// No return.
func RunCaller() {
	c := NewCaller()
	c.Run()
}

// NewCaller creates a new Caller instance.
//
// Returns a pointer to Caller.
func NewCaller() *Caller {
	// init log and chain
	elkLogger := logger.NewELKLogger("bvs_demo_eth")
	elkLogger.SetLogLevel("info")

	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo_eth")
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
	fmt.Printf("bvs contract: %s\n", txResp.BVSContract.String())
	return &Caller{
		bvsContract: txResp.BVSContract,
		chainIO:     ethChainIO,
	}
}

// Run runs the caller in an infinite loop, creating a new task with a random number every second.
//
// No parameters.
// No return.
func (c *Caller) Run() {
	contractABI, err := chainioabi.GetContractABI("../data/abi", "BVSSquaring")
	if err != nil {
		panic(err)
	}
	bvsSquaring := bvssquaringapi.NewBVSSquaringImpl(c.chainIO, c.bvsContract, contractABI)
	fromAddr := common.HexToAddress(core.C.Owner.UserAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      core.C.Owner.Password,
	}
	for {
		//bvsSquaring.BindClient(c.bvsContract)
		randomNumber := rand.Int63n(100)
		resp, err := bvsSquaring.CreateNewTask(context.Background(), wallet, randomNumber)
		if err != nil {
			panic(err)
		}
		fmt.Printf("resp: %s\n", resp.TxHash.String())
		time.Sleep(3 * time.Second)
	}
}
