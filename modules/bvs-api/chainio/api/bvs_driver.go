package api

import (
	"context"
	"encoding/json"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-cw/types/bvs-driver"
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

func (s *bvsDriverImpl) WithGasAdjustment(gasAdjustment float64) BVSDriver {
	s.gasAdjustment = gasAdjustment
	return s
}

func (s *bvsDriverImpl) WithGasPrice(gasPrice sdktypes.DecCoin) BVSDriver {
	s.gasPrice = gasPrice
	return s
}

func (s *bvsDriverImpl) WithGasLimit(gasLimit uint64) BVSDriver {
	s.gasLimit = gasLimit
	return s
}

func (s *bvsDriverImpl) BindClient(contractAddress string) {
	s.contractAddr = contractAddress
}

func (s *bvsDriverImpl) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  s.contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: s.gasAdjustment,
		GasPrice:      s.gasPrice,
		Gas:           s.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (s *bvsDriverImpl) SetRegisteredBVSContract(ctx context.Context, addr string) (*coretypes.ResultTx, error) {
	s.registeredBVSContract = addr

	msg := bvsdriver.ExecuteMsg{
		AddRegisteredBvsContract: &bvsdriver.AddRegisteredBvsContract{
			Address: addr,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}

	executeOptions := s.newExecuteOptions(msgBytes, "SetRegisteredBVSContract")
	return s.io.SendTransaction(ctx, executeOptions)
}

func NewBVSDriverImpl(chainIO io.ChainIO) BVSDriver {
	return &bvsDriverImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
