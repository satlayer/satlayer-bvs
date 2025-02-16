package babylond

import (
	"context"
	"cosmossdk.io/math"
	"fmt"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (c *BabylonContainer) StoreWasmCode(ctx context.Context, WASMByteCode []byte, from string) (*coretypes.ResultBroadcastTxCommit, error) {
	key, err := c.ClientCtx.Keyring.Key(from)
	if err != nil {
		return nil, err
	}

	fromAddr, err := key.GetAddress()
	if err != nil {
		return nil, err
	}

	clientCtx := c.ClientCtx.WithFrom(from).WithFromName(from).WithFromAddress(fromAddr)
	txf, err := c.TxFactory.Prepare(clientCtx)
	if err != nil {
		return nil, err
	}

	msg := &wasmtypes.MsgStoreCode{
		Sender:       fromAddr.String(),
		WASMByteCode: WASMByteCode,
		InstantiatePermission: &wasmtypes.AccessConfig{
			Permission: wasmtypes.AccessTypeAnyOfAddresses,
			Addresses:  []string{fromAddr.String()},
		},
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetGasLimit(50000000)
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(100000))))
	if err := txBuilder.SetMsgs(msg); err != nil {
		return nil, fmt.Errorf("setting msg: %w", err)
	}

	if err := tx.Sign(ctx, txf, from, txBuilder, true); err != nil {
		return nil, fmt.Errorf("signing transaction: %w", err)
	}

	txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	if err != nil {
		return nil, fmt.Errorf("encoding transaction: %w", err)
	}

	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}

	return node.BroadcastTxCommit(ctx, txBytes)
}

func GetCodeId(res *coretypes.ResultBroadcastTxCommit) (string, error) {
	if res.TxResult.Code != 0 {
		return "", fmt.Errorf("CheckTx failed: %s", res.CheckTx.Log)
	}

	for _, event := range res.TxResult.Events {
		if event.Type == "store_code" {
			for _, attr := range event.Attributes {
				if string(attr.Key) == "code_id" {
					return attr.Value, nil
				}
			}
		}
	}

	return "", fmt.Errorf("code_id not found")
}
