package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/slashevm"
)

var evmUserAddr = "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4"
var password = "123"

func Test_EVMIsValidator(t *testing.T) {
	validatorAddress := "0x0000000000000000000000000000000000000000"
	slashevm.IsValidator(validatorAddress)
}

func Test_EVMIsPaused(t *testing.T) {
	slashevm.IsPaused()
}

func Test_EVMGetMinimalSlashSignature(t *testing.T) {
	slashevm.GetMinimalSlashSignature()
}

func Test_EVMGetOwner(t *testing.T) {
	slashevm.GetOwner()
}

func Test_EVMGetPendingOwner(t *testing.T) {
	slashevm.GetPendingOwner()
}

func Test_EVMGetSlasher(t *testing.T) {
	slashevm.GetSlasher()
}

func Test_EVMSetSlasher(t *testing.T) {
	slashevm.SetSlasher(evmUserAddr, password, "0xC97705a8FaA9Bec2A50B5bbCc0661251BcB537A4")
}

func Test_EVMSetMinimalSlashSignature(t *testing.T) {
	slashevm.SetMinimalSlashSignature(evmUserAddr, password, 1)
}

func Test_EVMSetSlasherValidator(t *testing.T) {
	validators := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}
	values := []bool{true}
	slashevm.SetSlasherValidator(evmUserAddr, password, validators, values)
}

func Test_EVMPauseAndUnPaused(t *testing.T) {
	slashevm.Pause(evmUserAddr, password)
	slashevm.Unpause(evmUserAddr, password)
}

func Test_EVMSubmitSlashRequest(t *testing.T) {
	slasher := "bbn1m4gtpe3wfhlmvwultl678rxzexyduy60jjm6ty"
	operator := "bbn1rt6v30zxvhtwet040xpdnhz4pqt8p2za7y430x"
	share := 10
	slashSignature := 1
	slashValidators := []string{"bbn1m4gtpe3wfhlmvwultl678rxzexyduy60jjm6ty"}
	reason := "test"
	startTime := 1736144991696
	endTime := 1736144995296
	validatorsPublicKeys := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}
	slashevm.SubmitSlashRequest(evmUserAddr, password, slasher, operator, int64(share), uint16(slashSignature), slashValidators, reason, int64(startTime), int64(endTime), true, validatorsPublicKeys)
}

func Test_EVMExecuteSlashRequest(t *testing.T) {
	slashHash := "638789a8cd83d13edab39fcf89b7044e693a5e96ee56348328f2405442ec6d09"
	signatures := []string{"m2uENcWcVtC+JTVs5duyxS0tPJLH00s9wlKON/3fRdAhzsgS/AcdwGbHxUzbT5b/WlfANGHMDvmZy5eVyFnL9g=="}
	validatorsPublicKeys := []string{"A5/kz6NbN5TD//2K7tQ9JaUmu0gIbvJMTqgqR1iaWadA"}
	slashevm.ExecuteSlashRequest(evmUserAddr, password, signatures, validatorsPublicKeys, slashHash)
}

func Test_EVMCancelSlashRequest(t *testing.T) {
	slashHash := "8033994ebff512d0c140140ee6900064c9b98ff732b02320ae384ad3a7976966"
	slashevm.CancelSlashRequest(evmUserAddr, password, slashHash)
}
