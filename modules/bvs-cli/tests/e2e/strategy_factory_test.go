package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-cli/commands/strategyfactory"
)

var factoryUserKeyName = "caller"

func Test_IsTokenBlacklisted(t *testing.T) {
	tokenAddress := "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
	strategyfactory.IsTokenBlacklisted(tokenAddress)
}

func Test_UpdateConfig(t *testing.T) {
	newOwnerAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyCodeID := uint64(10995)
	strategyfactory.UpdateConfig(factoryUserKeyName, newOwnerAddress, strategyCodeID)
}

func Test_Factory_Pause(t *testing.T) {
	strategyfactory.Pause(factoryUserKeyName)
}

func Test_Factory_Unpause(t *testing.T) {
	strategyfactory.Unpause(factoryUserKeyName)
}

func Test_Factory_SetPauser(t *testing.T) {
	pauserAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyfactory.SetPauser(factoryUserKeyName, pauserAddress)
}

func Test_Factory_SetUnpauser(t *testing.T) {
	unpauserAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyfactory.SetUnpauser(factoryUserKeyName, unpauserAddress)
}

func Test_Factory_TransferOwnership(t *testing.T) {
	newOwnerAddress := "bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf"
	strategyfactory.TransferOwnership(factoryUserKeyName, newOwnerAddress)
}
