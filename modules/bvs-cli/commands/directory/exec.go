package directory

import (
	"context"
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (*api.Directory, io.ChainIO) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	directory := api.NewDirectory(newChainIO, conf.C.Contract.Directory).WithGasLimit(400000)
	return directory, newChainIO
}

func TransferOwner(userKeyName, newOwner string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.TransferOwnership(ctx, newOwner)
	if err != nil {
		fmt.Printf("Transfer ownership error! %v\n", err)
		return
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", txn.Hash)
}
