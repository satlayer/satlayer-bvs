package cosmwasmapi

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/bvs-cw/pauser"
	"github.com/stretchr/testify/suite"
)

type ClientTestSuite struct {
	suite.Suite
	container *babylond.BabylonContainer
	pauser    *bvs.Contract[pauser.InstantiateMsg]
}

func (s *ClientTestSuite) SetupSuite() {
	s.container = babylond.Run(context.Background())
	deployer := &bvs.Deployer{BabylonContainer: s.container}
	s.pauser = deployer.DeployPauser(nil)
}

func (s *ClientTestSuite) TearDownSuite() {
	s.Require().NoError(s.container.Terminate(context.Background()))
}

func TestClient(t *testing.T) {
	suite.Run(t, new(ClientTestSuite))
}

func (s *ClientTestSuite) Test_Query() {
	contract := s.container.GenerateAddress("contract")
	queryMsg := pauser.QueryMsg{
		IsPaused: &pauser.IsPaused{
			C: contract.String(),
			M: "Deposit",
		},
	}

	response, err := Query[pauser.IsPausedResponse](
		s.container.ClientCtx,
		context.Background(),
		s.pauser.Address,
		queryMsg,
	)
	s.NoError(err)
	s.Equal(response, pauser.IsPausedResponse(0))
}
