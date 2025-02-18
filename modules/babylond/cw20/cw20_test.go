package cw20

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/stretchr/testify/suite"
)

type Cw20TestSuite struct {
	suite.Suite
	Container *babylond.BabylonContainer
}

func (s *Cw20TestSuite) SetupSuite() {
	s.Container = babylond.Run(context.Background())
}

func (s *Cw20TestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestCw20(t *testing.T) {
	suite.Run(t, new(Cw20TestSuite))
}

func (s *Cw20TestSuite) TestDeployCw20Token() {
	minter := s.Container.GenerateAddress("cw20:minter")
	initMsg := InstantiateMsg{
		Decimals: 6,
		InitialBalances: []Cw20Coin{
			{
				Address: minter.String(),
				Amount:  "1000000000",
			},
		},
		Mint: &MinterResponse{
			Minter: minter.String(),
		},
		Name:   "Test Token",
		Symbol: "TEST",
	}

	contract := DeployCw20(s.Container, initMsg)
	s.NotEmpty(contract.Address)
}
