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

func (s *BvsTestSuite) Test_DeployRegistry() {
	contract := s.Deployer.DeployPauser(nil)
	s.NotEmpty(contract.Address)
}

func (s *BvsTestSuite) Test_DeployDirectory() {
	pauser := s.Deployer.DeployPauser(nil)
	contract := s.Deployer.DeployDirectory(pauser.Address)
	s.NotEmpty(contract.Address)
}
