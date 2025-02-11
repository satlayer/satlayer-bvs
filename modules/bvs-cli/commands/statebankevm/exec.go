package statebankevm

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

func RegBVS(userAddr, password, bvsContract string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	bvsContractAddr := common.HexToAddress(bvsContract)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.StateBank.SetRegisteredBVSContract(ctx, wallet, bvsContractAddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("RegBVS success. txn: %s\n", txResp.TxHash)
}

func AcceptOwner(userAddr, password string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.StateBank.AcceptOwnership(ctx, wallet)
	if err != nil {
		panic(err)
	}
	fmt.Printf("AcceptOwner success. txn: %s\n", txResp.TxHash)
}

func RenounceOwner(userAddr, password string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.StateBank.RenounceOwnership(ctx, wallet)
	if err != nil {
		panic(err)
	}
	fmt.Printf("RenounceOwner success. txn: %s\n", txResp.TxHash)
}

func TransferOwner(userAddr, password, newOwner string) {
	ctx := context.Background()
	s := NewService()
	fromAddr := common.HexToAddress(userAddr)
	newOwnerAddr := common.HexToAddress(newOwner)
	wallet := types.ETHWallet{
		FromAddr: fromAddr,
		PWD:      password,
	}

	txResp, err := s.StateBank.TransferOwnership(ctx, wallet, newOwnerAddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("TransferOwner success. txn: %s\n", txResp.TxHash)
}
