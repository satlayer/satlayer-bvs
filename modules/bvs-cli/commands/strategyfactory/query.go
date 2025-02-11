package strategyfactory

import "fmt"

func GetStrategy(token string) {
	s := NewService()
	resp, err := s.StrategyFactory.GetStrategy(token)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Strategy)
}

func IsTokenBlacklisted(token string) {
	s := NewService()
	resp, err := s.StrategyFactory.IsTokenBlacklisted(token)
	if err != nil {
		fmt.Printf("Failed to check if token is blacklisted: %v\n", err)
		return
	}
	fmt.Printf("%t\n", resp.IsBlacklisted)
}
