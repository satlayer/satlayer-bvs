package chain

import (
	"context"
	"fmt"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/conf"
)

func newService(keyName string) io.ChainIO {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	return newChainIO
}

func IncreaseTokenAllowance(userKeyName, tokenAddress string, spenderAddress string, amount uint64) {
	ctx := context.Background()
	chainIO := newService(userKeyName)
	txnResp, err := api.IncreaseTokenAllowance(ctx, chainIO, amount, tokenAddress, spenderAddress, sdktypes.NewInt64DecCoin("ubbn", 1))
	if err != nil {
		panic(err)
	}
	fmt.Printf("IncreaseTokenAllowance success, txn hash: %s\n", txnResp.Hash.String())
}
