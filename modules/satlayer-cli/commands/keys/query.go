package keys

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/satlayer-cli/conf"
)

func Show(keyName string) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}

	pubKey := newChainIO.GetCurrentAccountPubKey()
	address := sdk.AccAddress(pubKey.Address()).String()
	fmt.Printf("- address: %s\n  name: %s\n  pubkey: %+v\n", address, keyName, pubKey)
}

func Check(keyName string) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}

	result := newChainIO.IsCurrentAccountOnChain()
	if !result {
		fmt.Printf("Failed. Your account is not on chain.\n")
		return
	}

	fmt.Printf("Success. Your account is on chain.\n")
}
