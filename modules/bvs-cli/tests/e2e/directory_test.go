package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/directory"
)

var directoryKeyName = "caller"

// TODO: disabled transactional test, to be fixed in SL-182

//func Test_RegBVS(t *testing.T) {
//	BVSAddress := "bbn15zstqljcjlkyz4kmmhnhvq7mlprcccerukm9x27rt6teqelc057qhthy6l"
//	directory.RegBVS(directoryKeyName, BVSAddress)
//}
//
//func Test_UpdateMetadata(t *testing.T) {
//	metadata := "https://satlayer.com"
//	directory.UpdateMetadata(directoryKeyName, metadata)
//}
//

func Test_GetOperator(t *testing.T) {
	operatorAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	directory.GetOperator(operatorAddress)
}

func Test_CalcDigestHash(t *testing.T) {
	salt := "salt"
	expire := int64(1000)
	directory.CalcDigestHash(directoryKeyName, salt, expire)
}

func Test_IsSaltSpent(t *testing.T) {
	operatorAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	salt := "salt"
	directory.IsSaltSpent(operatorAddress, salt)
}

func Test_DomainName(t *testing.T) {
	directory.DomainName()
}

func Test_DomainTypeHash(t *testing.T) {
	directory.DomainTypeHash()
}

func Test_OperatorBvsRegistrationTypeHash(t *testing.T) {
	directory.OperatorBvsRegistrationTypeHash()
}

func Test_BvsInfo(t *testing.T) {
	hash := "3e9dd8890daa53e3b07af614231c9ccaac51805c449b8df61f62e2afb30d6685"
	directory.BvsInfo(hash)
}
