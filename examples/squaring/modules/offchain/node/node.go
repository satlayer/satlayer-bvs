package node

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	rio "io"
	"net/http"
	"strconv"
	"sync"
	"time"

	
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/indexer"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	

	"github.com/satlayer/satlayer-bvs/examples/squaring/offchain/core"
)

type Node struct {
	bvsContract string
	pubKeyStr   string
	chainIO     io.ChainIO
}

type Payload struct {
	TaskID    int64  `json:"task_id"`
	Result    int64  `json:"result"`
	Timestamp int64  `json:"timestamp"`
	Signature string `json:"signature"`
	PubKey    string `json:"pub_key"`
}

var wasmUpdateState sync.Map

// NewNode creates a new Node instance with the given configuration.
//
// It initializes a new Cosmos client, retrieves the account, and sets up the BVS contracts and state bank.
// Returns a pointer to the newly created Node instance.
func NewNode() *Node {
	elkLogger := logger.NewELKLogger("bvs_demo")
	elkLogger.SetLogLevel("info")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Owner.KeyDir, core.C.Owner.Bech32Prefix, elkLogger, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	chainIO, err = chainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	if err != nil {
		panic(err)
	}

	pubKey := chainIO.GetCurrentAccountPubKey()

	pubKeyStr := base64.StdEncoding.EncodeToString(pubKey.Bytes())

	return &Node{
		bvsContract: core.C.Chain.BVSContract,
		chainIO:     chainIO,
		pubKeyStr:   pubKeyStr,
	}
}

// Run starts the node's main execution loop.
//
// ctx is the context for the Run function.
// No return value.
func (n *Node) Run(ctx context.Context) {
	if err := n.syncState(ctx); err != nil {
		panic(err)
	}
	n.monitorDriver(ctx)
}

// syncState synchronizes the state with the latest blockchain state from the BVS Contract.
//
// ctx is the context for the syncState function.
// Returns an error if the synchronization fails.
func (n *Node) syncState(ctx context.Context) (err error) {
	res, err := n.chainIO.QueryNodeStatus(ctx)
	if err != nil {
		panic(err)
	}
	latestBlock := res.SyncInfo.LatestBlockHeight

	idx := indexer.NewEventIndexer(n.chainIO.GetClientCtx(), n.bvsContract, latestBlock, []string{"wasm-UpdateState"}, 1, 10)
	processingQueue, err := idx.Run(ctx)
	if err != nil {
		panic(err)
	}

	go func() {
		for event := range processingQueue {
			key, ok := event.AttrMap["key"]
			if !ok {
				continue
			}
			val, ok := event.AttrMap["value"]
			if !ok {
				continue
			}
			wasmUpdateState.Store(key, val)
		}
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
	res, err := n.chainIO.QueryNodeStatus(ctx)
	if err != nil {
		panic(err)
	}
	latestBlock := res.SyncInfo.LatestBlockHeight
	fmt.Println("latestBlock: ", latestBlock)

	evtIndexer := indexer.NewEventIndexer(
		n.chainIO.GetClientCtx(),
		n.bvsContract,
		latestBlock,
		[]string{"wasm-ExecuteBVSOffchain"},
		1,
		10)
	evtChain, err := evtIndexer.Run(ctx)
	if err != nil {
		panic(err)
	}

	fmt.Println("chain: ", evtChain)

	for evt := range evtChain {
		if evt.AttrMap["sender"] != n.bvsContract {
			continue
		}
		switch evt.EventType {
		case "wasm-ExecuteBVSOffchain":
			time.Sleep(5 * time.Second)
			taskId := evt.AttrMap["task_id"]
			fmt.Println("taskId: ", taskId)
			if err := n.calcTask(fmt.Sprintf("%s", taskId)); err != nil {
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
	value, exists := wasmUpdateState.Load(stateKey)
	if !exists {
		return
	}

	input, err := strconv.Atoi(value.(string))
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
	msgPayload := fmt.Sprintf("%s-%d-%d-%d", core.C.Chain.BVSContract, nowTs, taskID, result)
	core.L.Info(fmt.Sprintf("msgPayload: %s\n", msgPayload))
	signature, err := n.chainIO.GetSigner().Sign([]byte(msgPayload))

	payload := Payload{
		TaskID:    taskID,
		Result:    result,
		Timestamp: nowTs,
		Signature: signature,
		PubKey:    n.pubKeyStr,
	}
	fmt.Printf("task result send aggregator payload: %+v\n", payload)
	if err != nil {
		return
	}
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
