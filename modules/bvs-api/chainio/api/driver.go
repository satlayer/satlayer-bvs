package api

import (
	"context"
	"encoding/json"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-cw/driver"
)

type BVSDriver interface {
	WithGasAdjustment(gasAdjustment float64) BVSDriver
	WithGasPrice(gasPrice sdktypes.DecCoin) BVSDriver
	WithGasLimit(gasLimit uint64) BVSDriver

	SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error)
	BindClient(contractAddress string)
}

type bvsDriverImpl struct {
	registeredBVSContract string
	io                    io.ChainIO
	contractAddr          string
	gasAdjustment         float64
	gasPrice              sdktypes.DecCoin
	gasLimit              uint64
}

func NewBvsDriver(chainIO io.ChainIO) BVSDriver {
	return &bvsDriverImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *bvsDriverImpl) WithGasAdjustment(gasAdjustment float64) BVSDriver {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *bvsDriverImpl) WithGasPrice(gasPrice sdktypes.DecCoin) BVSDriver {
	r.gasPrice = gasPrice
	return r
}

func (r *bvsDriverImpl) WithGasLimit(gasLimit uint64) BVSDriver {
	r.gasLimit = gasLimit
	return r
}

func (r *bvsDriverImpl) BindClient(contractAddress string) {
	r.contractAddr = contractAddress
}

func (r *bvsDriverImpl) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  r.contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (r *bvsDriverImpl) SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
	r.registeredBVSContract = addr

	msg := driver.ExecuteMsg{
		AddRegisteredBvsContract: &driver.AddRegisteredBvsContract{
			Address: addr,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	executeOptions := r.newExecuteOptions(msgBytes, "SetRegisteredBVSContract")
	return r.io.SendTransaction(ctx, executeOptions)
}
