package bls

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBlsKeypair(t *testing.T) {
	var tests = map[string]struct {
		keyPath  string
		password string
		wantErr  bool
	}{
		"valid bls key save": {
			keyPath:  "./.keypair/bls_key.json",
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
			randomKey, err := GenRandomBlsKeys()
			assert.NoError(t, err)

			err = randomKey.SaveToFile(tt.keyPath, tt.password)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			readKeyPair, err := ReadPrivateKeyFromFile(tt.keyPath, tt.password)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, randomKey, readKeyPair)
			}
		})
	}
}
