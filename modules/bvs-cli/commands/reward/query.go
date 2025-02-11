package reward

import (
	"fmt"
)

func GetDistributionRootLength() {
	s := NewService()
	resp, err := s.Reward.GetDistributionRootsLength()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetCurrentDistributionRoot() {
	s := NewService()
	resp, err := s.Reward.GetCurrentDistributionRoot()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetDistributionRootAtIndex(index string) {
	s := NewService()
	resp, err := s.Reward.GetDistributionRootAtIndex(index)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetCurrentClaimableDistributionRoot() {
	s := NewService()
	resp, err := s.Reward.GetCurrentClaimableDistributionRoot()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetRootIndexFromHash(rootHash string) {
	s := NewService()
	resp, err := s.Reward.GetRootIndexFromHash(rootHash)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func IsUpdater(userAddress string) {
	s := NewService()
	resp, err := s.Reward.IsRewardsUpdater(userAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%t\n", resp)
}
