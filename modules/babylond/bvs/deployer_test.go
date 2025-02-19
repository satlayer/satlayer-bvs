package bvs

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"

	"github.com/stretchr/testify/suite"
)

type BvsTestSuite struct {
	suite.Suite
	Container *babylond.BabylonContainer
	Deployer  *Deployer
}

func (s *BvsTestSuite) SetupSuite() {
	s.Container = babylond.Run(context.Background())
	s.Deployer = &Deployer{s.Container}
}

func (s *BvsTestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestBvs(t *testing.T) {
	suite.Run(t, new(BvsTestSuite))
}

func (s *BvsTestSuite) TestDeployStateBank() {
	contract := s.Deployer.DeployStateBank()
	s.NotEmpty(contract.Address)
}

func (s *BvsTestSuite) TestDeployDriver() {
	contract := s.Deployer.DeployDriver()
	s.NotEmpty(contract.Address)
}

// TODO: deploy all CW contracts
