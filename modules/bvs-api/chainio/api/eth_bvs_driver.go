package api

import (
	"context"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
)

type ETHBVSDriver interface {
	IsBVSContractRegistered(ctx context.Context, bvsContract common.Address) (bool, error)
	Owner(ctx context.Context) (common.Address, error)
	PendingOwner(ctx context.Context) (common.Address, error)

	AddRegisteredBvsContract(ctx context.Context, wallet types.ETHWallet, contractAddress common.Address) (*sdktypes.Receipt, error)
	ExecuteBVSOffChain(ctx context.Context, wallet types.ETHWallet, taskID string) (*sdktypes.Receipt, error)
	AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error)
}

type ethBVSDriverImpl struct {
	io           io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (e *ethBVSDriverImpl) IsBVSContractRegistered(ctx context.Context, bvsContract common.Address) (bool, error) {
	var isRegistered bool
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "isBvsContractRegistered",
		Args:         []interface{}{bvsContract},
	}, &isRegistered); err != nil {
		return false, err
	}

	return isRegistered, nil
}

func (e *ethBVSDriverImpl) Owner(ctx context.Context) (common.Address, error) {
	var addr common.Address
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "owner",
		Args:         []interface{}{},
	}, &addr); err != nil {
		return common.Address{}, err
	}
	return addr, nil
}
func (e *ethBVSDriverImpl) PendingOwner(ctx context.Context) (common.Address, error) {
	var addr common.Address
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "pendingOwner",
		Args:         []interface{}{},
	}, &addr); err != nil {
		return common.Address{}, err
	}
	return addr, nil
}

func (e *ethBVSDriverImpl) AddRegisteredBvsContract(ctx context.Context, wallet types.ETHWallet, contractAddress common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "addRegisteredBvsContract",
			Args:         []interface{}{contractAddress},
		},
	})
}

func (e *ethBVSDriverImpl) ExecuteBVSOffChain(ctx context.Context, wallet types.ETHWallet, taskID string) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "executeBvsOffchain",
			Args:         []interface{}{taskID},
		},
	})
}

func (e *ethBVSDriverImpl) AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "acceptOwnership",
			Args:         []interface{}{},
		},
	})
}

func (e *ethBVSDriverImpl) RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "renounceOwnership",
			Args:         []interface{}{},
		},
	})
}

func (e *ethBVSDriverImpl) TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "transferOwnership",
			Args:         []interface{}{newOwner},
		},
	})
}

func NewETHBVSDriverImpl(chainIO io.ETHChainIO, contractAddr common.Address, contractABI *abi.ABI) ETHBVSDriver {
	return &ethBVSDriverImpl{
		io:           chainIO,
		contractABI:  contractABI,
		contractAddr: contractAddr,
	}
}
