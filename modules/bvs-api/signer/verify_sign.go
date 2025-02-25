package signer

import (
	"encoding/base64"
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// VerifySignature Function to verify signature
func VerifySignature(pubKey cryptotypes.PubKey, msgHash []byte, signatureBase64 string) (bool, error) {
	signatureBytes, err := base64.StdEncoding.DecodeString(signatureBase64)
	if err != nil {
		return false, fmt.Errorf("failed to decode signature: %w", err)
	}
	return pubKey.VerifySignature(msgHash, signatureBytes), nil
}
