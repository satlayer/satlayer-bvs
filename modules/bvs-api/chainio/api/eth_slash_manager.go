package api

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/satlayer/satlayer-api/chainio/io"
	"github.com/satlayer/satlayer-api/chainio/types"
)

type ETHSlashManager interface {
	MinimalSlashSignature(ctx context.Context) (*big.Int, error)
	Owner(ctx context.Context) (common.Address, error)
	Paused(ctx context.Context) (bool, error)
	PendingOwner(ctx context.Context) (common.Address, error)
	Slasher(ctx context.Context) (common.Address, error)
	Validators(ctx context.Context, str string) (bool, error)

	CancelSlashRequest(ctx context.Context, wallet types.ETHWallet, slashHash string) (*sdktypes.Receipt, error)
	ExecuteSlashRequest(ctx context.Context, wallet types.ETHWallet, slashHash string, signatures []string, validatorsPublicKeys []string) (*sdktypes.Receipt, error)
	SetMinimalSlashSignature(ctx context.Context, wallet types.ETHWallet, minimalSlashSignature *big.Int) (*sdktypes.Receipt, error)
	SetSlashValidator(ctx context.Context, wallet types.ETHWallet, validators []string, values []bool) (*sdktypes.Receipt, error)
	SetSlasher(ctx context.Context, wallet types.ETHWallet, slasher common.Address) (*sdktypes.Receipt, error)
	SubmitSlashRequest(ctx context.Context, wallet types.ETHWallet, executeSlashDetails types.SlashDetails, validatorsPublicKeys []string) (*sdktypes.Receipt, error)
	Initialize(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	Pause(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	Unpause(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error)
	TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error)
}

type ethSlashManagerImpl struct {
	io           io.ETHChainIO
	contractAddr common.Address
	contractABI  *abi.ABI
}

func (e *ethSlashManagerImpl) MinimalSlashSignature(ctx context.Context) (*big.Int, error) {
	result := new(big.Int)
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "minimalSlashSignature",
		Args:         []interface{}{},
	}, &result); err != nil {
		return result, err
	}
	return result, nil
}
func (e *ethSlashManagerImpl) Owner(ctx context.Context) (common.Address, error) {
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

func (e *ethSlashManagerImpl) Paused(ctx context.Context) (bool, error) {
	var result bool
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "paused",
		Args:         []interface{}{},
	}, &result); err != nil {
		return false, err
	}
	return result, nil
}

func (e *ethSlashManagerImpl) PendingOwner(ctx context.Context) (common.Address, error) {
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

func (e *ethSlashManagerImpl) Slasher(ctx context.Context) (common.Address, error) {
	var addr common.Address
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "slasher",
		Args:         []interface{}{},
	}, &addr); err != nil {
		return common.Address{}, err
	}
	return addr, nil
}

func (e *ethSlashManagerImpl) Validators(ctx context.Context, str string) (bool, error) {
	var result bool
	if err := e.io.CallContract(ctx, types.ETHCallOptions{
		ContractAddr: e.contractAddr,
		ContractABI:  e.contractABI,
		Method:       "validators",
		Args:         []interface{}{str},
	}, &result); err != nil {
		return false, err
	}
	return result, nil
}

func (e *ethSlashManagerImpl) CancelSlashRequest(ctx context.Context, wallet types.ETHWallet, slashHash string) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "cancelSlashRequest",
			Args:         []interface{}{slashHash},
		},
	})
}

func (e *ethSlashManagerImpl) ExecuteSlashRequest(ctx context.Context, wallet types.ETHWallet, slashHash string, signatures []string, validatorsPublicKeys []string) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "executeSlashRequest",
			Args:         []interface{}{slashHash, signatures, validatorsPublicKeys},
		},
	})
}

func (e *ethSlashManagerImpl) SetMinimalSlashSignature(ctx context.Context, wallet types.ETHWallet, minimalSlashSignature *big.Int) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "setMinimalSlashSignature",
			Args:         []interface{}{minimalSlashSignature},
		},
	})
}

func (e *ethSlashManagerImpl) SetSlashValidator(ctx context.Context, wallet types.ETHWallet, validators []string, values []bool) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "setSlashValidator",
			Args:         []interface{}{validators, values},
		},
	})
}

func (e *ethSlashManagerImpl) SetSlasher(ctx context.Context, wallet types.ETHWallet, slasher common.Address) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "setSlasher",
			Args:         []interface{}{slasher},
		},
	})
}

func (e *ethSlashManagerImpl) SubmitSlashRequest(ctx context.Context, wallet types.ETHWallet, executeSlashDetails types.SlashDetails, validatorsPublicKeys []string) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "submitSlashRequest",
			Args:         []interface{}{executeSlashDetails, validatorsPublicKeys},
		},
	})
}

func (e *ethSlashManagerImpl) Initialize(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "initialize",
			Args:         []interface{}{},
		},
	})
}

func (e *ethSlashManagerImpl) Pause(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "pause",
			Args:         []interface{}{},
		},
	})
}

func (e *ethSlashManagerImpl) Unpause(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
	return e.io.SendTransaction(ctx, types.ETHExecuteOptions{
		ETHWallet: wallet,
		ETHCallOptions: types.ETHCallOptions{
			ContractAddr: e.contractAddr,
			ContractABI:  e.contractABI,
			Method:       "unpause",
			Args:         []interface{}{},
		},
	})
}

func (e *ethSlashManagerImpl) AcceptOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
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

func (e *ethSlashManagerImpl) RenounceOwnership(ctx context.Context, wallet types.ETHWallet) (*sdktypes.Receipt, error) {
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

func (e *ethSlashManagerImpl) TransferOwnership(ctx context.Context, wallet types.ETHWallet, newOwner common.Address) (*sdktypes.Receipt, error) {
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

func NewETHSlashManagerImpl(chainIO io.ETHChainIO, contractAddr common.Address, contractABI *abi.ABI) ETHSlashManager {
	return &ethSlashManagerImpl{
		io:           chainIO,
		contractABI:  contractABI,
		contractAddr: contractAddr,
	}
}
