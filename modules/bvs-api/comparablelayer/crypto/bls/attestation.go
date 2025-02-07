package bls

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"

	bn254utils "github.com/satlayer/satlayer-api/comparablelayer/crypto/bn254"
	"github.com/satlayer/satlayer-api/comparablelayer/crypto/encrypt"
)

type encryptedBLSKeyJSON struct {
	PubKey string             `json:"pubKey"`
	Crypto encrypt.CryptoJSON `json:"crypto"`
}

// KeyPair BLS key pair
type KeyPair struct {
	PrivKey *PrivateKey
	PubKey  *PubKey
}

func NewKeyPair(sk *PrivateKey) *KeyPair {
	pk := bn254utils.MulByGeneratorG1(sk.PrivKey)
	return &KeyPair{sk, &PubKey{&G1Point{pk}}}
}

func NewKeyPairFromString(sk string) (*KeyPair, error) {
	ele, err := new(fr.Element).SetString(sk)
	if err != nil {
		return nil, err
	}
	return NewKeyPair(&PrivateKey{PrivKey: ele}), nil
}

func GenRandomBlsKeys() (*KeyPair, error) {
	maxVal := new(big.Int)
	maxVal.SetString(fr.Modulus().String(), 10)

	n, err := rand.Int(rand.Reader, maxVal)
	if err != nil {
		return nil, err
	}

	sk := &PrivateKey{PrivKey: new(fr.Element).SetBigInt(n)}
	return NewKeyPair(sk), nil
}

func ReadPrivateKeyFromFile(path string, password string) (*KeyPair, error) {
	keyStoreContents, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	encryptedBLSStruct := &encryptedBLSKeyJSON{}
	if err = json.Unmarshal(keyStoreContents, encryptedBLSStruct); err != nil {
		return nil, err
	}

	if encryptedBLSStruct.PubKey == "" {
		return nil, fmt.Errorf("invalid bls key file. pubkey field not found")
	}

	skBytes, err := encrypt.DecryptData(encryptedBLSStruct.Crypto, password)
	if err != nil {
		return nil, err
	}

	privKey := new(fr.Element).SetBytes(skBytes)
	keyPair := NewKeyPair(&PrivateKey{PrivKey: privKey})
	return keyPair, nil
}

// SaveToFile saves the private key in an encrypted keystore file
func (k *KeyPair) SaveToFile(path string, password string) error {
	data, err := k.EncryptedString(password)
	if err != nil {
		return err
	}

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		fmt.Println("Error creating directories:", err)
		return err
	}
	err = os.WriteFile(path, data, 0644)
	if err != nil {
		return err
	}
	return nil
}

func (k *KeyPair) EncryptedString(password string) ([]byte, error) {
	sk32Bytes := k.PrivKey.PrivKey.Bytes()
	skBytes := make([]byte, 32)
	for i := 0; i < 32; i++ {
		skBytes[i] = sk32Bytes[i]
	}

	cryptoStruct, err := encrypt.EncryptData(
		skBytes,
		[]byte(password),
		encrypt.StandardScryptN,
		encrypt.StandardScryptP,
	)
	if err != nil {
		return nil, err
	}

	encryptedBLSStruct := encryptedBLSKeyJSON{
		k.PubKey.String(),
		cryptoStruct,
	}
	data, err := json.Marshal(encryptedBLSStruct)
	if err != nil {
		return nil, err
	}
	return data, nil
}

func (k *KeyPair) SignMessage(message [32]byte) *Signature {
	H := bn254utils.MapToCurve(message)
	sig := new(bn254.G1Affine).ScalarMultiplication(H, k.PrivKey.PrivKey.BigInt(new(big.Int)))
	return &Signature{&G1Point{sig}}
}

func (k *KeyPair) SignHashedToCurveMessage(g1HashedMsg *bn254.G1Affine) *Signature {
	sig := new(bn254.G1Affine).ScalarMultiplication(g1HashedMsg, k.PrivKey.PrivKey.BigInt(new(big.Int)))
	return &Signature{&G1Point{sig}}
}

func (k *KeyPair) GetPubKeyG2() *G2Point {
	return &G2Point{bn254utils.MulByGeneratorG2(k.PrivKey.PrivKey)}
}

func (k *KeyPair) GetPubKeyG1() *G1Point {
	return k.PubKey.G1Point
}
