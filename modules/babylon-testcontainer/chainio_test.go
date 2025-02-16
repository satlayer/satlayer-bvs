package babylond

import (
	"context"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestChainIOQueryNodeStatus(t *testing.T) {
	container, err := Run(context.Background())
	assert.NoError(t, err)

	chainIO, err := container.NewChainIO("../.babylon")
	assert.NoError(t, err)

	status, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, status)
}
