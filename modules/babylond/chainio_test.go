package babylond

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestChainIOQueryNodeStatus(t *testing.T) {
	container := Run(context.Background())
	chainIO := container.NewChainIO("../.babylon")

	status, err := chainIO.QueryNodeStatus(context.Background())
	assert.NoError(t, err)
	assert.NotNil(t, status)
}
