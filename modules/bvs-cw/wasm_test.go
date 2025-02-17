package bvscw

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/satlayer/satlayer-bvs/bvs-cw/driver"
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
