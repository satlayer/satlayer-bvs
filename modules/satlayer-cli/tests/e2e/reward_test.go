package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/reward"
)

func Test_GetDistributionRootLength(t *testing.T) {
	reward.GetDistributionRootLength()
}

func Test_GetCurrentDistributionRoot(t *testing.T) {
	reward.GetCurrentDistributionRoot()
}

func Test_GetDistributionRootAtIndex(t *testing.T) {
	reward.GetDistributionRootAtIndex("0")
}
