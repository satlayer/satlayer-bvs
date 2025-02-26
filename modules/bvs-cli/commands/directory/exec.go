package directory

import (
	"context"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
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

func RegBVS(userKeyName, BVSAddr string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)

	txn, err := directory.RegisterBvs(ctx, BVSAddr)
	if err != nil {
		fmt.Printf("Register BVS error! %v\n", err)
		return
	}
	fmt.Printf("Directory register BVS success. txn: %s\n", txn.Hash)
	for _, event := range txn.TxResult.Events {
		if event.Type == "wasm" {
			for _, attr := range event.Attributes {
				if attr.Key == "bvs_hash" {
					fmt.Printf("Register BVS success. BVS Hash: %s\n", attr.Value)
					return
				}
			}
		}
	}

}

func RegOperator(operatorKeyName string) {
	ctx := context.Background()
	directory, newChainIO := newService(operatorKeyName)
	pubKey := newChainIO.GetCurrentAccountPubKey()
	address := sdk.AccAddress(pubKey.Address()).String()
	txn, err := directory.RegisterOperator(ctx, address, address, pubKey)
	if err != nil {
		fmt.Printf("Register operator error! %v\n", err)
		return
	}
	fmt.Printf("Register operator success. txn: %s\n", txn.Hash)
}

func DeregOperator(operatorKeyName string) {
	ctx := context.Background()
	directory, newChainIO := newService(operatorKeyName)
	pubKey := newChainIO.GetCurrentAccountPubKey()
	address := sdk.AccAddress(pubKey.Address()).String()
	txn, err := directory.DeregisterOperator(ctx, address)
	if err != nil {
		fmt.Printf("Deregister operator error! %v\n", err)
		return
	}
	fmt.Printf("Deregister operator success. txn: %s\n", txn.Hash)
}

func UpdateMetadata(userKeyName, metadata string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.UpdateMetadataURI(ctx, metadata)
	if err != nil {
		fmt.Printf("Update metadata error! %v\n", err)
		return
	}
	fmt.Printf("Update metadata success. txn: %s\n", txn.Hash)
}

func CancelSalt(userKeyName, salt string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.CancelSalt(ctx, salt)
	if err != nil {
		fmt.Printf("Cancel salt error! %v\n", err)
		return
	}
	fmt.Printf("Cancel salt success. txn: %s\n", txn.Hash)
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

func Pause(userKeyName string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.Pause(ctx)
	if err != nil {
		fmt.Printf("Pause error! %v\n", err)
		return
	}
	fmt.Printf("Pause success. txn: %s\n", txn.Hash)
}

func Unpause(userKeyName string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.Unpause(ctx)
	if err != nil {
		fmt.Printf("Unpause error! %v\n", err)
		return
	}
	fmt.Printf("Unpause success. txn: %s\n", txn.Hash)
}

func SetPauser(userKeyName, newPauser string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.SetPauser(ctx, newPauser)
	if err != nil {
		fmt.Printf("Set pauser error! %v\n", err)
		return
	}
	fmt.Printf("Set pauser success. txn: %s\n", txn.Hash)
}

func SetUnpauser(userKeyName, newUnpauser string) {
	ctx := context.Background()
	directory, _ := newService(userKeyName)
	txn, err := directory.SetUnpauser(ctx, newUnpauser)
	if err != nil {
		fmt.Printf("Set unpauser error! %v\n", err)
		return
	}
	fmt.Printf("Set unpauser success. txn: %s\n", txn.Hash)
}
