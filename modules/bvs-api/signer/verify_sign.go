package signer

import (
	"encoding/base64"
	"errors"

	sdksecp256k1 "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	dcrdsecp256k1 "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"
)

// VerifySignature Function to verify signature
func VerifySignature(pubKey cryptotypes.PubKey, msgHash []byte, signatureBase64 string) (bool, error) {
	// Decode base64 encoded signature into bytes
	signatureBytes, err := base64.StdEncoding.DecodeString(signatureBase64)
	if err != nil {
		return false, err
	}
	// Check that the signature length is correct (should be 64 bytes since the recovery bit was removed)
	if len(signatureBytes) != 64 {
		return false, errors.New("invalid signature length")
	}
	// Convert the Cosmos SDK's public key to the public key format of the dcrdsecp256k1 library
	cosmosSecp256k1PubKey, ok := pubKey.(*sdksecp256k1.PubKey)
	if !ok {
		return false, errors.New("invalid public key type")
	}
	dcrdPubKey, err := dcrdsecp256k1.ParsePubKey(cosmosSecp256k1PubKey.Bytes())
	if err != nil {
		return false, err
	}
	// Create ecdsa.Signature from signature bytes
	r := new(dcrdsecp256k1.ModNScalar)
	s := new(dcrdsecp256k1.ModNScalar)
	r.SetByteSlice(signatureBytes[:32])
	s.SetByteSlice(signatureBytes[32:])
	signature := ecdsa.NewSignature(r, s)
	// Verify signature
	return signature.Verify(msgHash, dcrdPubKey), nil
}
