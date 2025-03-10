package signer

import (
	"encoding/base64"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

func VerifySignature(pubKey cryptotypes.PubKey, msgHash []byte, signatureBase64 string) (bool, error) {
	signatureBytes, err := base64.StdEncoding.DecodeString(signatureBase64)
	if err != nil {
		return false, err
	}
	return pubKey.VerifySignature(msgHash, signatureBytes), nil
}
