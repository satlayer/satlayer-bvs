package bls

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"

	bn254utils "github.com/satlayer/satlayer-api/comparablelayer/crypto/bn254"
)

// G2Point represents a point on the G2 curve
type G2Point struct {
	*bn254.G2Affine
}

// NewG2Point creates a new G2Point
func NewG2Point(X, Y [2]*big.Int) *G2Point {
	return &G2Point{
		&bn254.G2Affine{
			// compatible with eigen, the order remains the same
			X: struct{ A0, A1 fp.Element }{
				A0: bn254utils.NewFpElement(X[1]),
				A1: bn254utils.NewFpElement(X[0]),
			},
			Y: struct{ A0, A1 fp.Element }{
				A0: bn254utils.NewFpElement(Y[1]),
				A1: bn254utils.NewFpElement(Y[0]),
			},
		},
	}
}

// NewZeroG2Point creates a new zero G2Point
func NewZeroG2Point() *G2Point {
	return NewG2Point([2]*big.Int{big.NewInt(0), big.NewInt(0)}, [2]*big.Int{big.NewInt(0), big.NewInt(0)})
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
