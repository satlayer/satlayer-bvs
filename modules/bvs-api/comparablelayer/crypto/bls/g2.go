package bls

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	bn254utils "github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/crypto/bn254"
)

// G2Point represents a point on the G2 curve
type G2Point struct {
	*bn254.G2Affine
}

// Add adds another G2 point to this one
func (p *G2Point) Add(p2 *G2Point) *G2Point {
	p.G2Affine.Add(p.G2Affine, p2.G2Affine)
	return p
}

// Sub subtracts another G2 point from this one
func (p *G2Point) Sub(p2 *G2Point) *G2Point {
	p.G2Affine.Sub(p.G2Affine, p2.G2Affine)
	return p
}

func (p *G2Point) Serialize() []byte {
	return bn254utils.SerializeG2(p.G2Affine)
}

func (p *G2Point) Deserialize(data []byte) *G2Point {
	return &G2Point{bn254utils.DeserializeG2(data)}
}
