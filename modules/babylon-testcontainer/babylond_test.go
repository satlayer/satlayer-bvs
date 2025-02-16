package babylond

import (
	"context"
	"cosmossdk.io/math"
	"encoding/hex"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
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
	url := suite.Container.RpcUri
	suite.Regexp(`http://localhost:\d+`, url)
}

func (suite *BabylondTestSuite) TestClientContext() {
	clientCtx := suite.Container.ClientCtx
	status, err := clientCtx.Client.Status(context.Background())
	suite.NoError(err)
	suite.GreaterOrEqual(status.SyncInfo.LatestBlockHeight, int64(1))
}

func (suite *BabylondTestSuite) TestGenesisBalance() {
	clientCtx := suite.Container.ClientCtx
	queryClient := banktypes.NewQueryClient(clientCtx)
	req := banktypes.QueryBalanceRequest{
		Address: "bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		Denom:   "ubbn",
	}

	res, err := queryClient.Balance(context.Background(), &req)
	suite.NoError(err)
	suite.Equal("100000000000000000ubbn", res.Balance.String())
}

func (suite *BabylondTestSuite) TestSendCoinsManually() {
	clientCtx := suite.Container.ClientCtx

	var receiver sdk.AccAddress
	sender, err := sdk.AccAddressFromBech32("bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv")
	suite.NoError(err)
	privKeyBytes, err := hex.DecodeString("230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A")
	suite.NoError(err)
	privKey := secp256k1.PrivKey{Key: privKeyBytes}

	{
		privateKey := secp256k1.GenPrivKey()
		address, err := bech32.ConvertAndEncode("bbn", privateKey.PubKey().Address())
		suite.NoError(err)

		receiver, err = sdk.AccAddressFromBech32(address)
		suite.NoError(err)
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(1000))))
	txBuilder.SetGasLimit(200000)

	msg := banktypes.NewMsgSend(sender, receiver, sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(10))))
	err = txBuilder.SetMsgs(msg)
	suite.NoError(err)

	account, err := clientCtx.AccountRetriever.GetAccount(clientCtx, sender)
	suite.NoError(err)

	signMode := signing.SignMode(clientCtx.TxConfig.SignModeHandler().DefaultMode())

	emptySig := signing.SignatureV2{
		PubKey: privKey.PubKey(),
		Data: &signing.SingleSignatureData{
			SignMode:  signMode,
			Signature: nil,
		},
		Sequence: account.GetSequence(),
	}
	err = txBuilder.SetSignatures(emptySig)
	suite.NoError(err)

	signerData := authsigning.SignerData{
		ChainID:       clientCtx.ChainID,
		AccountNumber: account.GetAccountNumber(),
		Sequence:      account.GetSequence(),
	}

	signV2, err := tx.SignWithPrivKey(
		context.Background(),
		signMode,
		signerData,
		txBuilder,
		&privKey,
		clientCtx.TxConfig,
		account.GetSequence(),
	)
	suite.NoError(err)

	err = txBuilder.SetSignatures(signV2)
	suite.NoError(err)

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	suite.NoError(err)

	res, err := clientCtx.BroadcastTx(txBytes)
	suite.NoError(err)
	suite.Equal("", res.RawLog)
	suite.Equal(uint32(0), res.Code)
	suite.Regexp("^[0-9A-F]{64}$", res.TxHash)

	time.Sleep(2 * time.Second)

	queryClient := banktypes.NewQueryClient(clientCtx)
	balRes, err := queryClient.Balance(context.Background(), &banktypes.QueryBalanceRequest{
		Address: receiver.String(),
		Denom:   "ubbn",
	})
	suite.NoError(err)
	suite.Equal("10ubbn", balRes.Balance.String())
}

func (suite *BabylondTestSuite) TestSendCoinsTxFactory() {
	from, err := sdk.AccAddressFromBech32("bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv")
	suite.NoError(err)
	to, err := sdk.AccAddressFromBech32("bbn1whhg5wce9g9nn6frehnagh526f6thc7y380jhl")
	suite.NoError(err)

	clientCtx := suite.Container.ClientCtx.WithFromName("genesis").WithFromAddress(from)
	txf, err := suite.Container.TxFactory.Prepare(clientCtx)
	suite.NoError(err)

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(1000))))
	txBuilder.SetGasLimit(200000)
	msg := banktypes.NewMsgSend(from, to, sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(100))))
	err = txBuilder.SetMsgs(msg)
	suite.NoError(err)

	ctx := context.Background()
	err = tx.Sign(ctx, txf, clientCtx.FromName, txBuilder, true)
	suite.NoError(err)

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	suite.NoError(err)

	res, err := clientCtx.BroadcastTxSync(txBytes)
	suite.NoError(err)
	suite.Equal(uint32(0), res.Code)

	time.Sleep(2 * time.Second)

	queryClient := banktypes.NewQueryClient(clientCtx)
	balRes, err := queryClient.Balance(context.Background(), &banktypes.QueryBalanceRequest{
		Address: to.String(),
		Denom:   "ubbn",
	})
	suite.NoError(err)
	suite.Equal("100ubbn", balRes.Balance.String())
}
