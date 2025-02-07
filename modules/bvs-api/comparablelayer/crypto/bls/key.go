package bls

import "github.com/consensys/gnark-crypto/ecc/bn254/fr"

type PrivateKey struct {
	PrivKey *fr.Element
}

func (p *PrivateKey) Type() string {
	return "BLSPrivKey"
}

type PubKey struct {
	*G1Point
}

func (p *PubKey) Type() string {
	return "BLSPubKey"
}
