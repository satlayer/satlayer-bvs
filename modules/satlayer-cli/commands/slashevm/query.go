package slashevm

import (
	"context"
	"fmt"
)

func IsValidator(validator string) {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.Validators(ctx, validator)
	if err != nil {
		panic(fmt.Sprintf("Failed to query validator: %v", err))
	}

	fmt.Printf("IsValidator response: %t\n", resp)
}

func IsPaused() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.Paused(ctx)
	if err != nil {
		panic(fmt.Sprintf("Failed to query validator: %v", err))
	}

	fmt.Printf("IsPaused response: %t\n", resp)
}

func GetMinimalSlashSignature() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.MinimalSlashSignature(ctx)
	if err != nil {
		panic(fmt.Sprintf("Error while fetching minimal slash signature: %v", err))
	}

	if resp == nil {
		panic("Received nil response from GetMinimalSlashSignature")
	}

	fmt.Printf("Minimal slash signature value: %d\n", resp)
}

func GetOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.Owner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("Owner address: %s\n", resp)
}

func GetPendingOwner() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.PendingOwner(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("PendingOwner address: %s\n", resp)
}

func GetSlasher() {
	ctx := context.Background()
	s := NewService()
	resp, err := s.Slash.Slasher(ctx)
	if err != nil {
		panic(fmt.Sprintf("%v", err))
	}

	fmt.Printf("Slasher address: %s\n", resp)
}
