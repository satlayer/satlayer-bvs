package slash

import (
	"context"
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"

	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
)

func newService(keyName string) (api.SlashManager, io.ChainIO) {
	s := NewService()
	chainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	slash := api.NewSlashManager(chainIO, conf.C.Contract.Slash).WithGasLimit(2000000)
	slash.BindClient(conf.C.Contract.Slash)
	return slash, chainIO
}

func SetSlasher(userKeyName string, slasher string, value bool) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetSlasher(ctx, slasher, value)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set slasher success. txn: %s\n", txResp.Hash)
}

func SetDelegationManager(userKeyName, newDelegationManager string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetDelegationManager(ctx, newDelegationManager)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set delegation manager success. txn: %s\n", txResp.Hash)
}

func SetMinimalSlashSignature(userKeyName string, minimalSignature int64) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetMinimalSlashSignature(ctx, minimalSignature)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set minimal slash signature success. txn: %s\n", txResp.Hash)
}

func SetPauser(userKeyName, newPauser string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetPauser(ctx, newPauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", txResp.Hash)
}

func SetUnpauser(userKeyName, newUnpauser string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetUnpauser(ctx, newUnpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", txResp.Hash)
}

func SetSlasherValidator(userKeyName string, validators []string, values []bool) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.SetSlasherValidator(ctx, validators, values)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set slasher validator success. txn: %s\n", txResp.Hash)
}

func Pause(userKeyName string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.Pause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", txResp.Hash)
}

func Unpause(userKeyName string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.Unpause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", txResp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	ctx := context.Background()
	slash, _ := newService(userKeyName)
	txResp, err := slash.TransferOwnership(ctx, newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", txResp.Hash)
}

func SubmitSlashRequest(userKeyNames []string, slasher string, operator string, share string, slashSignature int64, slashValidators []string, reason string, startTime int64, endTime int64, status bool) {
	s := NewService()
	ctx := context.Background()

	var validatorsPublicKeys []cryptotypes.PubKey

	for _, keyName := range userKeyNames {
		newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
		if err != nil {
			panic(fmt.Sprintf("Failed to setup keyring for %s: %v", keyName, err))
		}

		pubKey := newChainIO.GetCurrentAccountPubKey()
		validatorsPublicKeys = append(validatorsPublicKeys, pubKey)
	}

	slashDetails := slashmanager.SubmitSlashRequestSlashDetails{
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

	txResp, err := s.Slash.SubmitSlashRequest(ctx, slashDetails, validatorsPublicKeys)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Submit slash request success. txn: %s\n", txResp.Hash)
}

func ExecuteSlashRequest(userKeyNames []string, slashHash string) {
	s := NewService()
	ctx := context.Background()

	var validatorsPublicKeys []cryptotypes.PubKey

	for _, keyName := range userKeyNames {
		newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
		if err != nil {
			panic(fmt.Sprintf("Failed to setup keyring for %s: %v", keyName, err))
		}

		pubKey := newChainIO.GetCurrentAccountPubKey()
		validatorsPublicKeys = append(validatorsPublicKeys, pubKey)
	}

	txResp, err := s.Slash.ExecuteSlashRequest(ctx, slashHash, validatorsPublicKeys)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Execute slash request success. txn: %s\n", txResp.Hash)
}

func CancelSlashRequest(userKeyNames, slashHash string) {
	ctx := context.Background()
	slash, _ := newService(userKeyNames)
	txResp, err := slash.CancelSlashRequest(ctx, slashHash)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Cancel slash request success. txn: %s\n", txResp.Hash)
}
