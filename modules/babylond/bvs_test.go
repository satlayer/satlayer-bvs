package babylond

import (
	"context"
	"testing"

	"github.com/stretchr/testify/suite"
)

type BvsTestSuite struct {
	suite.Suite
	Container *BabylonContainer
}

func (s *BvsTestSuite) SetupSuite() {
	s.Container = Run(context.Background())
}

func (s *BvsTestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestBvs(t *testing.T) {
	suite.Run(t, new(BvsTestSuite))
}

func (s *BvsTestSuite) TestDeployStateBank() {
	contract := s.Container.DeployStateBank()
	s.NotEmpty(contract.Address)
}

func (s *BvsTestSuite) TestDeployDriver() {
	contract := s.Container.DeployDriver()
	s.NotEmpty(contract.Address)
}

// TODO: deploy all CW contracts
