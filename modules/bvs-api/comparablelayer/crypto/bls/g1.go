package bls

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"

	bn254utils "github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/crypto/bn254"
)

// G1Point represents a point on the G1 curve
type G1Point struct {
	*bn254.G1Affine
}

// NewG1Point creates a new G1Point
func NewG1Point(x, y *big.Int) *G1Point {
	return &G1Point{
		&bn254.G1Affine{
			X: bn254utils.NewFpElement(x),
			Y: bn254utils.NewFpElement(y),
		},
	}
}

// NewZeroG1Point creates a new zero G1Point
func NewZeroG1Point() *G1Point {
	return NewG1Point(big.NewInt(0), big.NewInt(0))
}

// Add adds another G1 point to this one
func (p *G1Point) Add(p2 *G1Point) *G1Point {
	p.G1Affine.Add(p.G1Affine, p2.G1Affine)
	return p
}

// Sub subtracts another G1 point from this one
func (p *G1Point) Sub(p2 *G1Point) *G1Point {
	p.G1Affine.Sub(p.G1Affine, p2.G1Affine)
	return p
}

func (p *G1Point) Serialize() []byte {
	return bn254utils.SerializeG1(p.G1Affine)
}

func (p *G1Point) Deserialize(data []byte) *G1Point {
	return &G1Point{bn254utils.DeserializeG1(data)}
}

// VerifyEquivalence verifies G1Point is equivalent to the G2Point
func (p *G1Point) VerifyEquivalence(p2 *G2Point) (bool, error) {
	return bn254utils.CheckG1AndG2DiscreteLogEquality(p.G1Affine, p2.G2Affine)
}
