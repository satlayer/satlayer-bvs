package util

import (
	"encoding/base64"
	"fmt"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/hello-world-bvs/aggregator/core"
)

// PubKeyToAddress converts a base64 encoded public key to a secp256k1 public key and its corresponding Cosmos address.
//
// pubKey is the base64 encoded public key to be converted.
// Returns the converted secp256k1 public key, its corresponding Cosmos address, and an error if the conversion fails.
func PubKeyToAddress(pubKey string) (*secp256k1.PubKey, string, error) {
	pubKeyRawBytes, err := base64.StdEncoding.DecodeString(pubKey)
	if err != nil {
		core.L.Error(fmt.Sprintf("failed to decode public key: %v\n", err))
		return nil, "", err
	}

	newPubKey := secp256k1.PubKey{Key: pubKeyRawBytes}
	address := types.AccAddress(newPubKey.Address()).String()

	return &newPubKey, address, nil
}
