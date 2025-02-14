package babylond

import (
	"context"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/stretchr/testify/suite"
	"testing"
)

type BabylondTestSuite struct {
	suite.Suite
	Container *BabylonContainer
}

func (suite *BabylondTestSuite) SetupSuite() {
	container, err := Run(context.Background())
	suite.Require().NoError(err)
	suite.Container = container
}

func (suite *BabylondTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.Container.Container.Terminate(context.Background()))
}

func TestBabylond(t *testing.T) {
	suite.Run(t, new(BabylondTestSuite))
}

func (suite *BabylondTestSuite) TestRpcUri() {
	url := suite.Container.GetRpcUri()
	suite.Regexp(`http://localhost:\d+`, url)
}

func (suite *BabylondTestSuite) TestClientContext() {
	clientCtx := suite.Container.ClientCtx()
	status, err := clientCtx.Client.Status(context.Background())
	suite.NoError(err)
	suite.GreaterOrEqual(status.SyncInfo.LatestBlockHeight, int64(1))
}

func (suite *BabylondTestSuite) TestGenesisBalance() {
	clientCtx := suite.Container.ClientCtx()
	queryClient := types.NewQueryClient(clientCtx)
	req := types.QueryBalanceRequest{
		Address: "bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		Denom:   "ubbn",
	}

	res, err := queryClient.Balance(context.Background(), &req)
	suite.NoError(err)
	suite.Equal("1000000ubbn", res.Balance.String())
}
