package e2e

import (
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
)

// TODO: Temporary Wallet Utility (for testing), due to how the keychain is currently managed, too much headache

func GetPubKeyFromKeychainByAddress(chainio io.ChainIO, address string) cryptotypes.PubKey {
	addr, err := sdk.AccAddressFromBech32(address)
	if err != nil {
		panic(err)
	}
	key, err := chainio.GetClientCtx().Keyring.KeyByAddress(addr)
	if err != nil {
		panic(err)
	}
	pubKey, err := key.GetPubKey()
	if err != nil {
		panic(err)
	}
	return pubKey
}

func GetPubKeyFromKeychainByUid(chainio io.ChainIO, uid string) cryptotypes.PubKey {
	key, err := chainio.GetClientCtx().Keyring.Key(uid)
	if err != nil {
		panic(err)
	}
	pubKey, err := key.GetPubKey()
	if err != nil {
		panic(err)
	}
	return pubKey
}
