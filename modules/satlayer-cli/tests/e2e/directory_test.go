package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/commands/directory"
)

var directoryKeyName = "caller"

func Test_RegBVS(t *testing.T) {
	BVSAddress := "bbn15zstqljcjlkyz4kmmhnhvq7mlprcccerukm9x27rt6teqelc057qhthy6l"
	chainName := "babylond"
	chainId := "sat-bbn-testnet1"
	directory.RegBVS(directoryKeyName, BVSAddress, chainName, chainId)
}

func Test_UpdateMetadata(t *testing.T) {
	metadata := "https://satlayer.com"
	directory.UpdateMetadata(directoryKeyName, metadata)
}

func Test_GetOperator(t *testing.T) {
	operatorAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	directory.GetOperator(operatorAddress)
}

func Test_CalcDigestHash(t *testing.T) {
	salt := "salt"
	expire := uint64(1000)
	directory.CalcDigestHash(directoryKeyName, salt, expire)
}

func Test_IsSaltSpent(t *testing.T) {
	operatorAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	salt := "salt"
	directory.IsSaltSpent(operatorAddress, salt)
}

func Test_GetDelegationManager(t *testing.T) {
	directory.GetDelegationManager()
}

func Test_GetOwner(t *testing.T) {
	directory.GetOwner()
}

func Test_GetDomainName(t *testing.T) {
	directory.GetDomainName()
}

func Test_GetDomainTypeHash(t *testing.T) {
	directory.GetDomainTypeHash()
}

func Test_GetOperatorBVSRegistrationTypeHash(t *testing.T) {
	directory.GetOperatorBVSRegistrationTypeHash()
}

func Test_GetBVSInfo(t *testing.T) {
	hash := "23b6ff0d376632650684c0d3e773eaff7faaf864ef5d05a99eddcd51c99efc74"
	directory.GetBVSInfo(hash)
}
