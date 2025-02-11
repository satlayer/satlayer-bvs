package directoryevm

import (
	"context"
	"fmt"
)

func GetOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Directory.Owner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("Owner address: %s\n", resp)
}

func GetPendingOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Directory.PendingOwner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("PendingOwner address: %s\n", resp)
}

func GetBVSInfo(bvsHash string) {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Directory.GetBVSInfo(ctx, bvsHash)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("bvsContract: %s\n", resp.BVSContract)
}
