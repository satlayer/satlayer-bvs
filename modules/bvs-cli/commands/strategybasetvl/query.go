package strategybasetvl

import "fmt"

func GetShares(staker string, strategy string) {
	s := NewService()
	resp, err := s.StrategyBaseTVL.GetShares(staker, strategy)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func SharesUnderlyingView(shareAmount uint64) {
	s := NewService()
	resp, err := s.StrategyBaseTVL.SharesToUnderlyingView(shareAmount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingShareView(underlyingAmount uint64) {
	s := NewService()
	resp, err := s.StrategyBaseTVL.UnderlyingToShareView(underlyingAmount)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingView(user string) {
	s := NewService()
	resp, err := s.StrategyBaseTVL.UnderlyingView(user)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func UnderlyingToken() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.UnderlyingToken()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetLimits() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.GetTvlLimits()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Get limits success. limits: %s\n", resp.MaxPerDeposit)
	fmt.Printf("Get limits success. limits: %s\n", resp.MaxTotalDeposits)
}

func GetStrategyManager() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.GetStrategyManager()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetTotalShares() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.GetTotalDeposits()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.TotalShares)
}

func Explanation() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.Explanation()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Explanation)
}

func GetStrategyState() {
	s := NewService()
	resp, err := s.StrategyBaseTVL.GetStrategyState()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Get strategy state success. StrategyManger: %s, UnderlyingToken: %s, TotalShares: %s\n", resp.StrategyManager, resp.UnderlyingToken, resp.TotalShares)
}
