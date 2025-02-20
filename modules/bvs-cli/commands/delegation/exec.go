package delegation

import (
	"context"
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func newService(keyName string) (api.Delegation, io.ChainIO) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(keyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	delegation := api.NewDelegationImpl(newChainIO, conf.C.Contract.Delegation).WithGasLimit(400000)
	return delegation, newChainIO
}

func RegOperator(KeyName, approverAddress string) {
	ctx := context.Background()
	delegation, newChainIO := newService(KeyName)
	pubKey := newChainIO.GetCurrentAccountPubKey()
	txResp, err := delegation.RegisterAsOperator(
		ctx,
		pubKey,
		"",
		approverAddress,
		"",
		0,
	)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Delegation Register operator success. txn: %s\n", txResp.Hash)
}

func UpdateOperatorDetails(userKeyName, receiver, delegationApprover string, stakerOptOutWindowBlocks int64) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.ModifyOperatorDetails(
		ctx,
		receiver,
		delegationApprover,
		stakerOptOutWindowBlocks,
	)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Delegation Update operator details success. txn: %s\n", txResp.Hash)
}

func UpdateOperatorMetadataURI(userKeyName, uri string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.UpdateOperatorMetadataURI(ctx, uri)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Delegation Update operator metadata uri success. txn: %s\n", txResp.Hash)
}

func DelegateTo(stakerKeyName, operatorAddress, approverKeyName string) {
	s := NewService()
	ctx := context.Background()
	approverAddress := "0"
	var approverPubKey cryptotypes.PubKey = nil
	if approverKeyName != "" {
		newChainIO, err := s.ChainIO.SetupKeyring(approverKeyName, conf.C.Account.KeyringBackend)
		if err != nil {
			panic(err)
		}
		approverPubKey = newChainIO.GetCurrentAccountPubKey()
		approverAddress = sdk.AccAddress(approverPubKey.Address()).String()
	}
	newChainIO, err := s.ChainIO.SetupKeyring(stakerKeyName, conf.C.Account.KeyringBackend)
	delegation := api.NewDelegationImpl(newChainIO, conf.C.Contract.Delegation).WithGasLimit(400000)
	txResp, err := delegation.DelegateTo(
		ctx,
		operatorAddress,
		approverAddress,
		approverKeyName,
		approverPubKey,
	)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Delegation Delegate to success. txn: %s\n", txResp.Hash)
}

func Undelegate(stakerKeyName, operatorAddress string) {
	ctx := context.Background()
	delegation, _ := newService(stakerKeyName)
	txResp, err := delegation.UnDelegate(ctx, operatorAddress)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Undelegate success. txn: %s\n", txResp.Hash)
}

func DelegateBySignature(stakerKeyName, operatorAddress, approverKeyName string) {
	s := NewService()
	ctx := context.Background()

	approverAddress := "0"
	var approverPubKey cryptotypes.PubKey = nil
	if approverKeyName != "" {
		newChainIO, err := s.ChainIO.SetupKeyring(approverKeyName, conf.C.Account.KeyringBackend)
		if err != nil {
			panic(err)
		}
		approverPubKey = newChainIO.GetCurrentAccountPubKey()
		approverAddress = sdk.AccAddress(approverPubKey.Address()).String()
	}
	newChainIO, err := s.ChainIO.SetupKeyring(stakerKeyName, conf.C.Account.KeyringBackend)
	if err != nil {
		panic(err)
	}
	stakerPubKey := newChainIO.GetCurrentAccountPubKey()
	stakerAddress := sdk.AccAddress(stakerPubKey.Address()).String()
	delegation := api.NewDelegationImpl(newChainIO, conf.C.Contract.Delegation).WithGasLimit(400000)
	txResp, err := delegation.DelegateToBySignature(
		ctx,
		operatorAddress,
		stakerAddress,
		stakerKeyName,
		approverAddress,
		approverKeyName,
		stakerPubKey,
		approverPubKey,
	)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Delegation DelegateBySignature success. txn: %s\n", txResp.Hash)
}

func SetMinWithdrawDelayBlocks(userKeyName string, blocks int64) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.SetMinWithdrawalDelayBlocks(ctx, blocks)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set min withdraw delay blocks success. txn: %s\n", txResp.Hash)
}

func SetStrategyWithdrawDelayBlocks(userKeyName string, strategies []string, blocks []int64) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.SetStrategyWithdrawalDelayBlocks(ctx, strategies, blocks)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set strategy withdraw delay blocks success. txn: %s\n", txResp.Hash)
}

func TransferOwnership(userKeyName, newOwner string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.TransferOwnership(ctx, newOwner)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Transfer ownership success. txn: %s\n", txResp.Hash)
}

func Pause(userKeyName string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.Pause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Pause success. txn: %s\n", txResp.Hash)
}

func Unpause(userKeyName string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.Unpause(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Unpause success. txn: %s\n", txResp.Hash)
}

func SetPauser(userKeyName, newPauser string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.SetPauser(ctx, newPauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set pauser success. txn: %s\n", txResp.Hash)
}

func SetUnpauser(userKeyName, newUnpauser string) {
	ctx := context.Background()
	delegation, _ := newService(userKeyName)
	txResp, err := delegation.SetUnpauser(ctx, newUnpauser)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Set unpauser success. txn: %s\n", txResp.Hash)
}
