package metrics

import (
	"context"

	"github.com/prometheus/client_golang/prometheus"
)

// Metrics is the interface for the SatLayerMetrics server
type Metrics interface {
	AddFeeEarnedTotal(amount float64, token string)
	SetPerformanceScore(score float64)
	Start(ctx context.Context, reg prometheus.Gatherer) <-chan error
}
