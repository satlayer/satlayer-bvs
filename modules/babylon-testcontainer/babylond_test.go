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

func (s *BabylondTestSuite) SetupSuite() {
	container, err := Run(context.Background())
	s.Require().NoError(err)
	s.Container = container
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

func (s *BabylondTestSuite) TestGenesisBalance() {
	clientCtx := s.Container.ClientCtx
	queryClient := banktypes.NewQueryClient(clientCtx)
	req := banktypes.QueryBalanceRequest{
		Address: "bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		Denom:   "ubbn",
	}

	res, err := queryClient.Balance(context.Background(), &req)
	s.NoError(err)
	// Greater than 10^16, because the genesis balance is 10^17, concurrently running tests may have spent some
	s.Greater(res.Balance.Amount.Int64(), int64(10000000000000000))
}

func (s *BabylondTestSuite) TestSendCoinsManually() {
	clientCtx := s.Container.ClientCtx

	var receiver sdk.AccAddress
	sender, err := sdk.AccAddressFromBech32("bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv")
	s.NoError(err)
	privKeyBytes, err := hex.DecodeString("230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A")
	s.NoError(err)
	privKey := secp256k1.PrivKey{Key: privKeyBytes}

	{
		privateKey := secp256k1.GenPrivKey()
		address, err := bech32.ConvertAndEncode("bbn", privateKey.PubKey().Address())
		s.NoError(err)

		receiver, err = sdk.AccAddressFromBech32(address)
		s.NoError(err)
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(1000))))
	txBuilder.SetGasLimit(200000)

	msg := banktypes.NewMsgSend(sender, receiver, sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(10))))
	err = txBuilder.SetMsgs(msg)
	s.NoError(err)

	account, err := clientCtx.AccountRetriever.GetAccount(clientCtx, sender)
	s.NoError(err)

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
	s.NoError(err)

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
	s.NoError(err)

	err = txBuilder.SetSignatures(signV2)
	s.NoError(err)

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	s.NoError(err)

	res, err := clientCtx.BroadcastTx(txBytes)
	s.NoError(err)
	s.Equal("", res.RawLog)
	s.Equal(uint32(0), res.Code)
	s.Regexp("^[0-9A-F]{64}$", res.TxHash)

	time.Sleep(2 * time.Second)

	queryClient := banktypes.NewQueryClient(clientCtx)
	balRes, err := queryClient.Balance(context.Background(), &banktypes.QueryBalanceRequest{
		Address: receiver.String(),
		Denom:   "ubbn",
	})
	s.NoError(err)
	s.Equal("10ubbn", balRes.Balance.String())
}

func (s *BabylondTestSuite) TestFundAccount() {
	res, err := s.Container.FundAccountUbbn("bbn1whhg5wce9g9nn6frehnagh526f6thc7y380jhl", 10000)
	s.NoError(err)
	s.Equal(uint32(0), res.TxResult.Code)

	queryClient := banktypes.NewQueryClient(s.Container.ClientCtx)
	balRes, err := queryClient.Balance(context.Background(), &banktypes.QueryBalanceRequest{
		Address: "bbn1whhg5wce9g9nn6frehnagh526f6thc7y380jhl",
		Denom:   "ubbn",
	})
	s.NoError(err)
	s.Equal("10000ubbn", balRes.Balance.String())
}
