package api

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"fmt"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
)

func Sha256(data []byte) []byte {
	hasher := sha256.New()
	hasher.Write(data)
	return hasher.Sum(nil)
}

type increaseAllowanceReq struct {
	IncreaseAllowance increaseAllowance `json:"increase_allowance"`
}

type increaseAllowance struct {
	Spender string `json:"spender"`
	Amount  string `json:"amount"`
}

// TODO: this is cw20 code, should not be here.

func IncreaseTokenAllowance(ctx context.Context, chainIO io.ChainIO, amount uint64, tokenContractAddr, spenderAddr string, gasPrice sdktypes.DecCoin) (*coretypes.ResultTx, error) {
	msg := increaseAllowanceReq{
		IncreaseAllowance: increaseAllowance{
			Spender: spenderAddr,
			Amount:  fmt.Sprintf("%d", amount),
		},
	}

	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	executeOptions := types.ExecuteOptions{
		ContractAddr:  tokenContractAddr,
		ExecuteMsg:    msgBytes,
		Funds:         "",
		GasAdjustment: 1.2,
		GasPrice:      gasPrice,
		Gas:           300000,
		Memo:          "test tx",
		Simulate:      true,
	}

	return chainIO.SendTransaction(ctx, executeOptions)
}
