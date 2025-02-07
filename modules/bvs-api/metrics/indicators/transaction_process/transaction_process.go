package transactionprocess

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"

	"github.com/satlayer/satlayer-bvs/bvs-api/metrics/consts"
)

type Indicators interface {
	ObserveBroadcastLatencyMs(latencyMs int64)
	ObserveConfirmationLatencyMs(latencyMs int64)
	ObserveGasUsedOsmo(gasUsedUosmo uint64)
	ObserveSpeedups(speedUps int)
	IncrementProcessingTxCount()
	DecrementProcessingTxCount()
	IncrementProcessedTxsTotal(state string)
}

type PromIndicators struct {
	broadcastLatencyMs    prometheus.Summary
	confirmationLatencyMs prometheus.Summary
	gasUsedOsmo           prometheus.Summary
	speedUps              prometheus.Histogram
	processingTxCount     prometheus.Gauge
	processedTxsTotal     *prometheus.CounterVec
}

var _ Indicators = (*PromIndicators)(nil)

func NewPromIndicators(reg prometheus.Registerer, subsystem string) *PromIndicators {
	return &PromIndicators{
		broadcastLatencyMs: promauto.With(reg).NewSummary(
			prometheus.SummaryOpts{
				Namespace:  consts.TransactionProcess,
				Subsystem:  subsystem,
				Name:       "broadcast_latency_ms",
				Help:       "transaction confirmation latency summary in milliseconds",
				Objectives: map[float64]float64{0.5: 0.05, 0.9: 0.01, 0.95: 0.01, 0.99: 0.001},
			},
		),
		confirmationLatencyMs: promauto.With(reg).NewSummary(
			prometheus.SummaryOpts{
				Namespace:  consts.TransactionProcess,
				Subsystem:  subsystem,
				Name:       "confirmation_latency_ms",
				Help:       "total transaction confirmation latency summary in milliseconds",
				Objectives: map[float64]float64{0.5: 0.05, 0.9: 0.01, 0.95: 0.01, 0.99: 0.001},
			},
		),
		gasUsedOsmo: promauto.With(reg).NewSummary(
			prometheus.SummaryOpts{
				Namespace:  consts.TransactionProcess,
				Subsystem:  subsystem,
				Name:       "gas_used_total",
				Help:       "total gas used to submit each transaction onchain",
				Objectives: map[float64]float64{0.5: 0.05, 0.9: 0.01, 0.95: 0.01, 0.99: 0.001},
			},
		),
		speedUps: promauto.With(reg).NewHistogram(
			prometheus.HistogramOpts{
				Namespace: consts.TransactionProcess,
				Subsystem: subsystem,
				Name:      "speedups_total",
				Help:      "number of times a transaction's gas price was increased",
				Buckets:   prometheus.LinearBuckets(0, 1, 10),
			},
		),
		processingTxCount: promauto.With(reg).NewGauge(
			prometheus.GaugeOpts{
				Namespace: consts.TransactionProcess,
				Subsystem: subsystem,
				Name:      "processing_tx_count",
				Help:      "number of transactions currently being processed",
			},
		),
		processedTxsTotal: promauto.With(reg).NewCounterVec(
			prometheus.CounterOpts{
				Namespace: consts.TransactionProcess,
				Subsystem: subsystem,
				Name:      "processed_txs_total",
				Help:      "number of transactions processed by state (success, error)",
			},
			[]string{"state"},
		),
	}
}

func (p *PromIndicators) ObserveBroadcastLatencyMs(latencyMs int64) {
	p.broadcastLatencyMs.Observe(float64(latencyMs))
}

func (p *PromIndicators) ObserveConfirmationLatencyMs(latencyMs int64) {
	p.confirmationLatencyMs.Observe(float64(latencyMs))
}

func (p *PromIndicators) ObserveGasUsedOsmo(gasUsedUosmo uint64) {
	osmo := gasUsedUosmo / 1e6
	p.gasUsedOsmo.Observe(float64(osmo))
}

func (p *PromIndicators) ObserveSpeedups(speedUps int) {
	p.speedUps.Observe(float64(speedUps))
}

func (p *PromIndicators) IncrementProcessingTxCount() {
	p.processingTxCount.Inc()
}

func (p *PromIndicators) DecrementProcessingTxCount() {
	p.processingTxCount.Dec()
}

func (p *PromIndicators) IncrementProcessedTxsTotal(state string) {
	p.processedTxsTotal.WithLabelValues(state).Inc()
}
