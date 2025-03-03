package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategybase"
)

// TODO: disabled transactional test, to be fixed in SL-182
//func Test_GetShares(t *testing.T) {
//	stakerAddress := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
//	strategybase.GetShares(stakerAddress)
//}

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
