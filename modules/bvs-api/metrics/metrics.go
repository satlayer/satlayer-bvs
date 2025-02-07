package metrics

import (
	"context"
	"errors"
	"net/http"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"

	"github.com/satlayer/satlayer-api/logger"
	"github.com/satlayer/satlayer-api/utils"
)

type Metrics interface {
	Start(ctx context.Context, reg prometheus.Gatherer) <-chan error
}

type SatLayerMetrics struct {
	ipPortAddress string
	logger        logger.Logger
}

var _ Metrics = (*SatLayerMetrics)(nil)

// NewSatLayerMetrics Initialize monitoring
func NewSatLayerMetrics(ipPortAddress string, logger logger.Logger) Metrics {
	return &SatLayerMetrics{
		ipPortAddress: ipPortAddress,
		logger:        logger,
	}
}

// Start "/metrics" to report metric data
func (s SatLayerMetrics) Start(ctx context.Context, reg prometheus.Gatherer) <-chan error {
	s.logger.Info("Starting metrics server at port", logger.WithField("ipPortAddress", s.ipPortAddress))
	errChan := make(chan error, 1)
	mux := http.NewServeMux()
	httpServer := http.Server{
		Addr:    s.ipPortAddress,
		Handler: mux,
	}
	mux.Handle("/metrics", promhttp.HandlerFor(
		reg,
		promhttp.HandlerOpts{},
	))

	// shutdown server on context done
	go func() {
		<-ctx.Done()
		s.logger.Info("shutdown signal received")
		defer func() {
			close(errChan)
		}()

		if err := httpServer.Shutdown(context.Background()); err != nil {
			errChan <- err
		}
		s.logger.Info("shutdown completed")
	}()

	go func() {
		err := httpServer.ListenAndServe()
		if errors.Is(err, http.ErrServerClosed) {
			s.logger.Info("server closed")
		} else {
			errChan <- utils.WrapError("prometheus server failed", err)
		}
	}()
	return errChan
}
