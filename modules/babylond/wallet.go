package babylond

import (
	"context"
	"encoding/hex"

	"cosmossdk.io/math"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func (c *BabylonContainer) GenerateAddress(uid string) sdk.AccAddress {
	record, err := c.ClientCtx.Keyring.Key(uid)
	if record == nil {
		privKey := secp256k1.GenPrivKey()
		err := c.ClientCtx.Keyring.ImportPrivKeyHex(uid, hex.EncodeToString(privKey.Bytes()), "secp256k1")
		if err != nil {
			panic(err)
		}

		record, err = c.ClientCtx.Keyring.Key(uid)
		if record == nil {
			panic(err)
		}
	} else {
		if err != nil {
			panic(err)
		}
	}

	address, err := record.GetAddress()
	if err != nil {
		panic(err)
	}
	return address
}

func (c *BabylonContainer) FundAddress(address string, coin sdk.Coin) *coretypes.ResultBroadcastTxCommit {
	ctx := context.Background()

	from, err := sdk.AccAddressFromBech32("bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv")
	if err != nil {
		panic(err)
	}
	to, err := sdk.AccAddressFromBech32(address)
	if err != nil {
		panic(err)
	}

	clientCtx := c.ClientCtx.WithFromName("genesis").WithFromAddress(from)
	txf, err := c.TxFactory.Prepare(clientCtx)
	if err != nil {
		panic(err)
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(1000))))
	txBuilder.SetGasLimit(200000)
	msg := banktypes.NewMsgSend(from, to, sdk.NewCoins(coin))
	err = txBuilder.SetMsgs(msg)
	if err != nil {
		panic(err)
	}

	err = tx.Sign(ctx, txf, clientCtx.FromName, txBuilder, true)
	if err != nil {
		panic(err)
	}

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	if err != nil {
		panic(err)
	}

	node, err := clientCtx.GetNode()
	if err != nil {
		panic(err)
	}
	result, err := node.BroadcastTxCommit(context.Background(), txBytes)
	if err != nil {
		panic(err)
	}

	return result
}

func (c *BabylonContainer) FundAddressUbbn(address string, amount int64) *coretypes.ResultBroadcastTxCommit {
	return c.FundAddress(address, sdk.NewCoin("ubbn", math.NewInt(amount)))
}

func (c *BabylonContainer) ImportPrivKey(uid string, hex string) {
	err := c.ClientCtx.Keyring.ImportPrivKeyHex(uid, hex, "secp256k1")
	if err != nil {
		panic(err)
	}
}
