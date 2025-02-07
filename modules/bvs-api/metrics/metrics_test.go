package metrics_test

import (
	"context"
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	"github.com/satlayer/satlayer-bvs/bvs-api/metrics"
	satlayerdefault "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/satlayer_default"
)

type SatlayerMetricsTestSuite struct {
	suite.Suite
	reg         *prometheus.Registry
	logger      logger.Logger
	testAddress string
	bvsName     string
}

func (suite *SatlayerMetricsTestSuite) SetupTest() {
	// Create a Prometheus registry
	suite.reg = prometheus.NewRegistry()
	// Mock logger, you can implement it according to the actual logger package
	suite.logger = logger.NewMockELKLogger()
	// Create a test address with IP and port
	suite.testAddress = "localhost:8090"
	suite.bvsName = "localtest"
}

func (suite *SatlayerMetricsTestSuite) Test_Start() {
	// add indicators
	satlayerIndicators := satlayerdefault.NewPromIndicators(suite.bvsName, suite.reg)
	satlayerIndicators.AddFeeEarnedTotal(1, "token-address")

	// Initialize the metrics structure
	metricsServer := metrics.NewSatLayerMetrics(suite.testAddress, suite.logger)

	// Create a context with cancellation capability
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start the metrics server
	errChan := metricsServer.Start(ctx, suite.reg)

	// Wait for the server to start
	time.Sleep(1 * time.Second)

	// Send an HTTP request to the "/metrics" endpoint
	resp, err := http.Get("http://" + suite.testAddress + "/metrics")
	assert.NoError(suite.T(), err, "failed to make request to /metrics")
	defer resp.Body.Close()

	// Assert that the server responded successfully
	assert.Equal(suite.T(), http.StatusOK, resp.StatusCode, "metrics endpoint should return 200 OK")

	body, err := io.ReadAll(resp.Body)
	assert.NoError(suite.T(), err)

	assert.Contains(suite.T(), string(body), "satlayer_fees_earned_total")
	assert.Contains(suite.T(), string(body), "satlayer_performance_score")

	// Cancel the context to test if the server can gracefully shut down
	cancel()

	// Wait for the error signal, if there is an error, the test fails
	select {
	case err := <-errChan:
		if err != nil {
			suite.T().Fatalf("server failed with error: %v", err)
		}
	case <-time.After(2 * time.Second):
		suite.T().Fatal("server shutdown timed out")
	}
}

func TestSatlayerMetricsTestSuite(t *testing.T) {
	suite.Run(t, new(SatlayerMetricsTestSuite))
}
