package satlayerdefault

import (
	"testing"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
)

type SatlayerIndicatorsTestSuite struct {
	suite.Suite
	reg        *prometheus.Registry
	indicators *PromIndicators
}

// this runs before all tests to initialize reg and metrics
func (suite *SatlayerIndicatorsTestSuite) SetupTest() {
	suite.reg = prometheus.NewRegistry()
	suite.indicators = NewPromIndicators("localbvs", suite.reg)
}

func (suite *SatlayerIndicatorsTestSuite) Test_AddEigenFeeEarnedTotal() {
	suite.indicators.AddFeeEarnedTotal(1, "token-address")
	assert.Equal(suite.T(), 1.0, testutil.ToFloat64(suite.indicators.feeEarnedTotal.WithLabelValues("token-address")))
}

func (suite *SatlayerIndicatorsTestSuite) Test_SetEigenPerformanceScore() {
	suite.indicators.SetPerformanceScore(1)
	assert.Equal(suite.T(), 1.0, testutil.ToFloat64(suite.indicators.performanceScore))
}

func TestSatlayerIndicatorsTestSuite(t *testing.T) {
	suite.Run(t, new(SatlayerIndicatorsTestSuite))
}
