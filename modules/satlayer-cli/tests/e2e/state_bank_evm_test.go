package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/statebankevm"
)

// HACK: SL-96 - disable on chain registration due to hard coded contract address
// until we can run testnet locally
// func Test_EVMStateBankRegBVS(t *testing.T) {
// 	var evmUserAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"
// 	var password = "123"
// 	bvsContract := "0xaA0851f2939EF2D8B51971B510383Fcb5c246a17"
// 	statebankevm.RegBVS(evmUserAddr, password, bvsContract)
// }

func Test_EVMStateBankGetOwner(t *testing.T) {
	statebankevm.GetOwner()
}

func Test_EVMStateBankGetPendingOwner(t *testing.T) {
	statebankevm.GetPendingOwner()
}

func Test_EVMStateBankIsRegistered(t *testing.T) {
	bvsContract := "0xaA0851f2939EF2D8B51971B510383Fcb5c246a17"
	statebankevm.IsBVSRegistered(bvsContract)
}
