package bls

// Signature represents a BLS signature
type Signature struct {
	*G1Point `json:"g1_point"`
}
