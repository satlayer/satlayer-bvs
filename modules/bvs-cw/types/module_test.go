package types

import (
	"github.com/satlayer/satlayer-bvs/bvs-cw/types/bvs-driver"
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_GeneratedTypes(t *testing.T) {
	msg := bvs_driver.ExecuteMsg{
		TransferOwnership: &bvs_driver.TransferOwnership{
			NewOwner: "new_owner",
		},
	}

	msgBytes, err := msg.Marshal()
	assert.NoError(t, err)
	assert.Equal(t, `{"transfer_ownership":{"new_owner":"new_owner"}}`, string(msgBytes))
}
