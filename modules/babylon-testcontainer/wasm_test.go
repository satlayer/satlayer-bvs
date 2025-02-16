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

	ctx := context.Background()
	res, err := container.StoreWasmCode(ctx, data, "genesis")
	assert.NoError(t, err)
	assert.Equal(t, uint32(0), res.TxResult.Code)

	codeId, err := GetCodeId(res)
	assert.Equal(t, "1", codeId)
}
