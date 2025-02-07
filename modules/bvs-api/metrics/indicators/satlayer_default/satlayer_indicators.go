package satlayerdefault

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"

	"github.com/satlayer/satlayer-bvs/bvs-api/metrics/consts"
)

type PromIndicators struct {
	feeEarnedTotal   *prometheus.CounterVec
	performanceScore prometheus.Gauge
}

func NewPromIndicators(bvsName string, reg prometheus.Registerer) *PromIndicators {
	indicator := &PromIndicators{
		feeEarnedTotal: promauto.With(reg).NewCounterVec(
			prometheus.CounterOpts{
				Namespace:   consts.SatLayerPromNamespace,
				Name:        "fees_earned_total",
				Help:        "The amount of fees earned in <token>",
				ConstLabels: prometheus.Labels{"bvs_name": bvsName},
			},
			[]string{"token"},
		),
		performanceScore: promauto.With(reg).NewGauge(
			prometheus.GaugeOpts{
				Namespace:   consts.SatLayerPromNamespace,
				Name:        "performance_score",
				Help:        "The performance metric is a score between 0 and 100 and each developer can define their own way of calculating the score. The score is calculated based on the performance of the Node and the performance of the backing services.",
				ConstLabels: prometheus.Labels{"bvs_name": bvsName},
			},
		),
	}
	indicator.initPerformanceScore()
	return indicator
}

func (s *PromIndicators) initPerformanceScore() {
	// Performance score starts as 100, and goes down if node doesn't perform well
	s.performanceScore.Set(100)
}

// AddFeeEarnedTotal Total cost indicator
func (s *PromIndicators) AddFeeEarnedTotal(amount float64, token string) {
	s.feeEarnedTotal.WithLabelValues(token).Add(amount)
}

// SetPerformanceScore Performance indicators
func (s *PromIndicators) SetPerformanceScore(score float64) {
	s.performanceScore.Set(score)
}
