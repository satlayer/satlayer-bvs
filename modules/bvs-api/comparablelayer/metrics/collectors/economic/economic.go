package economic

import (
	"github.com/prometheus/client_golang/prometheus"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/logging"
	"github.com/satlayer/satlayer-bvs/bvs-api/metrics/collectors/economic"
)

// Collector exports the economic metrics listed at
type Collector struct {
	satLayerEconomic *economic.Collector
}

var _ prometheus.Collector = (*Collector)(nil)

func NewCollector(bvsName, operatorAddr string, strategies []string, delegation api.Delegation, logger logging.Logger) *Collector {
	return &Collector{
		satLayerEconomic: economic.NewCollector(bvsName, operatorAddr, strategies, delegation, logger.GetElkLogger()),
	}
}

// Describe describes to Prometheus the metrics this collector will collect
func (ec *Collector) Describe(ch chan<- *prometheus.Desc) {
	ec.satLayerEconomic.Describe(ch)
}

// Collect  collect custom metric values
func (ec *Collector) Collect(ch chan<- prometheus.Metric) {
	ec.satLayerEconomic.Collect(ch)
}
