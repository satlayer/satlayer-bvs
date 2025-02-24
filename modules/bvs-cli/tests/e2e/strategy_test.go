package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategy"
)

func Test_GetDeposits(t *testing.T) {
	stakerAddress := "bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk"
	strategyAddress := "bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a"
	strategy.GetDeposits(stakerAddress, strategyAddress)
}

func Test_GetStakerStrategyListLength(t *testing.T) {
	strategy.GetStakerStrategyListLength("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_IsThirdTransferForbidden(t *testing.T) {
	strategy.IsThirdTransferForbidden("bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a")
}

func Test_GetNonce(t *testing.T) {
	strategy.GetNonce("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_GetStakerStrategyList(t *testing.T) {
	strategy.GetStakerStrategyList("bbn1yph32eys4tdzv47dymfmn4el9x3k5rvpgjnphk")
}

func Test_GetStrategyOwner(t *testing.T) {
	strategy.GetOwner()
}

func Test_GetStrategyDetails(t *testing.T) {
	strategy.IsStrategyWhiteListed("bbn102zy555uul67xct4f29plgt6wq63wacmjp93csxpz8z538jrzcdqmj993a")
}

func Test_GetStrategyManagerState(t *testing.T) {
	strategy.GetStrategyManagerState()
}

// TODO(post-hermetic): break due to rename from GetDepositTypehash
func test_GetDepositTypeHash(t *testing.T) {
	strategy.GetDepositTypeHash()
}

// TODO(post-hermetic): break due to rename from GetDomainTypehash
func test_GetStrategyDomainTypeHash(t *testing.T) {
	strategy.GetDomainTypeHash()
}

func Test_GetStrategyDomainName(t *testing.T) {
	strategy.GetDomainName()
}

func Test_GetStrategyDelegationManager(t *testing.T) {
	strategy.GetDelegationManager()
}
