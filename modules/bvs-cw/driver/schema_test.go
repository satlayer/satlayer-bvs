package driver

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGeneratedTypes(t *testing.T) {
	msg := ExecuteMsg{
		TransferOwnership: &TransferOwnership{
			NewOwner: "new_owner",
		},
	}

	msgBytes, err := msg.Marshal()
	assert.NoError(t, err)
	assert.Equal(t, `{"transfer_ownership":{"new_owner":"new_owner"}}`, string(msgBytes))
}
