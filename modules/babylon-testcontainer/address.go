package babylond

import (
	"fmt"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/types/bech32"
)

type Address struct {
	PrivateKeyHex string
	PublicKeyHex  string
	Bech32Addr    string
}

func GenAddress() Address {
	privateKey := secp256k1.GenPrivKey()
	publicKey := privateKey.PubKey()
	address := publicKey.Address()
	bech32Addr, err := bech32.ConvertAndEncode("bbn", address)
	if err != nil {
		panic(err)
	}

	return Address{
		PrivateKeyHex: fmt.Sprintf("%X", privateKey.Bytes()),
		PublicKeyHex:  fmt.Sprintf("%X", publicKey.Bytes()),
		Bech32Addr:    bech32Addr,
	}
}
