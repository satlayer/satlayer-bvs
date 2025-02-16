package babylond

import (
	"context"
	"github.com/stretchr/testify/assert"
	"os"
	"testing"
)

func TestWasmStoreCode(t *testing.T) {
	container, err := Run(context.Background())
	assert.NoError(t, err)

	data, err := os.ReadFile("cw_bvs_driver.wasm")
	assert.NoError(t, err)

	res, err := container.StoreWasmCode(data, "genesis")
	assert.NoError(t, err)
	assert.Equal(t, uint32(0), res.Code)
	assert.Equal(t, "", res.RawLog)
	//assert.Equal(t, uint32(1), res.CodeID)
}
