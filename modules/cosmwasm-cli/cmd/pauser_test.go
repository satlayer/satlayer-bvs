package cmd

import (
	"context"
	"testing"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/pauser"
	"github.com/stretchr/testify/suite"
)

type PauserTestSuite struct {
	suite.Suite
	container *babylond.BabylonContainer
	pauser    *bvs.Contract[pauser.InstantiateMsg]
}

func (s *PauserTestSuite) SetupSuite() {
	s.container = babylond.Run(context.Background())

	deployer := &bvs.Deployer{BabylonContainer: s.container}
	s.pauser = deployer.DeployPauser(nil)
}

func (s *PauserTestSuite) TearDownSuite() {
	s.Require().NoError(s.container.Terminate(context.Background()))
}

func TestPauser(t *testing.T) {
	suite.Run(t, new(PauserTestSuite))
}

func (s *PauserTestSuite) Test_IsPaused() {
	rootCmd := RootCmd()
	randomContractAddr := s.container.GenerateAddress("randomContractAddr")

	args := []string{
		"pauser", "query", "is_paused", randomContractAddr.String(), "SomeMethod",
		"--contract=" + s.pauser.Address,
		"--node=" + s.container.RpcUri,
		"--chain-id=" + s.container.ChainId,
	}
	rootCmd.SetArgs(args)
	err := rootCmd.Execute()
	s.NoError(err)
}
