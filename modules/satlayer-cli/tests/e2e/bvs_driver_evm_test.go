package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/bvsdriverevm"
)

// HACK: SL-96 - disable on chain registration due to hard coded contract address
// until we can run testnet locally
// func Test_EVMBVSDriverRegBVS(t *testing.T) {
// 	var evmUserAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"
// 	var password = "123"
// 	bvsContract := "0xaA0851f2939EF2D8B51971B510383Fcb5c246a17"
// 	bvsdriverevm.RegBVS(evmUserAddr, password, bvsContract)
// }

func Test_EVMBVSDriverGetOwner(t *testing.T) {
	bvsdriverevm.GetOwner()
}

func Test_EVMBVSDriverGetPendingOwner(t *testing.T) {
	bvsdriverevm.GetPendingOwner()
}

func Test_EVMBVSDriverIsRegisted(t *testing.T) {
	bvsHash := "d3e0d8390a391318ae2bc6a1383dafae297e402f8fcc783ac149e2cafb5d66fc"
	bvsdriverevm.IsBVSRegistered(bvsHash)
}
