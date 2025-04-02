package cmd

import (
	"context"
	"math/rand"
	"strconv"
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

	deployer := bvs.NewDeployer(s.container)
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

	rootCmd.SetArgs([]string{
		"pauser", "query", "is_paused", randomContractAddr.String(), "SomeMethod",
		"--contract=" + s.pauser.Address,
		"--node=" + s.container.RpcUri,
		"--chain-id=" + s.container.ChainId,
	})
	s.NoError(rootCmd.Execute())
}

func (s *PauserTestSuite) Test_Pause() {
	rootCmd := RootCmd()
	from := strconv.Itoa(rand.Intn(1000000000))

	owner := s.container.GenerateAddress("owner")
	s.container.FundAddressUbbn(owner.String(), 1000000000000)
	privKey, algoStr := s.container.ExportPrivKeyHex("owner")

	kr := s.container.NewKeyring("satlayer", "test", "")
	s.Require().NoError(kr.ImportPrivKeyHex(from, privKey, algoStr))

	randomContractAddr := s.container.GenerateAddress("randomContractAddr")

	rootCmd.SetArgs([]string{
		"pauser", "execute", "pause", randomContractAddr.String(), "SomeMethod",
		"--keyring-backend=test",
		"--from=" + from,
		"--contract=" + s.pauser.Address,
		"--node=" + s.container.RpcUri,
		"--chain-id=" + s.container.ChainId,
	})
	s.NoError(rootCmd.Execute())
}
