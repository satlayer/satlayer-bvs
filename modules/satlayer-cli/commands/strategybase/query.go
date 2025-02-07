package strategybase

import "fmt"

func GetShares(staker string, strategy string) {
	s := NewService()
	resp, err := s.StrategyBase.GetShares(staker, strategy)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func SharesUnderlyingView(shareAmount uint64) {
	s := NewService()
	resp, err := s.StrategyBase.SharesToUnderlyingView(shareAmount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingShareView(underlyingAmount uint64) {
	s := NewService()
	resp, err := s.StrategyBase.UnderlyingToShareView(underlyingAmount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingView(user string) {
	s := NewService()
	resp, err := s.StrategyBase.UnderlyingView(user)
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
