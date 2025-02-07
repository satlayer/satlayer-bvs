package economic

import (
	"strconv"

	"github.com/prometheus/client_golang/prometheus"

	"github.com/satlayer/satlayer-api/chainio/api"
	"github.com/satlayer/satlayer-api/logger"
	"github.com/satlayer/satlayer-api/metrics/consts"
)

type Collector struct {
	bvsName      string
	operatorAddr string
	strategies   []string
	delegation   api.Delegation
	logger       logger.Logger
	// Contributions delegated by an operator in different strategies.
	delegatedShares *prometheus.Desc
	// When an operator is slashed, its value is 1, otherwise it is 0. This is a metric used to track operator status.
	//slashingStatus *prometheus.Desc
	// Not implemented yet
	//registeredStake *prometheus.Desc
}

var _ prometheus.Collector = (*Collector)(nil)

func NewCollector(bvsName, operatorAddr string, strategies []string, delegation api.Delegation, logger logger.Logger) *Collector {
	return &Collector{
		bvsName:      bvsName,
		operatorAddr: operatorAddr,
		strategies:   strategies,
		delegation:   delegation,
		logger:       logger,
		delegatedShares: prometheus.NewDesc(
			consts.SatLayerPromNamespace+"_delegated_shares",
			"",
			[]string{"operator_address", "strategy"},
			prometheus.Labels{},
		),
	}
}

// Describe describes to Prometheus the metrics this collector will collect
func (c *Collector) Describe(ch chan<- *prometheus.Desc) {
	ch <- c.delegatedShares
	//ch <- c.slashingStatus
	//ch <- c.registeredStake
}

// Collect  collect custom metric values
func (c *Collector) Collect(ch chan<- prometheus.Metric) {
	// delegated_shares
	resp, err := c.delegation.GetOperatorShares(c.operatorAddr, c.strategies)
	if err != nil {
		c.logger.Error("Failed to get operator stakers", logger.WithField("err", err))
	} else {
		for i, share := range resp.Shares {
			val, _ := strconv.ParseFloat(share, 64)
			ch <- prometheus.MustNewConstMetric(c.delegatedShares, prometheus.GaugeValue, val, c.operatorAddr, c.strategies[i])
		}
	}
}
