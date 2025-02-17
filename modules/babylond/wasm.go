package babylond

import (
	"context"
	"fmt"
	"strconv"

	"cosmossdk.io/math"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-cw"
)

type DeployedWasmContract struct {
	CodeId  uint64
	Address string
}

func (c *BabylonContainer) DeployCrate(crate string, initMsg []byte, label, from string) (*DeployedWasmContract, error) {
	wasmByteCode, err := bvscw.ReadWasmFile(crate)
	if err != nil {
		panic(err)
	}

	return c.DeployWasmCode(wasmByteCode, initMsg, label, from)
}

func (c *BabylonContainer) DeployWasmCode(wasmByteCode []byte, initMsg []byte, label, from string) (*DeployedWasmContract, error) {
	res, err := c.StoreWasmCode(wasmByteCode, from)
	if err != nil {
		panic(err)
	}

	codeId, err := GetCodeId(res)
	if err != nil {
		panic(err)
	}

	res, err = c.InitWasmCode(codeId, initMsg, label, from)
	if err != nil {
		panic(err)
	}

	addr, err := GetContractAddress(res)
	if err != nil {
		panic(err)
	}

	return &DeployedWasmContract{
		CodeId:  codeId,
		Address: addr,
	}, nil
}

func (c *BabylonContainer) StoreWasmCode(wasmByteCode []byte, from string) (*coretypes.ResultBroadcastTxCommit, error) {
	clientCtx := c.withFromClientContext(from)
	txf, err := c.TxFactory.Prepare(clientCtx)
	if err != nil {
		panic(err)
	}

	msg := &wasmtypes.MsgStoreCode{
		Sender:       clientCtx.FromAddress.String(),
		WASMByteCode: wasmByteCode,
		InstantiatePermission: &wasmtypes.AccessConfig{
			Permission: wasmtypes.AccessTypeAnyOfAddresses,
			Addresses:  []string{clientCtx.FromAddress.String()},
		},
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetGasLimit(50000000)
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(100000))))
	if err := txBuilder.SetMsgs(msg); err != nil {
		return nil, fmt.Errorf("setting msg: %w", err)
	}

	if err := tx.Sign(context.Background(), txf, from, txBuilder, true); err != nil {
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

	return node.BroadcastTxCommit(context.Background(), txBytes)
}

func (c *BabylonContainer) InitWasmCode(codeId uint64, initMsg []byte, label, from string) (*coretypes.ResultBroadcastTxCommit, error) {
	clientCtx := c.withFromClientContext(from)
	txf, err := c.TxFactory.Prepare(clientCtx)
	if err != nil {
		panic(err)
	}

	msg := &wasmtypes.MsgInstantiateContract{
		Sender: clientCtx.FromAddress.String(),
		CodeID: codeId,
		Msg:    initMsg,
		Label:  label,
	}

	txBuilder := clientCtx.TxConfig.NewTxBuilder()
	txBuilder.SetGasLimit(50000000)
	txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("ubbn", math.NewInt(100000))))
	if err := txBuilder.SetMsgs(msg); err != nil {
		return nil, fmt.Errorf("setting msg: %w", err)
	}

	if err := tx.Sign(context.Background(), txf, from, txBuilder, true); err != nil {
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

	return node.BroadcastTxCommit(context.Background(), txBytes)
}

func (c *BabylonContainer) withFromClientContext(from string) client.Context {
	key, err := c.ClientCtx.Keyring.Key(from)
	if err != nil {
		panic(err)
	}

	fromAddr, err := key.GetAddress()
	if err != nil {
		panic(err)
	}

	return c.ClientCtx.WithFrom(from).WithFromName(from).WithFromAddress(fromAddr)
}

func GetCodeId(res *coretypes.ResultBroadcastTxCommit) (uint64, error) {
	if res.TxResult.Code != 0 {
		return 0, fmt.Errorf("CheckTx failed: %s", res.TxResult.Log)
	}

	for _, event := range res.TxResult.Events {
		if event.Type == "store_code" {
			for _, attr := range event.Attributes {
				if string(attr.Key) == "code_id" {
					return strconv.ParseUint(attr.Value, 10, 64)
				}
			}
		}
	}

	return 0, fmt.Errorf("code_id not found")
}

func GetContractAddress(res *coretypes.ResultBroadcastTxCommit) (string, error) {
	if res.TxResult.Code != 0 {
		return "", fmt.Errorf("CheckTx failed: %s", res.TxResult.Log)
	}

	for _, event := range res.TxResult.Events {
		if event.Type == "instantiate" {
			for _, attr := range event.Attributes {
				if string(attr.Key) == "_contract_address" {
					return attr.Value, nil
				}
			}
		}
	}

	return "", fmt.Errorf("contract_address not found")
}
