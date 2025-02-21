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

type Driver struct {
	registeredBVSContract string
	io                    io.ChainIO
	contractAddr          string
	gasAdjustment         float64
	gasPrice              sdktypes.DecCoin
	gasLimit              uint64
}

func NewDriver(chainIO io.ChainIO) *Driver {
	return &Driver{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *Driver) WithGasAdjustment(gasAdjustment float64) *Driver {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *Driver) WithGasPrice(gasPrice sdktypes.DecCoin) *Driver {
	r.gasPrice = gasPrice
	return r
}

func (r *Driver) WithGasLimit(gasLimit uint64) *Driver {
	r.gasLimit = gasLimit
	return r
}

func (r *Driver) BindClient(contractAddress string) {
	r.contractAddr = contractAddress
}

func (r *Driver) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *Driver) SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
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
