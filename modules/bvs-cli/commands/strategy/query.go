package strategy

import "fmt"

func GetDeposits(stakerAddress string) {
	s := NewService()
	resp, err := s.Strategy.GetDeposits(stakerAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetStakerStrategyListLength(stakerAddress string) {
	s := NewService()
	resp, err := s.Strategy.StakerStrategyListLength(stakerAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetStakerStrategyList(stakerAddress string) {
	s := NewService()
	resp, err := s.Strategy.GetStakerStrategyList(stakerAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetStakerStrategyShares(staker string, strategy string) {
	s := NewService()
	resp, err := s.Strategy.GetStakerStrategyShares(staker, strategy)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func Owner() {
	s := NewService()
	resp, err := s.Strategy.Owner()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func IsStrategyWhitelisted(strategyAddress string) {
	s := NewService()
	resp, err := s.Strategy.IsStrategyWhitelisted(strategyAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetStrategyWhitelist() {
	s := NewService()
	resp, err := s.Strategy.GetStrategyWhitelister()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetStrategyManagerState() {
	s := NewService()
	resp, err := s.Strategy.GetStrategyManagerState()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func DelegationManager() {
	s := NewService()
	resp, err := s.Strategy.DelegationManager()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}
