package bls

import (
	bn254utils "github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/crypto/bn254"
)

// Signature represents a BLS signature
type Signature struct {
	*G1Point `json:"g1_point"`
}

// NewZeroSignature creates a new zero signature
func NewZeroSignature() *Signature {
	return &Signature{NewZeroG1Point()}
}

// Add adds another signature to this one
func (s *Signature) Add(otherS *Signature) *Signature {
	s.G1Point.Add(otherS.G1Point)
	return s
}

// Verify verifies a message against a public key
func (s *Signature) Verify(pubkey *G2Point, message [32]byte) (bool, error) {
	return bn254utils.VerifySig(s.G1Affine, pubkey.G2Affine, message)
}
