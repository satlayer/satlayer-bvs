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

func IsStrategyWhitelisted(strategyAddress string) {
	s := NewService()
	resp, err := s.Strategy.IsStrategyWhitelisted(strategyAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}
