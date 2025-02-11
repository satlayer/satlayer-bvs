package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/delegation"
)

var delegationUserKey = "operator1"

func Test_UpdateOperatorDetails(t *testing.T) {
	receiver := "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv"
	delegationApprover := "bbn1yh5vdtu8n55f2e4fjea8gh0dw9gkzv7uxt8jrv"
	stakerOptOutWindowBlocks := uint64(0)
	delegation.UpdateOperatorDetails(delegationUserKey, receiver, delegationApprover, stakerOptOutWindowBlocks)
}

func Test_UpdateOperatorMetadataURI(t *testing.T) {
	uri := "metadata.uri"
	delegation.UpdateOperatorMetadataURI(delegationUserKey, uri)
}

func Test_IsDelegated(t *testing.T) {
	staker := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
	delegation.IsDelegated(staker)
}
func Test_IsOperator(t *testing.T) {
	operator := "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x"
	delegation.IsOperator(operator)
}

func Test_OperatorDetails(t *testing.T) {
	operator := "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x"
	delegation.GetOperatorDetails(operator)
}
