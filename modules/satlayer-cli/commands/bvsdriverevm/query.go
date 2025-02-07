package bvsdriverevm

import (
	"context"
	"fmt"
	"github.com/ethereum/go-ethereum/common"
)

func GetOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.BVSDriver.Owner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("Owner address: %s\n", resp)
}

func GetPendingOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.BVSDriver.PendingOwner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("PendingOwner address: %s\n", resp)
}

func IsBVSRegistered(bvsContract string) {
	ctx := context.Background()
	s := NewService()
	bvsContractAddr := common.HexToAddress(bvsContract)
	resp, err := s.BVSDriver.IsBVSContractRegistered(ctx, bvsContractAddr)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("Result: %t\n", resp)
}
