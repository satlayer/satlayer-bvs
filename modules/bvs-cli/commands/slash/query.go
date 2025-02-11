package slash

import (
	"encoding/hex"
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func GetSlashDetails(slashHash string) {
	s := NewService()
	resp, err := s.Slash.GetSlashDetails(slashHash)
	if err != nil {
		panic(err)
	}
	getSlashDetails := resp.SlashDetails
	fmt.Printf("%+v\n", getSlashDetails)
}

func IsValidator(validator string) {
	s := NewService()
	resp, err := s.Slash.IsValidator(validator)
	if err != nil {
		panic(fmt.Sprintf("Failed to query validator: %v", err))
	}
	if resp == nil {
		panic("Received nil response from validator query")
	}

	isValidator := resp.IsValidator
	fmt.Printf("IsValidator response: %t\n", isValidator)
}

func GetMinimalSlashSignature() {
	s := NewService()
	resp, err := s.Slash.GetMinimalSlashSignature()
	if err != nil {
		panic(fmt.Sprintf("Error while fetching minimal slash signature: %v", err))
	}

	if resp == nil {
		panic("Received nil response from GetMinimalSlashSignature")
	}

	fmt.Printf("Minimal slash signature value: %d\n", resp.MinimalSlashSignature)
}

func CalculateSlashHash(KeyNames []string, sender string, slasher string, operator string, share string, slashSignature uint64, slashValidators []string, reason string, startTime uint64, endTime uint64, status bool) {
	s := NewService()

	var validatorsPublicKeys []cryptotypes.PubKey

	for _, keyName := range KeyNames {
		newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
		if err != nil {
			panic(fmt.Sprintf("Failed to setup keyring for %s: %v", keyName, err))
		}

		pubKey := newChainIO.GetCurrentAccountPubKey()
		validatorsPublicKeys = append(validatorsPublicKeys, pubKey)
	}

	slashDetails := types.ExecuteSlashDetails{
		Slasher:        slasher,
		Operator:       operator,
		Share:          share,
		SlashSignature: slashSignature,
		SlashValidator: slashValidators,
		Reason:         reason,
		StartTime:      startTime,
		EndTime:        endTime,
		Status:         status,
	}

	resp, err := s.Slash.CalculateSlashHash(sender, slashDetails, validatorsPublicKeys)
	if err != nil {
		panic(fmt.Sprintf("Failed to calculate slash hash: %v", err))
	}

	slashHashHex := hex.EncodeToString(resp.MessageBytes)
	fmt.Printf("Slash Hash (Hex): %s\n", slashHashHex)
}
