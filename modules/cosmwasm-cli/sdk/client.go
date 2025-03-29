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

func WithKeyring(ctx client.Context) client.Context {
	keyringBackend := viper.GetString("keyring-backend")

	kr, err := keyring.New("satlayer", keyringBackend, ctx.KeyringDir, ctx.Input, ctx.Codec, ctx.KeyringOptions...)
	if err != nil {
		panic(err)
	}

	from := viper.GetString("from")
	ctx.WithKeyring(kr).WithFromName(from)
	return ctx
}

func DefaultBroadcastOptions() cosmwasmapi.BroadcastOptions {
	return cosmwasmapi.DefaultBroadcastOptions()
}
