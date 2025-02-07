package ecdsa

import (
	"os"
	"path/filepath"
	"testing"

	cosmossecp256k1 "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/stretchr/testify/assert"
)

func TestECDSAPrivateKey(t *testing.T) {
	var tests = map[string]struct {
		keyPath  string
		password string
		wantErr  bool
	}{
		"valid ecdsa key save": {
			keyPath:  "./.keypair/ecdsa_key.json",
			password: "test",
			wantErr:  false,
		},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			t.Cleanup(func() {
				dir := filepath.Dir(tt.keyPath)
				_ = os.RemoveAll(dir)
			})
			randomKey := cosmossecp256k1.GenPrivKey()

			err := CosmosWriteKey(tt.keyPath, randomKey, tt.password)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			readKeyPair, err := ReadCosmosKey(tt.keyPath, tt.password)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, randomKey, readKeyPair)
			}
		})
	}
}
