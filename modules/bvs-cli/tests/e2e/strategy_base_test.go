package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategybase"
)

func Test_GetShares(t *testing.T) {
	stakerAddress := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
	strategyAddress := "bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"
	strategybase.GetShares(stakerAddress, strategyAddress)
}

func Test_SharesUnderlyingView(t *testing.T) {
	strategybase.SharesUnderlyingView(12)
}

func Test_UnderlyingShareView(t *testing.T) {
	strategybase.UnderlyingShareView(12)
}

func Test_UnderlyingView(t *testing.T) {
	strategybase.UnderlyingView("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_UnderlyingToken(t *testing.T) {
	strategybase.UnderlyingToken()
}
