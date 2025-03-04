package strategybase

import "fmt"

func Shares(staker string) {
	s := NewService()
	resp, err := s.StrategyBase.Shares(staker)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func SharesToUnderlying(amount string) {
	s := NewService()
	resp, err := s.StrategyBase.SharesToUnderlying(amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingToShares(amount string) {
	s := NewService()
	resp, err := s.StrategyBase.UnderlyingToShares(amount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func Underlying(staker string) {
	s := NewService()
	resp, err := s.StrategyBase.Underlying(staker)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingToken() {
	s := NewService()
	resp, err := s.StrategyBase.UnderlyingToken()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}
