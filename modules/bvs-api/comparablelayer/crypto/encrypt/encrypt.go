package encrypt

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"

	"github.com/xdg-go/pbkdf2"
	"golang.org/x/crypto/scrypt"
	"golang.org/x/crypto/sha3"
)

const (
	keyHeaderKDF = "scrypt"
	// StandardScryptN is the N parameter of Scrypt encryption algorithm, using 256MB
	// memory and taking approximately 1s CPU time on a modern processor.
	StandardScryptN = 1 << 18

	// StandardScryptP is the P parameter of Scrypt encryption algorithm, using 256MB
	// memory and taking approximately 1s CPU time on a modern processor.
	StandardScryptP = 1
	scryptR         = 8
	scryptDKLen     = 32
)

var ErrDecrypt = errors.New("could not decrypt key with given password")

type encryptedBLSKeyJSON struct {
	PubKey string     `json:"pubKey"`
	Crypto CryptoJSON `json:"crypto"`
}

type CryptoJSON struct {
	Cipher     string                 `json:"cipher"`
	CipherText string                 `json:"ciphertext"`
	IV         string                 `json:"iv"`
	KDF        string                 `json:"kdf"`
	KDFParams  map[string]interface{} `json:"kdfparams"`
	MAC        string                 `json:"mac"`
}

func DecryptData(cryptoJSON CryptoJSON, auth string) ([]byte, error) {
	if cryptoJSON.Cipher != "aes-128-ctr" {
		return nil, fmt.Errorf("cipher not supported: %v", cryptoJSON.Cipher)
	}
	mac, err := hex.DecodeString(cryptoJSON.MAC)
	if err != nil {
		return nil, err
	}

	iv, err := hex.DecodeString(cryptoJSON.IV)
	if err != nil {
		return nil, err
	}

	cipherText, err := hex.DecodeString(cryptoJSON.CipherText)
	if err != nil {
		return nil, err
	}

	derivedKey, err := getKDFKey(cryptoJSON, auth)
	if err != nil {
		return nil, err
	}

	calculatedMAC := keccak256(derivedKey[16:32], cipherText)
	if !bytes.Equal(calculatedMAC, mac) {
		return nil, ErrDecrypt
	}

	plainText, err := aesCTRXOR(derivedKey[:16], cipherText, iv)
	if err != nil {
		return nil, err
	}
	return plainText, err
}

// EncryptData encrypts the data given as 'data' with the password 'auth'.
func EncryptData(data, auth []byte, scryptN, scryptP int) (CryptoJSON, error) {
	salt := make([]byte, 32)
	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		panic("reading from crypto/rand failed: " + err.Error())
	}
	derivedKey, err := scrypt.Key(auth, salt, scryptN, scryptR, scryptP, scryptDKLen)
	if err != nil {
		return CryptoJSON{}, err
	}
	encryptKey := derivedKey[:16]

	iv := make([]byte, aes.BlockSize) // 16
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		panic("reading from crypto/rand failed: " + err.Error())
	}
	cipherText, err := aesCTRXOR(encryptKey, data, iv)
	if err != nil {
		return CryptoJSON{}, err
	}
	mac := keccak256(derivedKey[16:32], cipherText)

	scryptParamsJSON := make(map[string]interface{}, 5)
	scryptParamsJSON["n"] = scryptN
	scryptParamsJSON["r"] = scryptR
	scryptParamsJSON["p"] = scryptP
	scryptParamsJSON["dklen"] = scryptDKLen
	scryptParamsJSON["salt"] = hex.EncodeToString(salt)

	cryptoStruct := CryptoJSON{
		Cipher:     "aes-128-ctr",
		CipherText: hex.EncodeToString(cipherText),
		IV:         hex.EncodeToString(iv),
		KDF:        keyHeaderKDF,
		KDFParams:  scryptParamsJSON,
		MAC:        hex.EncodeToString(mac),
	}
	return cryptoStruct, nil
}

func keccak256(data ...[]byte) []byte {
	hash := sha3.NewLegacyKeccak256()
	for _, b := range data {
		hash.Write(b)
	}
	return hash.Sum(nil)
}

func getKDFKey(cryptoJSON CryptoJSON, auth string) ([]byte, error) {
	authArray := []byte(auth)
	salt, err := hex.DecodeString(cryptoJSON.KDFParams["salt"].(string))
	if err != nil {
		return nil, err
	}
	dkLen := ensureInt(cryptoJSON.KDFParams["dklen"])

	if cryptoJSON.KDF == keyHeaderKDF {
		n := ensureInt(cryptoJSON.KDFParams["n"])
		r := ensureInt(cryptoJSON.KDFParams["r"])
		p := ensureInt(cryptoJSON.KDFParams["p"])
		return scrypt.Key(authArray, salt, n, r, p, dkLen)
	} else if cryptoJSON.KDF == "pbkdf2" {
		iterCount := ensureInt(cryptoJSON.KDFParams["c"])
		prf := cryptoJSON.KDFParams["prf"].(string)
		if prf != "hmac-sha256" {
			return nil, fmt.Errorf("unsupported PBKDF2 PRF: %s", prf)
		}
		key := pbkdf2.Key(authArray, salt, iterCount, dkLen, sha256.New)
		return key, nil
	}

	return nil, fmt.Errorf("unsupported KDF: %s", cryptoJSON.KDF)
}

func aesCTRXOR(key, inText, iv []byte) ([]byte, error) {
	// AES-128 is selected due to size of encryptKey.
	aesBlock, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	stream := cipher.NewCTR(aesBlock, iv)
	outText := make([]byte, len(inText))
	stream.XORKeyStream(outText, inText)
	return outText, err
}

func ensureInt(x interface{}) int {
	res, ok := x.(int)
	if !ok {
		res = int(x.(float64))
	}
	return res
}
