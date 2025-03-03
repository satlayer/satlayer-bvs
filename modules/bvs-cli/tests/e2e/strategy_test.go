package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategy"
)

// TODO: disabled transactional test, to be fixed in SL-182
//func Test_GetDeposits(t *testing.T) {
//	stakerAddress := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
//	strategy.GetDeposits(stakerAddress)
//}

func Test_GetStakerStrategyListLength(t *testing.T) {
	strategy.GetStakerStrategyListLength("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_GetStakerStrategyList(t *testing.T) {
	strategy.GetStakerStrategyList("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_GetStrategyDetails(t *testing.T) {
	strategy.IsStrategyWhitelisted("bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a")
}
