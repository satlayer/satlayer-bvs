package babylond

import (
	"context"
	"testing"

	"github.com/stretchr/testify/suite"
)

type BabylondTestSuite struct {
	suite.Suite
	Container *BabylonContainer
}

func (s *BabylondTestSuite) SetupSuite() {
	s.Container = Run(context.Background())
}

func (s *BabylondTestSuite) TearDownSuite() {
	s.Require().NoError(s.Container.Container.Terminate(context.Background()))
}

func TestBabylond(t *testing.T) {
	suite.Run(t, new(BabylondTestSuite))
}

func (s *BabylondTestSuite) TestRpcUri() {
	url := s.Container.RpcUri
	s.Regexp(`http://localhost:\d+`, url)
}

func (s *BabylondTestSuite) TestClientContext() {
	clientCtx := s.Container.ClientCtx
	status, err := clientCtx.Client.Status(context.Background())
	s.NoError(err)
	s.GreaterOrEqual(status.SyncInfo.LatestBlockHeight, int64(1))
}
