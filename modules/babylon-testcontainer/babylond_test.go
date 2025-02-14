package babylond

import (
	"context"
	"github.com/stretchr/testify/assert"
	"testing"
	"time"
)

func TestRpcEndpoint(t *testing.T) {
	ctx := context.Background()
	container, err := Run(ctx)
	assert.NoError(t, err)
	assert.Regexp(t, `http://localhost:\d+`, container.GetRpcUrl())
}

func TestChainIOQueryNodeStatus(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	container, err := Run(ctx)
	assert.NoError(t, err)

	chainIO, err := container.GetChainIO()
	assert.NoError(t, err)

	status, err := chainIO.QueryNodeStatus(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, status)
}
