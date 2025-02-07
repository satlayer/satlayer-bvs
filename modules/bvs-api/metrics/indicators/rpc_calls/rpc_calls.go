package rpccalls

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"

	"github.com/satlayer/satlayer-api/metrics/consts"
)

type PromIndicators struct {
	rpcRequestDurationSeconds *prometheus.HistogramVec
	rpcRequestTotal           *prometheus.CounterVec
}

func NewPromIndicators(bvsName string, reg prometheus.Registerer) *PromIndicators {
	return &PromIndicators{
		rpcRequestDurationSeconds: promauto.With(reg).NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace:   consts.SatLayerPromNamespace,
				Name:        "rpc_request_duration_seconds",
				Help:        "Duration of json-rpc <method> in seconds",
				ConstLabels: prometheus.Labels{"bvs_name": bvsName},
			},
			[]string{"method", "client_version"},
		),
		rpcRequestTotal: promauto.With(reg).NewCounterVec(
			prometheus.CounterOpts{
				Namespace:   consts.SatLayerPromNamespace,
				Name:        "rpc_request_total",
				Help:        "Total number of json-rpc <method> requests",
				ConstLabels: prometheus.Labels{"bvs_name": bvsName},
			},
			[]string{"method", "client_version"},
		),
	}
}

// ObserveRPCRequestDurationSeconds observes the duration of a json-rpc request
func (p *PromIndicators) ObserveRPCRequestDurationSeconds(duration float64, method, clientVersion string) {
	p.rpcRequestDurationSeconds.With(prometheus.Labels{
		"method":         method,
		"client_version": clientVersion,
	}).Observe(duration)
}

// AddRPCRequestTotal adds a json-rpc request to the total number of requests
func (p *PromIndicators) AddRPCRequestTotal(method, clientVersion string) {
	p.rpcRequestTotal.With(prometheus.Labels{
		"method":         method,
		"client_version": clientVersion,
	}).Inc()
}
