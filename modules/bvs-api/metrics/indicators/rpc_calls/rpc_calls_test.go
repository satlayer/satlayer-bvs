package rpccalls

import (
	"testing"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/testutil"
	"github.com/stretchr/testify/assert"
)

func TestRPCCallsIndicators(t *testing.T) {
	reg := prometheus.NewRegistry()
	rpcCalls := NewPromIndicators("localbvs", reg)

	rpcCalls.AddRPCRequestTotal("test", "test1.0")
	assert.Equal(
		t,
		1.0,
		testutil.ToFloat64(rpcCalls.rpcRequestTotal.WithLabelValues("test", "test1.0")),
	)
}
