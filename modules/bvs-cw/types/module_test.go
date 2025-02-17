package types

import (
	"github.com/satlayer/satlayer-bvs/bvs-cw/types/bvs-driver"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestGeneratedTypes(t *testing.T) {
	msg := bvsdriver.ExecuteMsg{
		TransferOwnership: &bvsdriver.TransferOwnership{
			NewOwner: "new_owner",
		},
	}

	msgBytes, err := msg.Marshal()
	assert.NoError(t, err)
	assert.Equal(t, `{"transfer_ownership":{"new_owner":"new_owner"}}`, string(msgBytes))
}
