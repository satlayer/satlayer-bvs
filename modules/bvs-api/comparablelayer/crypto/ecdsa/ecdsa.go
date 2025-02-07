package ecdsa

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	cosmossecp256k1 "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/crypto/encrypt"
)

type encryptedJSON struct {
	PubKey string             `json:"pubKey"`
	Crypto encrypt.CryptoJSON `json:"crypto"`
}

func CosmosWriteKeyFromHex(path, privateKeyHex, password string) error {
	privKeyBytes, err := hex.DecodeString(privateKeyHex)
	if err != nil {
		return fmt.Errorf("failed to decode hex key: %w", err)
	}

	privKey := cosmossecp256k1.PrivKey{Key: privKeyBytes}
	return CosmosWriteKey(path, &privKey, password)
}

// CosmosWriteKey writes Cosmos private key to file
func CosmosWriteKey(path string, privateKey *cosmossecp256k1.PrivKey, password string) error {
	cryptoStruct, err := encrypt.EncryptData(
		privateKey.Bytes(),
		[]byte(password),
		encrypt.StandardScryptN,
		encrypt.StandardScryptP,
	)
	if err != nil {
		return err
	}

	encryptedStruct := encryptedJSON{
		privateKey.PubKey().String(),
		cryptoStruct,
	}

	data, err := json.Marshal(encryptedStruct)
	if err != nil {
		return fmt.Errorf("failed to marshal encryped struct to JSON: %w", err)
	}

	return writeBytesToFile(path, data)
}

func writeBytesToFile(path string, data []byte) error {
	dir := filepath.Dir(path)

	// create the directory if it doesn't exist
	if err := os.MkdirAll(dir, 0755); err != nil {
		fmt.Println("Error creating directories:", err)
		return err
	}

	file, err := os.Create(filepath.Clean(path))
	if err != nil {
		fmt.Println("file create error")
		return err
	}
	defer file.Close()

	_, err = file.Write(data)
	return err
}

// ReadCosmosKey Cosmos compatible key read from file
func ReadCosmosKey(keyStoreFile string, password string) (*cosmossecp256k1.PrivKey, error) {
	keyStoreContents, err := os.ReadFile(keyStoreFile)
	if err != nil {
		return nil, err
	}

	var encryptedStruct encryptedJSON
	if err := json.Unmarshal(keyStoreContents, &encryptedStruct); err != nil {
		return nil, fmt.Errorf("failed to unmarshal key JSON: %w", err)
	}

	if encryptedStruct.PubKey == "" {
		return nil, fmt.Errorf("invalid key")
	}

	privKeyBytes, err := encrypt.DecryptData(encryptedStruct.Crypto, password)
	if err != nil {
		return nil, fmt.Errorf("failed to decrypt key: %w", err)
	}

	privKey := cosmossecp256k1.PrivKey{Key: privKeyBytes}
	return &privKey, nil
}

// KeyAndCosmosAddressFromHexKey for Cosmos
func KeyAndCosmosAddressFromHexKey(hexkey string) (*cosmossecp256k1.PrivKey, sdk.AccAddress, error) {
	hexkey = strings.TrimPrefix(hexkey, "0x")
	privKeyBytes, err := hex.DecodeString(hexkey)
	if err != nil {
		return nil, sdk.AccAddress{}, fmt.Errorf("failed to decode hex key: %w", err)
	}

	privKey := cosmossecp256k1.PrivKey{Key: privKeyBytes}
	address := GetCosmosAddressFromPrivateKey(&privKey)

	return &privKey, address, nil
}

// GetCosmosAddressFromPrivateKey Get Cosmos address from private key
func GetCosmosAddressFromPrivateKey(privKey *cosmossecp256k1.PrivKey) sdk.AccAddress {
	pubKey := privKey.PubKey()
	address := sdk.AccAddress(pubKey.Address())
	return address
}
