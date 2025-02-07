package metrics

import (
	"context"

	"github.com/prometheus/client_golang/prometheus"

	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/logging"
	"github.com/satlayer/satlayer-bvs/bvs-api/metrics"
	satlayerdefault "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/satlayer_default"
)

// SatLayerMetrics contains instrumented metrics that should be incremented by the bvs node using the methods below
type SatLayerMetrics struct {
	metrics metrics.Metrics
	// metrics
	// fees are not yet turned on, so these should just be 0 for the time being
	satlayerIndicators *satlayerdefault.PromIndicators
}

var _ Metrics = (*SatLayerMetrics)(nil)

func NewSatLayerMetrics(bvsName, ipPortAddress string, reg prometheus.Registerer, logger logging.Logger) *SatLayerMetrics {
	satLayerIndicators := satlayerdefault.NewPromIndicators(bvsName, reg)
	satLayerMetrics := metrics.NewSatLayerMetrics(ipPortAddress, logger.GetElkLogger())
	return &SatLayerMetrics{
		metrics:            satLayerMetrics,
		satlayerIndicators: satLayerIndicators,
	}
}

// AddFeeEarnedTotal adds the fee earned to the total fee earned metric
func (m *SatLayerMetrics) AddFeeEarnedTotal(amount float64, token string) {
	m.satlayerIndicators.AddFeeEarnedTotal(amount, token)
}

// SetPerformanceScore sets the performance score of the node
func (m *SatLayerMetrics) SetPerformanceScore(score float64) {
	m.satlayerIndicators.SetPerformanceScore(score)
}

// Start creates a http handler for reg and starts the prometheus server in a goroutine, listening at m.ipPortAddress.
// reg needs to be the prometheus registry that was passed in the NewSatLayerMetrics constructor
func (m *SatLayerMetrics) Start(ctx context.Context, reg prometheus.Gatherer) <-chan error {
	return m.metrics.Start(ctx, reg)
}
