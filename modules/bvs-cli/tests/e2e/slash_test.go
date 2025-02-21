package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/slash"
)

var slashKeyName = "caller"

// TODO: disabled transactional test, to be fixed in SL-182
//func Test_GetSlashDetails(t *testing.T) {
//	slashHash := "8644527532e4230b12809aade8cf2aa018b24e7c1f1051562d653744cc49bcab"
//	slash.GetSlashDetails(slashHash)
//}

func Test_IsValidator(t *testing.T) {
	validatorAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	slash.IsValidator(validatorAddress)
}

func Test_GetMinimalSlashSignature(t *testing.T) {
	slash.GetMinimalSlashSignature()
}

//func Test_SetSlasher(t *testing.T) {
//	slasherAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
//	value := true
//	slash.SetSlasher(slashKeyName, slasherAddress, value)
//}
//
//func Test_SetDelegationManager(t *testing.T) {
//	newDelegationManager := "bbn1fys7yex6zgt73870uzewwglk0dzrwy6fsp0c4tent3080n3235ls6upg7c"
//	slash.SetDelegationManager(slashKeyName, newDelegationManager)
//}
//
//func Test_SetMinimalSlashSignature(t *testing.T) {
//	minimalSignature := int64(1)
//	slash.SetMinimalSlashSignature(slashKeyName, minimalSignature)
//}
//
//func Test_Slash_SetPauser(t *testing.T) {
//	newPauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
//	slash.SetPauser(slashKeyName, newPauser)
//}
//
//func Test_Slash_SetUnpauser(t *testing.T) {
//	newUnpauser := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
//	slash.SetUnpauser(slashKeyName, newUnpauser)
//}
//
//func Test_Slash_SetSlasherValidator(t *testing.T) {
//	validators := []string{"bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"}
//	values := []bool{true}
//	slash.SetSlasherValidator(slashKeyName, validators, values)
//}
//
//func Test_Slash_Pause(t *testing.T) {
//	slash.Pause(slashKeyName)
//}
//
//func Test_Slash_Unpause(t *testing.T) {
//	slash.Unpause(slashKeyName)
//}
//
//func Test_Slash_TransferOwnership(t *testing.T) {
//	newOwner := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
//	slash.TransferOwnership(slashKeyName, newOwner)
//}
