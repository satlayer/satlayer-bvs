package api

import (
	"context"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

type ETHBVSDirectory interface {
	GetBVSInfo(ctx context.Context, bvsHash string) (*types.GetETHBVSInfoResp, error)
	Owner(ctx context.Context) (common.Address, error)
	PendingOwner(ctx context.Context) (common.Address, error)

	RegisterBVS(ctx context.Context, wallet types.ETHWallet, bvsHash string, bvsContract common.Address) (*sdktypes.Receipt, error)
	AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error)
}

type ethBVSDirectoryImpl struct {
	io           io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (e *ethBVSDirectoryImpl) GetBVSInfo(ctx context.Context, bvsHash string) (*types.GetETHBVSInfoResp, error) {
	resp := new(types.GetETHBVSInfoResp)
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "getBVSInfo",
		Args:         []interface{}{bvsHash},
	}, &resp); err != nil {
		return nil, err
	}

	return resp, nil
}

func (e *ethBVSDirectoryImpl) Owner(ctx context.Context) (common.Address, error) {
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
func (e *ethBVSDirectoryImpl) PendingOwner(ctx context.Context) (common.Address, error) {
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

func (e *ethBVSDirectoryImpl) RegisterBVS(ctx context.Context, wallet types.ETHWallet, bvsHash string, bvsContract common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "registerBvs",
			Args:         []interface{}{bvsHash, bvsContract},
		},
	})
}

func (e *ethBVSDirectoryImpl) AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
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

func (e *ethBVSDirectoryImpl) RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
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

func (e *ethBVSDirectoryImpl) TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error) {
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

func NewETHBVSDirectoryImpl(chainIO io.ETHChainIO, contractAddr common.Address, contractABI *abi.ABI) ETHBVSDirectory {
	return &ethBVSDirectoryImpl{
		io:           chainIO,
		contractABI:  contractABI,
		contractAddr: contractAddr,
	}
}
