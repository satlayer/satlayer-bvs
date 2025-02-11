package node

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	rio "io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/prometheus/client_golang/prometheus"
	chainioabi "github.com/satlayer/satlayer-bvs/bvs-api/chainio/abi"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/offchain/core"
)

type Node struct {
	bvsContract common.Address
	chainIO     io.ETHChainIO
	stateBank   api.ETHStateBank
	wallet      types.ETHWallet
}

type Payload struct {
	TaskID    int64  `json:"task_id"`
	Result    int64  `json:"result"`
	Timestamp int64  `json:"timestamp"`
	Signature string `json:"signature"`
	UserAddr  string `json:"user_addr"`
}

// NewNode creates a new Node instance with the given configuration.
//
// It initializes a new Cosmos client, retrieves the account, and sets up the BVS contracts and state bank.
// Returns a pointer to the newly created Node instance.
func NewNode() *Node {
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

	//pubKeyStr := base64.StdEncoding.EncodeToString(pubKey.Bytes())
	contractABI, err := chainioabi.GetContractABI("../data/abi", "BVSDirectory")
	directoryContract := common.HexToAddress(core.C.Chain.BVSDirectory)
	ctx := context.Background()
	directory := api.NewETHBVSDirectoryImpl(ethChainIO, directoryContract, contractABI)
	txResp, err := directory.GetBVSInfo(ctx, core.C.Chain.BVSHash)
	if err != nil {
		panic(err)
	}

	stateBankContractABI, err := chainioabi.GetContractABI("../data/abi", "StateBank")
	stateBankContract := common.HexToAddress(core.C.Chain.StateBank)

	stateBank := api.NewETHStateBankImpl(ethChainIO, stateBankContract, stateBankContractABI)

	//ks := keystore.NewKeyStore(core.C.Owner.KeyDir, keystore.LightScryptN, keystore.LightScryptP)
	//address := common.HexToAddress(core.C.Owner.UserAddr)
	//account := accounts.Account{Address: address}
	//owner, err := ks.Find(account)
	if err != nil {
		panic(err)
	}
	fromAddr := common.HexToAddress(core.C.Owner.UserAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      core.C.Owner.Password,
	}

	return &Node{
		bvsContract: txResp.BVSContract,
		stateBank:   stateBank,
		chainIO:     ethChainIO,
		wallet:      wallet,
	}
}

// Run starts the node's main execution loop.
//
// ctx is the context for the Run function.
// No return value.
func (n *Node) Run(ctx context.Context) {
	if err := n.syncStateBank(ctx); err != nil {
		panic(err)
	}
	n.monitorDriver(ctx)
}

// syncStateBank synchronizes the state bank with the latest blockchain state.
//
// ctx is the context for the syncStateBank function.
// Returns an error if the synchronization fails.
func (n *Node) syncStateBank(ctx context.Context) (err error) {
	latestBlock, err := n.chainIO.GetLatestBlockNumber(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Println("latestBlock: ", latestBlock)
	idx := n.stateBank.Indexer(
		n.chainIO.GetETHClient(),
		n.bvsContract.String(),
		latestBlock,
		[]common.Hash{common.HexToHash("0x6c94acf0e05d6bee21156de7d7badca3e844eb6b31df94a1bdf9bbf3cd2847de")},
		1,
		10)
	processingQueue, err := idx.Run(ctx)
	if err != nil {
		panic(err)
	}

	go func() {
		n.stateBank.EventHandler(processingQueue)
	}()

	ticker := time.NewTicker(time.Second * 2)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if idx.IsUpToDate {
				return
			}
		}
	}
}

// monitorDriver monitors the driver contract for events and performs actions based on the event type.
//
// ctx is the context for the monitorDriver function.
// Returns an error if there is an issue with the monitoring process.
func (n *Node) monitorDriver(ctx context.Context) {
	latestBlock, err := n.chainIO.GetLatestBlockNumber(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Println("latestBlock: ", latestBlock)
	driverContractABI, err := chainioabi.GetContractABI("../data/abi", "BVSDriver")
	driverontract := common.HexToAddress(core.C.Chain.BVSDriver)
	evtIndexer := indexer.NewETHIndexer(
		n.chainIO.GetETHClient(),
		driverContractABI,
		driverontract,
		latestBlock,
		[]common.Hash{common.HexToHash("0xa7da4da51c1af4d368e16b2b57532e13b0ac629a6c2215d32958f8e53edd0e3a")},
		1,
		10)
	evtChain, err := evtIndexer.Run(ctx)
	if err != nil {
		panic(err)
	}

	fmt.Println("chain: ", evtChain)

	for evt := range evtChain {
		sender := fmt.Sprintf("%v", evt.AttrMap["sender"])
		if strings.ToLower(sender) != strings.ToLower(n.bvsContract.String()) {
			continue
		}
		switch evt.EventType {
		case "ExecuteBvsOffchain":
			time.Sleep(5 * time.Second)
			taskId := evt.AttrMap["taskId"]
			if err := n.calcTask(fmt.Sprintf("%v", taskId)); err != nil {
				fmt.Println("ExecuteBVSOffchain error: ", err)
			}
		default:
			fmt.Println("unhandled event: ", evt.EventType)
		}
	}
}

// calcTask calculates the task result and sends it to the aggregator.
//
// taskId is the unique identifier of the task.
// Returns an error if there is an issue with the calculation or sending process.
func (n *Node) calcTask(taskId string) (err error) {
	stateKey := fmt.Sprintf("taskId.%s", taskId)
	fmt.Printf("stateKey: %s\n", stateKey)
	value, err := n.stateBank.GetEthUpdateState(stateKey)
	if err != nil {
		return
	}

	input, err := strconv.Atoi(value)
	task, err := strconv.Atoi(taskId)
	if err != nil {
		fmt.Println("format err:", err)
		return
	}

	result := n.square(int64(input))
	err = n.sendAggregator(int64(task), result)
	if err != nil {
		panic(err)
	}
	return
}

// square calculates the square of a given integer.
//
// input is the number to be squared.
// Returns the squared result as an int64.
func (n *Node) square(input int64) int64 {
	return input * input
}

// sendAggregator sends the task result to the aggregator.
//
// taskID is the unique identifier of the task.
// result is the calculated result of the task.
// Returns an error if there is an issue with the sending process.
func (n *Node) sendAggregator(taskID int64, result int64) (err error) {
	nowTs := time.Now().Unix()
	msgPayload := fmt.Sprintf("%s-%d-%d-%d", core.C.Chain.BVSHash, nowTs, taskID, result)
	core.L.Info(fmt.Sprintf("msgPayload: %s\n", msgPayload))
	hash := crypto.Keccak256Hash([]byte(msgPayload))

	signature, err := n.chainIO.SignHash(n.wallet, hash.Bytes())
	if err != nil {
		return
	}
	hexString := hex.EncodeToString(signature)
	payload := Payload{
		TaskID:    taskID,
		Result:    result,
		Timestamp: nowTs,
		Signature: hexString,
		UserAddr:  core.C.Owner.UserAddr,
	}
	fmt.Printf("task result send aggregator payload: %+v\n", payload)
	jsonData, err := json.Marshal(payload)
	if err != nil {
		fmt.Printf("Error marshaling JSON: %s", err)
		return
	}

	resp, err := http.Post(core.C.Aggregator.URL, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		fmt.Printf("Error sending aggregator : %s\n", err)
		return
	}
	if resp.StatusCode != 200 {
		body, _ := rio.ReadAll(resp.Body)
		fmt.Printf("Error sending aggregator : %s\n", string(body))
		return
	}
	return
}
