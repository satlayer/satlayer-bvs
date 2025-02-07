package rpccalls

import (
	"github.com/prometheus/client_golang/prometheus"

	rpccalls "github.com/satlayer/satlayer-api/metrics/indicators/rpc_calls"
)

type Collector struct {
	promIndicators *rpccalls.PromIndicators
}

// NewCollector returns a rpccalls Collector that collects metrics for json-rpc calls
func NewCollector(bvsName string, reg prometheus.Registerer) *Collector {
	return &Collector{
		promIndicators: rpccalls.NewPromIndicators(bvsName, reg),
	}
}

// ObserveRPCRequestDurationSeconds observes the duration of a json-rpc request
func (c *Collector) ObserveRPCRequestDurationSeconds(duration float64, method, clientVersion string) {
	c.promIndicators.ObserveRPCRequestDurationSeconds(duration, method, clientVersion)
}

// AddRPCRequestTotal adds a json-rpc request to the total number of requests
func (c *Collector) AddRPCRequestTotal(method, clientVersion string) {
	c.promIndicators.AddRPCRequestTotal(method, clientVersion)
}
