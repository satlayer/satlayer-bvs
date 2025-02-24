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

func IsThirdTransferForbidden(strategyAddress string) {
	s := NewService()
	resp, err := s.Strategy.IsThirdPartyTransfersForbidden(strategyAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetNonce(stakerAddress string) {
	s := NewService()
	resp, err := s.Strategy.GetNonce(stakerAddress)
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

func GetOwner() {
	s := NewService()
	resp, err := s.Strategy.GetOwner()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func IsStrategyWhiteListed(strategyAddress string) {
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

func GetDepositTypeHash() {
	s := NewService()
	resp, err := s.Strategy.GetDepositTypeHash()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetDomainTypeHash() {
	s := NewService()
	resp, err := s.Strategy.GetDomainTypeHash()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetDomainName() {
	s := NewService()
	resp, err := s.Strategy.GetDomainName()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}

func GetDelegationManager() {
	s := NewService()
	resp, err := s.Strategy.GetDelegationManager()
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\n", resp.Data)
}
