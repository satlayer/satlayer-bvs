package cosmwasmapi

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"time"

	sdkmath "cosmossdk.io/math"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"

	sdkerrors "cosmossdk.io/errors"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// Query queries the smart contract with the given msg, and returns the response.
// Using the generated types, you can create queries like:
// `response, err := Query[pauser.IsPausedResponse](ClientCtx, ctx, address, queryMsg)`
func Query[Response interface{}](
	clientCtx client.Context, ctx context.Context, addr string, msg interface{},
) (Response, error) {
	var result Response
	queryClient := wasmtypes.NewQueryClient(clientCtx)

	queryBytes, err := json.Marshal(msg)
	if err != nil {
		return result, err
	}

	queryMsg := &wasmtypes.QuerySmartContractStateRequest{
		Address:   addr,
		QueryData: queryBytes,
	}

	response, err := queryClient.SmartContractState(ctx, queryMsg)
	if err != nil {
		return result, err
	}

	err = json.Unmarshal(response.Data, &result)
	return result, err
}

type ExecuteOptions struct {
	ContractAddr  string           // ContractAddr: Address of the smart contract
	ExecuteMsg    []byte           // ExecuteMsg: Message to be executed, represented as a struct
	Funds         string           // Funds: Amount of funds to send to the contract, represented as a string
	GasAdjustment float64          // GasAdjustment: Gas adjustment factor for adjusting the estimated gas amount
	GasPrice      sdktypes.DecCoin // GasPrice: Gas price, represented as a string (e.g., "0.025uatom")
	Gas           uint64           // Gas: Amount of gas reserved for transaction execution
	Simulate      bool             // Simulate: Whether to simulate the transaction to estimate gas usage and set Gas accordingly
}

func Execute(
	clientCtx client.Context, ctx context.Context, sender string, opts ExecuteOptions,
) (coretypes.ResultTx, error) {
	var result coretypes.ResultTx

	// build + broadcast tx
	res, err := BroadcastTx(clientCtx, ctx, sender, opts)
	if err != nil {
		return result, err
	}

	// poll for tx
	txHash := res.TxHash
	attempt := 1
	maxRetries := 10
	for {
		txRes, err := GetTx(clientCtx, ctx, txHash)
		if err == nil && txRes.TxResult.Code == 0 {
			return txRes, nil
		}
		attempt++
		if attempt > maxRetries {
			return result, errors.New("maximum number of retries reached")
		}
		// wait for 0.5 second before retrying
		time.Sleep(500 * time.Millisecond)
	}
}

func BroadcastTx(
	clientCtx client.Context, ctx context.Context, sender string, opts ExecuteOptions,
) (sdktypes.TxResponse, error) {
	var result sdktypes.TxResponse

	// TODO: move const to config
	const denom = "ubbn"

	amount, err := sdktypes.ParseCoinsNormalized(opts.Funds)
	if err != nil {
		return result, err
	}
	contractMsg := &wasmtypes.MsgExecuteContract{
		Sender:   sender,
		Contract: opts.ContractAddr,
		Msg:      opts.ExecuteMsg,
		Funds:    amount,
	}

	// BUILD TXs
	txf := tx.Factory{}.
		WithChainID(clientCtx.ChainID).
		WithKeybase(clientCtx.Keyring).
		WithTxConfig(clientCtx.TxConfig).
		WithAccountRetriever(clientCtx.AccountRetriever).
		WithFromName(clientCtx.FromName).
		WithSignMode(signing.SignMode_SIGN_MODE_DIRECT).
		WithSimulateAndExecute(opts.Simulate).
		WithGasAdjustment(opts.GasAdjustment).
		WithGasPrices(opts.GasPrice.String()).
		WithGas(opts.Gas)

	txf, err = txf.Prepare(clientCtx)
	if err != nil {
		return result, err
	}

	// whether to simulate gas calculations
	if txf.SimulateAndExecute() {
		_, adjusted, err := tx.CalculateGas(clientCtx, txf, contractMsg)
		if err != nil {
			return result, err
		}
		if adjusted > opts.Gas {
			adjusted = opts.Gas
		}
		txf = txf.WithGas(adjusted)
	}

	senderAccAddress, err := sdktypes.AccAddressFromBech32(sender)
	if err != nil {
		return result, err
	}
	txBuilder := clientCtx.TxConfig.NewTxBuilder()

	// Calculate the fee based on the gas and gas price (fee = Gas * GasPrice)
	gasInt := sdkmath.NewIntFromUint64(txf.Gas())
	fee := txf.GasPrices().AmountOf(denom).MulInt(gasInt).RoundInt()
	txBuilder.SetFeeAmount(sdktypes.NewCoins(sdktypes.NewCoin(denom, fee)))
	txBuilder.SetGasLimit(txf.Gas())
	txBuilder.SetFeePayer(senderAccAddress)

	err = txBuilder.SetMsgs(contractMsg)
	if err != nil {
		return result, err
	}

	// Sign Tx
	keyName, err := clientCtx.Keyring.KeyByAddress(senderAccAddress)
	if err != nil {
		return result, err
	}
	err = tx.Sign(ctx, txf, keyName.Name, txBuilder, true)
	if err != nil {
		return result, err
	}

	signedTx := txBuilder.GetTx()

	// Encode the transaction
	txBytes, err := clientCtx.TxConfig.TxEncoder()(signedTx)
	if err != nil {
		return result, err
	}

	// Broadcast TX
	res, err := clientCtx.BroadcastTx(txBytes)
	if err != nil {
		return result, err
	}

	// Code must be 0 for transaction to be valid (validated by consensus)
	if res.Code != 0 {
		// Cast the error from tx response to registered errors
		err = sdkerrors.ABCIError(res.Codespace, res.Code, res.RawLog)
		return result, err
	}

	return *res, nil
}

func GetTx(
	clientCtx client.Context, ctx context.Context, txHash string,
) (coretypes.ResultTx, error) {
	var result coretypes.ResultTx
	txHashBytes, err := hex.DecodeString(txHash)
	if err != nil {
		return result, err
	}

	node, err := clientCtx.GetNode()
	if err != nil {
		return result, err
	}
	res, err := node.Tx(ctx, txHashBytes, true)
	if err != nil {
		return result, err
	}

	return *res, nil
}
