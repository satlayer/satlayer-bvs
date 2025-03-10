package babylond

import (
	"time"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

func (c *BabylonContainer) NewChainIO(homeDir string) io.ChainIO {
	chainIo, err := io.NewChainIO(ChainId, c.RpcUri, homeDir, "bbn", types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}
	return chainIo
}
