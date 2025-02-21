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
	"os"
	"testing"
	"time"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/redis"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"golang.org/x/exp/rand"

	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/api"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/core"
	"github.com/satlayer/satlayer-bvs/examples/squaring/aggregator/svc"
)

type Payload struct {
	TaskID    uint64 `json:"task_id" binding:"required"`
	Result    int64  `json:"result" binding:"required"`
	Timestamp int64  `json:"timestamp" binding:"required"`
	Signature string `json:"signature" binding:"required"`
	PubKey    string `json:"pub_key" binding:"required"`
}

// TestAggregator tests the functionality of the aggregator.
//
// t is the testing object provided by Go's testing package.

// TODO(kevin): to fix, disabled due to rename of JSON and Struct
func testAggregator(t *testing.T) {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../../../.babylond"
	keyName := "operator2"

	redisContainer, err := startRedis()
	if err != nil {
		t.Fatalf("failed to start redis container: %s", err)
		return
	}

	redisUrl := fmt.Sprintf("%s:%s", redisContainer.Host, redisContainer.Port)

	err = os.Setenv("KEY_DIR", homeDir)
	if err != nil {
		return
	}
	core.InitConfig(core.Config{
		Database: core.Database{
			RedisHost: redisUrl,
		},
	})
	svc.InitMonitor()

	elkLogger := logger.NewMockELKLogger()
	reg := prometheus.NewRegistry()
	metricsIndicators := transactionprocess.NewPromIndicators(reg, "bvs_demo")
	cs, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", elkLogger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		t.Fatalf("failed to create chain IO: %s", err)
		return
	}

	cs, err = cs.SetupKeyring(keyName, "test")
	if err != nil {
		t.Fatalf("failed to setup keyring: %s", err)
		return
	}

	pubKey := cs.GetCurrentAccountPubKey()
	pubKeyStr := base64.StdEncoding.EncodeToString(pubKey.Bytes())

	router := gin.Default()
	// setup routes
	api.SetupRoutes(router)
	rand.Seed(uint64(time.Now().UnixNano()))
	i := rand.Intn(100000)
	nowTS := time.Now().Unix()
	msgPayload := fmt.Sprintf("%s-%d-%d-%d", core.C.Chain.BvsHash, nowTS, i, i)
	t.Logf("msgPayload: %s\n", msgPayload)
	signature, err := cs.GetSigner().Sign([]byte(msgPayload))
	t.Logf("signature: %+v\n", signature)
	payload := Payload{
		TaskID:    uint64(i),
		Result:    int64(i),
		Timestamp: nowTS,
		Signature: signature,
		PubKey:    pubKeyStr,
	}
	t.Logf("payload: %+v\n", payload)
	if err != nil {
		t.Fatalf("failed to sign: %v\n", err)
		return
	}
	sendTaskResult(payload, router, t)
}

// sendTaskResult sends a task result to the aggregator API.
//
// payload is the task result payload to be sent.
// t is the testing object provided by Go's testing package.
func sendTaskResult(payload Payload, router *gin.Engine, t *testing.T) {
	jsonData, err := json.Marshal(payload)
	if err != nil {
		fmt.Printf("Error marshaling JSON: %s", err)
		return
	}
	req, _ := http.NewRequest("POST", "http://localhost:9090/api/aggregator", bytes.NewBuffer(jsonData))

	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	if err != nil {
		t.Fatalf("Error sending POST request: %s", err)
		return
	}
	if w.Code != http.StatusOK {
		t.Errorf("Expected status code %d, got %d", http.StatusOK, w.Code)
	}

	t.Logf("Response Body: %s\n", w.Body.String())
	t.Logf("")
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
