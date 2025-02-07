package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/chain"
)

func Test_QueryNode(t *testing.T) {
	chain.QueryNode()
}

func Test_QueryAccount(t *testing.T) {
	account1 := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	chain.QueryAccount(account1)
}
