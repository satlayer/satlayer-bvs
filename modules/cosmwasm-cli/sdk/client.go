package sdk

import (
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/spf13/viper"

	cosmwasmapi "github.com/satlayer/satlayer-bvs/cosmwasm-api"
)

func NewClientCtx() client.Context {
	return cosmwasmapi.NewClientCtx(
		viper.GetString("node"),
		viper.GetString("chain-id"),
	)
}

// WithKeyring sets up the keyring for the client context.
// This will essentially attach a signer to the client context.
func WithKeyring(ctx client.Context) client.Context {
	keyringBackend := viper.GetString("keyring-backend")

	kr, err := keyring.New("satlayer", keyringBackend, ctx.KeyringDir, ctx.Input, ctx.Codec, ctx.KeyringOptions...)
	if err != nil {
		panic(err)
	}

	from := viper.GetString("from")

	key, err := kr.Key(from)
	if err != nil {
		panic(err)
	}

	addr, err := key.GetAddress()
	if err != nil {
		panic(err)
	}

	return ctx.WithKeyring(kr).WithFromName(from).WithFromAddress(addr).WithFrom(from)
}

func DefaultBroadcastOptions() cosmwasmapi.BroadcastOptions {
	return cosmwasmapi.DefaultBroadcastOptions()
}
