package cosmwasmapi

import (
	"context"
	"encoding/json"

	sdkerrors "cosmossdk.io/errors"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
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
) (coretypes.ResultBroadcastTxCommit, error) {
	var result coretypes.ResultBroadcastTxCommit
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
		WithSimulateAndExecute(true).
		WithGasAdjustment(1.3).
		WithGasPrices("1ubbn")

	txf, err = txf.Prepare(clientCtx)
	if err != nil {
		return result, err
	}

	// TODO: check if this is necessary or not
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
	txBuilder.SetFeeAmount(sdktypes.NormalizeCoins(sdktypes.NewDecCoins(opts.GasPrice)))
	txBuilder.SetFeePayer(senderAccAddress)
	txBuilder.SetGasLimit(opts.Gas)
	err = txBuilder.SetMsgs(contractMsg)
	if err != nil {
		return result, err
	}

	// SIGN TX
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

	// BROADCAST TX
	node, err := clientCtx.GetNode()
	if err != nil {
		panic(err)
	}
	res, err := node.BroadcastTxCommit(context.Background(), txBytes)
	if err != nil {
		panic(err)
	}

	// code must be 0 for successful transaction
	if res.CheckTx.IsErr() {
		// cast the error from tx response to registered errors
		err = sdkerrors.ABCIError(res.CheckTx.Codespace, res.CheckTx.Code, res.CheckTx.Log)
		return result, err
	}

	// code must be 0 for successful transaction
	if res.TxResult.IsErr() {
		// cast the error from tx response to registered errors
		err = sdkerrors.ABCIError(res.TxResult.Codespace, res.TxResult.Code, res.TxResult.Log)
		return result, err
	}

	resBytes, err := json.Marshal(res)

	err = json.Unmarshal(resBytes, &result)

	return result, nil
}
