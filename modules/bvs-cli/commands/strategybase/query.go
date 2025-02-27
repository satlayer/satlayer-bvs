package strategybase

import "fmt"

func UnderlyingToken() {
	s := NewService()
	resp, err := s.StrategyBase.UnderlyingToken()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}
