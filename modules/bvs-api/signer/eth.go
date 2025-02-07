package signer

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
)

type ETHSigner struct {
	client  *ethclient.Client
	chainID *big.Int
}

func NewETHSigner(client *ethclient.Client, chainID *big.Int) *ETHSigner {
	return &ETHSigner{client: client, chainID: chainID}
}

func (e *ETHSigner) BuildAndSignTx(
	ctx context.Context,
	signerFn bind.SignerFn,
	fromAddr common.Address,
	contractAddr common.Address,
	gasFeeCapAdjustmentRate int64,
	gasLimitAdjustmentRate float64,
	gasLimit uint64,
	input []byte,
) (*sdktypes.Transaction, error) {
	unsignedTx, err := e.BuildUnsignedTx(ctx, fromAddr, contractAddr, gasFeeCapAdjustmentRate, gasLimitAdjustmentRate, gasLimit, input)
	if err != nil {
		return nil, err
	}

	// Sign the transaction
	signedTx, err := signerFn(fromAddr, unsignedTx)
	if err != nil {
		return nil, fmt.Errorf("failed to sign transaction: %w", err)
	}
	return signedTx, nil
}

func (e *ETHSigner) BuildUnsignedTx(
	ctx context.Context,
	fromAddr common.Address,
	contractAddr common.Address,
	gasFeeCapAdjustmentRate int64,
	gasLimitAdjustmentRate float64,
	gasLimit uint64,
	input []byte,
) (*sdktypes.Transaction, error) {
	nonce, err := e.client.PendingNonceAt(ctx, fromAddr)
	if err != nil {
		return nil, fmt.Errorf("failed to get nonce: %w", err)
	}

	gasTipCap, gasFeeCap, err := e.SuggestGasFees(ctx, gasFeeCapAdjustmentRate)
	if err != nil {
		return nil, fmt.Errorf("failed to suggest gas fees: %w", err)
	}

	estimateGasLimit, err := e.client.EstimateGas(ctx, ethereum.CallMsg{
		From:      fromAddr,
		To:        &contractAddr,
		GasFeeCap: gasFeeCap,
		GasTipCap: gasTipCap,
		Data:      input,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to estimate gas: %w", err)
	}

	estimateGasLimit = uint64(float64(estimateGasLimit) * gasLimitAdjustmentRate)
	if estimateGasLimit > gasLimit {
		return nil, fmt.Errorf("failed to estimate gas limit (%d > %d)", estimateGasLimit, gasLimit)
	}

	tx := sdktypes.NewTx(&sdktypes.DynamicFeeTx{
		ChainID:   e.chainID,
		Nonce:     nonce,
		GasTipCap: gasTipCap,
		GasFeeCap: gasFeeCap,
		Gas:       estimateGasLimit,
		To:        &contractAddr,
		Data:      input,
	})
	return tx, nil
}

// SuggestGasFees
// gasTipCap  The user is willing to pay additional fees to the miner, in units of wei/gas
// gasFeeCap  The maximum fee per unit of gas that users are willing to pay for a transaction, also in units of wei/gas
func (e *ETHSigner) SuggestGasFees(ctx context.Context, gasFeeCapAdjustmentRate int64) (gasTipCap, gasFeeCap *big.Int, err error) {
	gasTipCap, err = e.client.SuggestGasTipCap(ctx)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to suggest gas tip cap: %w", err)
	}

	// get base fee
	head, err := e.client.HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to get header: %w", err)
	}

	// gasFeeCap = baseFee * 2 + gasTipCap
	gasFeeCap = new(big.Int).Mul(head.BaseFee, big.NewInt(gasFeeCapAdjustmentRate))
	gasFeeCap.Add(gasFeeCap, gasTipCap)

	return gasTipCap, gasFeeCap, nil
}
