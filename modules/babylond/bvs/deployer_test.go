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
	contract := s.Deployer.DeployRegistry(nil)
	s.NotEmpty(contract.Address)
}

func (s *BvsTestSuite) Test_DeployDirectory() {
	registry := s.Deployer.DeployRegistry(nil)
	tAddr := s.Container.GenerateAddress("throw-away")
	contract := s.Deployer.DeployDirectory(registry.Address, tAddr.String())
	s.NotEmpty(contract.Address)
}
