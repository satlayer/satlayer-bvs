package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/directoryevm"
)

//var evmUserAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"

func Test_EVMDirectoryRegBVS(t *testing.T) {
	var evmUserAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"
	var password = "123"
	bvsContract := "0xaA0851f2939EF2D8B51971B510383Fcb5c246a17"
	bvsHash := "226466AF1CF2ECDA66821E7833C325F15037D6BB7CC0CE39A8908587D02C1046"
	directoryevm.RegBVS(evmUserAddr, password, bvsHash, bvsContract)
}

func Test_EVMDirectoryGetOwner(t *testing.T) {
	directoryevm.GetOwner()
}

func Test_EVMDirectoryGetPendingOwner(t *testing.T) {
	directoryevm.GetPendingOwner()
}

func Test_EVMDirectoryGetBVSInfo(t *testing.T) {
	bvsHash := "d3e0d8390a391318ae2bc6a1383dafae297e402f8fcc783ac149e2cafb5d66fc"
	directoryevm.GetBVSInfo(bvsHash)
}
