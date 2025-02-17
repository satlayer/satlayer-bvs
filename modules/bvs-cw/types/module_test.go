package types

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cw/types/driver"
	"github.com/stretchr/testify/assert"
)

func TestGeneratedTypes(t *testing.T) {
	msg := driver.ExecuteMsg{
		TransferOwnership: &driver.TransferOwnership{
			NewOwner: "new_owner",
		},
	}

	msgBytes, err := msg.Marshal()
	assert.NoError(t, err)
	assert.Equal(t, `{"transfer_ownership":{"new_owner":"new_owner"}}`, string(msgBytes))
}
