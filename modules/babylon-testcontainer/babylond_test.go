package babylond

import (
	"context"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/stretchr/testify/suite"
	"testing"
	"time"
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
	//suite.Require().NoError(suite.Container.Container.Terminate(context.Background()))
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
	queryClient := banktypes.NewQueryClient(clientCtx)
	req := banktypes.QueryBalanceRequest{
		Address: "bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		Denom:   "ubbn",
	}

	res, err := queryClient.Balance(context.Background(), &req)
	suite.NoError(err)
	suite.Equal("1000000ubbn", res.Balance.String())
}

func (suite *BabylondTestSuite) TestSendCoins() {
	clientCtx := suite.Container.ClientCtx()

	config := sdk.GetConfig()
	config.SetBech32PrefixForAccount("bbn", "bbnpub")

	privateKey := secp256k1.GenPrivKey()
	address, err := bech32.ConvertAndEncode("bbn", privateKey.PubKey().Address())
	suite.NoError(err)

	receiver, err := sdk.AccAddressFromBech32(address)
	suite.NoError(err)

	coin, err := sdk.ParseCoinNormalized("1ubbn")
	suite.NoError(err)

	genesisAddr, err := sdk.AccAddressFromBech32("bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv")
	suite.NoError(err)

	msg := banktypes.NewMsgSend(genesisAddr, receiver, sdk.NewCoins(coin))

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	err = txBuilder.SetMsgs(msg)
	suite.NoError(err)

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	suite.NoError(err)

	res, err := clientCtx.BroadcastTx(txBytes)
	suite.NoError(err)
	suite.Regexp("^[0-9A-F]{64}$", res.TxHash)

	time.Sleep(10 * time.Second)

	queryClient := banktypes.NewQueryClient(clientCtx)
	balReq := banktypes.QueryBalanceRequest{
		Address: address,
		Denom:   "ubbn",
	}
	balRes, err := queryClient.Balance(context.Background(), &balReq)
	suite.NoError(err)
	suite.Equal("1ubbn", balRes.Balance.String())
}
