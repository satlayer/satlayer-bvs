package e2e

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/satlayer/satlayer-bvs/examples/squaring/internal/tests"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/redis"

	"github.com/gin-gonic/gin"
	"golang.org/x/exp/rand"

	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/api"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/core"
)

const (
	keyDir = "../../../.babylond"
)

type Payload struct {
	TaskID    uint64 `json:"task_id" binding:"required"`
	Result    int64  `json:"result" binding:"required"`
	Timestamp int64  `json:"timestamp" binding:"required"`
	Signature string `json:"signature" binding:"required"`
	PubKey    string `json:"pub_key" binding:"required"`
}

type aggregatorTestSuite struct {
	tests.TestSuite
}

func (suite *aggregatorTestSuite) SetupSuite() {
	suite.TestSuite.SetupSuite(keyDir, "operator2")

	suite.Babylond.FundAddressUbbn("bbn1nrueqkp0wmujyxuqp952j8mnxngm5gek3fsgrj", 1e8)
}

// entrypoint for the test suite
func TestAggregator(t *testing.T) {
	suite.Run(t, new(aggregatorTestSuite))
}

func (suite *aggregatorTestSuite) TestExecuteAggregator() {
	t := suite.T()

	// get public key
	pubKey := suite.ChainIO.GetCurrentAccountPubKey()
	pubKeyStr := base64.StdEncoding.EncodeToString(pubKey.Bytes())
	account, err := suite.ChainIO.GetCurrentAccount()
	assert.NoError(t, err)
	address := account.GetAddress().String()

	// get random task id
	randTaskID := rand.Uint64()

	// get random task result
	randResult := rand.Int63()

	// get message to sign
	now := time.Now().Unix()
	msgPayload := fmt.Sprintf("%s-%d-%d-%d", core.C.Chain.BvsHash, now, randTaskID, randResult)

	// sign message
	signature, err := suite.ChainIO.GetSigner().Sign([]byte(msgPayload))
	assert.NoError(t, err)

	// construct payload to send to aggregator api
	payload := Payload{
		TaskID:    randTaskID,
		Result:    randResult,
		Timestamp: now,
		Signature: signature,
		PubKey:    pubKeyStr,
	}

	res := sendTask(t, payload)

	// check if the response status code is 200
	assert.Equal(t, http.StatusOK, res.Code)

	// assert that 1 task is saved to the queue (REDIS)
	listLength, err := core.S.RedisConn.LLen(context.Background(), core.PKTaskQueue).Result()
	assert.NoError(t, err)
	assert.Equal(t, int64(1), listLength)

	// assert that the task saved to the queue is the same as the one sent
	rawTasks, err := core.S.RedisConn.LRange(context.Background(), core.PKTaskQueue, 0, 1).Result()
	assert.NoError(t, err)

	var task core.Task
	err = json.Unmarshal([]byte(rawTasks[0]), &task)
	assert.NoError(t, err)

	assert.Equal(t, task, core.Task{
		TaskID: randTaskID,
		TaskResult: core.TaskResult{
			Operator: address,
			Result:   randResult,
		},
	})
}

// sendTask sends a task to the aggregator api
func sendTask(t *testing.T, tpayload Payload) *httptest.ResponseRecorder {
	router := gin.Default()
	// setup routes
	api.SetupRoutes(router)

	jsonData, err := json.Marshal(tpayload)
	assert.NoError(t, err)

	req, _ := http.NewRequest("POST", "/api/aggregator", bytes.NewBuffer(jsonData))

	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	assert.NoError(t, err)

	return w
}

type RedisContainer struct {
	Container testcontainers.Container
	Host      string
	Port      string
}

func startRedis() (*RedisContainer, error) {
	ctx := context.Background()

	rc, err := redis.Run(ctx,
		"redis:7",
		redis.WithLogLevel(redis.LogLevelVerbose),
	)

	if err != nil {
		log.Printf("failed to start container: %s", err)
		return nil, err
	}

	host, err := rc.Host(context.Background())
	if err != nil {
		log.Fatalf("failed to get redis container host: %s", err)
		return nil, err
	}

	port, err := rc.MappedPort(context.Background(), "6379")
	if err != nil {
		log.Fatalf("failed to get redis container port: %s", err)
		return nil, err
	}
	return &RedisContainer{
		Container: rc,
		Host:      host,
		Port:      port.Port(),
	}, nil
}
