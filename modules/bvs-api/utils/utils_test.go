package utils

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEcdsaPrivateKeyToCosmosAddress(t *testing.T) {
	// Generate a private key
	privateKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	require.NoError(t, err)

	testCases := []struct {
		name           string
		prefix         string
		expectedPrefix string
	}{
		{"Babylon", "bbn", "bbn"},
		{"Cosmos Hub", "cosmos", "cosmos"},
		{"Custom Chain", "custom", "custom"},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			address, err := EcdsaPrivateKeyToCosmosAddress(privateKey, tc.prefix)
			assert.NoError(t, err)
			assert.NotEmpty(t, address)
			assert.True(t, len(address) > len(tc.expectedPrefix))
			assert.Equal(t, tc.expectedPrefix, address[:len(tc.expectedPrefix)])

			// Additional checks
			assert.Regexp(t, "^"+tc.expectedPrefix+"1[a-zA-Z0-9]{38}$", address)
		})
	}
}
