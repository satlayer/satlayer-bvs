package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategybasetvl"
)

var userKeyName = "caller"

func Test_TVL_GetShares(t *testing.T) {
	stakerAddress := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
	strategyAddress := "bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"
	strategybasetvl.GetShares(stakerAddress, strategyAddress)
}

func Test_TVL_SharesUnderlyingView(t *testing.T) {
	strategybasetvl.SharesUnderlyingView(12)
}

func Test_TVL_UnderlyingShareView(t *testing.T) {
	strategybasetvl.UnderlyingShareView(12)
}

func Test_TVL_UnderlyingView(t *testing.T) {
	strategybasetvl.UnderlyingView("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_TVL_UnderlyingToken(t *testing.T) {
	strategybasetvl.UnderlyingToken()
}

// TODO(post-hermetic): break due to TVL to tvl rename
func test_TVL_GetLimits(t *testing.T) {
	strategybasetvl.GetLimits()
}

func Test_TVL_GetStrategyManager(t *testing.T) {
	strategybasetvl.GetStrategyManager()
}

func Test_TVL_GetStrategyState(t *testing.T) {
	strategybasetvl.GetStrategyState()
}

func Test_TVL_GetTotalShares(t *testing.T) {
	strategybasetvl.GetTotalShares()
}

func Test_TVL_Explanation(t *testing.T) {
	strategybasetvl.Explanation()
}

// TODO: disabled transactional test, to be fixed in SL-182

//func Test_Pause(t *testing.T) {
//	strategybasetvl.Pause(userKeyName)
//}
//
//func Test_Unpause(t *testing.T) {
//	strategybasetvl.Unpause(userKeyName)
//}
//
//func Test_SetPauser(t *testing.T) {
//	strategybasetvl.SetPauser(userKeyName, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
//}
//
//func Test_SetUnpauser(t *testing.T) {
//	strategybasetvl.SetUnpauser(userKeyName, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
//}
//
//func Test_TransferOwnership(t *testing.T) {
//	strategybasetvl.TransferOwnership(userKeyName, "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf")
//}
//
//func Test_SetTvlLimits(t *testing.T) {
//	strategybasetvl.SetTvlLimits(userKeyName, "10000000", "12000000")
//}
