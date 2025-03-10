package svc

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/core"
	"github.com/satlayer/satlayer-bvs/examples/squaring/bvssquaringapi"
)

var MonitorInstance Monitor

func InitMonitor() {
	MonitorInstance = *NewMonitor()
}

type Monitor struct {
	bvsContract  string
	directoryApi *api.Directory
	chainIO      io.ChainIO
}

// NewMonitor creates a new Monitor instance with a Cosmos client and BVS contract.
//
// It takes no parameters.
// Returns a pointer to a Monitor struct.
func NewMonitor() *Monitor {
	keyDir := os.Getenv("KEY_DIR")
	if keyDir == "" {
		keyDir = core.C.Owner.KeyDir
	}
	fmt.Printf("homeDir: %s\n", keyDir)

	// init log and chain
	elkLogger := logger.NewELKLogger(core.C.Chain.BvsContract)
	elkLogger.SetLogLevel("info")
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, keyDir, core.C.Owner.Bech32Prefix, elkLogger, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	// setup running key
	chainIO, err = chainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	if err != nil {
		panic(err)
	}

	bvsDirectoryApi := api.NewDirectory(chainIO, core.C.Chain.BvsDirectory)

	return &Monitor{
		bvsContract:  core.C.Chain.BvsContract,
		directoryApi: bvsDirectoryApi,
		chainIO:      chainIO,
	}
}

// Run starts the task queue monitoring process.
//
// It takes a context.Context object as a parameter.
// No return values.
func (m *Monitor) Run(ctx context.Context) {
	core.L.Info("Start to monitor task queue")
	for {
		results, err := core.S.RedisConn.BLPop(context.Background(), 0, core.PKTaskQueue).Result()
		fmt.Printf("results: %+v\n", results)
		if err != nil {
			core.L.Error(fmt.Sprintf("Failed to read task queue, due to {%s}", err))
			continue
		}
		fmt.Printf("result--->: %s\n", results[1])

		task := core.Task{}
		if err := json.Unmarshal([]byte(results[1]), &task); err != nil {
			core.L.Error(fmt.Sprintf("Failed to parse task queue, due to {%s}", err))
			continue
		}
		fmt.Printf("task: %+v\n", task)

		pkTaskResult := fmt.Sprintf("%s%d", core.PKTaskResult, task.TaskID)
		taskResultStr, err := json.Marshal(task.TaskResult)
		if err != nil {
			core.L.Error(fmt.Sprintf("Failed to marshal task result, due to {%s}", err))
			return
		}

		if err := core.S.RedisConn.LPush(ctx, pkTaskResult, taskResultStr).Err(); err != nil {
			core.L.Error(fmt.Sprintf("Failed to save task result, due to {%s}", err))
			return
		}
		m.verifyTask(ctx, task.TaskID)
	}
}

// verifyTask is a method of the Monitor struct. It is responsible for verifying a task
// by reading the task result from Redis and checking if the result has reached a
// certain threshold. If the threshold is met, it sets the task as finished in Redis,
// logs the task result and operators, and sends the task result to a specified
// destination.
//
// The function takes a context.Context object and an uint64 taskId as parameters.
// It does not return anything.
func (m *Monitor) verifyTask(ctx context.Context, taskId uint64) {
	pkTaskResult := fmt.Sprintf("%s%d", core.PKTaskResult, taskId)
	// timer to read redis queue and verify the task result
	results, err := core.S.RedisConn.LRange(ctx, pkTaskResult, 0, -1).Result()
	fmt.Printf("verify results: %s\n", results)
	if err != nil {
		core.L.Error(fmt.Sprintf("Failed to read task result, due to {%s}", err))
		return
	}

	resultCntMap := make(map[int64]uint)
	resultOperatorMap := make(map[int64][]string)
	var taskResult core.TaskResult

	for _, result := range results {
		fmt.Printf("verify result: %s\n", result)
		if err := json.Unmarshal([]byte(result), &taskResult); err != nil {
			core.L.Error(fmt.Sprintf("Failed to parse task result, due to {%s}", err))
			return
		}

		resultCntMap[taskResult.Result]++
		resultOperatorMap[taskResult.Result] = append(resultOperatorMap[taskResult.Result], taskResult.Operator)

		if resultCntMap[taskResult.Result] >= core.C.App.Threshold {
			pkTaskFinished := fmt.Sprintf("%s%d", core.PKTaskFinished, taskId)
			if err := core.S.RedisConn.Set(ctx, pkTaskFinished, taskResult.Result, 0).Err(); err != nil {
				core.L.Error(fmt.Sprintf("Failed to set task finished, due to {%s}", err))
				return
			}
			operators := strings.Join(resultOperatorMap[taskResult.Result], "&")
			core.L.Info(fmt.Sprintf("Task {%d} is finished. The result is {%d}. The operators are {%s}", taskId, taskResult.Result, operators))
			if err := m.sendTaskResult(taskId, taskResult.Result, operators); err != nil {
				core.L.Error(fmt.Sprintf("Failed to send task result, due to {%s}", err))
			}

			pkTaskOperator := fmt.Sprintf("%s%d", core.PKTaskOperator, taskId)
			core.S.RedisConn.Del(ctx, pkTaskResult)
			core.S.RedisConn.Del(ctx, pkTaskOperator)
			break
		}
	}
}

// sendTaskResult sends the task result to BVS Squaring API.
//
// taskId: the unique identifier of the task
// result: the result of the task
// operators: the operators involved in the task
// error: an error if the task result sending fails
func (m *Monitor) sendTaskResult(taskId uint64, result int64, operators string) error {
	fmt.Println("sendTaskResult", taskId, result, operators)

	bvsSquaring := bvssquaringapi.NewBVSSquaring(m.chainIO)
	bvsSquaring.BindClient(m.bvsContract)
	_, err := bvsSquaring.RespondToTask(context.Background(), int64(taskId), result, operators)
	if err != nil {
		return err
	}

	return nil
}
