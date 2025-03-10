package io

import (
	"context"
	"encoding/hex"
	"fmt"
	"time"

	"go.uber.org/zap"

	"cosmossdk.io/math"
	"github.com/CosmWasm/wasmd/x/wasm"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/std"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/module"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/signer"
)

type ChainIO interface {
	// SetupKeyring Please call this method to set the necessary configuration before executing the transaction.
	// @param keyringBackend
	// os: Use the operating system's secure storage (e.g. keychain for macOS, libsecret for Linux)
	// file: Use the file system to store keys
	// test: Use memory to store keys, mainly for testing purposes
	SetupKeyring(keyName, keyringBackend string, keyringServiceName ...string) (ChainIO, error)
	SendTransaction(ctx context.Context, opts types.ExecuteOptions) (*coretypes.ResultTx, error)
	ExecuteContract(opts types.ExecuteOptions) (*sdktypes.TxResponse, error)
	BroadcastTx(signedTx sdktypes.Tx) (*sdktypes.TxResponse, error)
	QueryContract(opts types.QueryOptions) (*wasmtypes.QuerySmartContractStateResponse, error)
	QueryNodeStatus(ctx context.Context) (*coretypes.ResultStatus, error)
	QueryTransaction(txHash string) (*coretypes.ResultTx, error)
	QueryAccount(address string) (client.Account, error)
	IsCurrentAccountOnChain() bool
	GetCurrentAccountPubKey() cryptotypes.PubKey
	GetCurrentAccount() (client.Account, error)
	GetClientCtx() client.Context
	GetSigner() *signer.Signer
}

// ChainIO chain io Facade
type chainIO struct {
	clientCtx client.Context
	signer    *signer.Signer
	pubKey    cryptotypes.PubKey
	params    types.TxManagerParams
}

func (c chainIO) SetupKeyring(keyName, keyringBackend string, keyringServiceName ...string) (ChainIO, error) {
	keyringServiceNameStr := types.DefaultKeyringServiceName
	if len(keyringServiceName) > 0 {
		keyringServiceNameStr = keyringServiceName[0]
	}
	// init keyring
	kr, err := newKeyringFromBackend(c.clientCtx, keyringBackend, keyringServiceNameStr)
	if err != nil {
		return &c, err
	}
	c.clientCtx = c.clientCtx.WithKeyring(kr)

	// Load account information
	keyInfo, err := kr.Key(keyName)
	if err != nil {
		return &c, err
	}
	accAddress, err := keyInfo.GetAddress()
	if err != nil {
		return &c, err
	}

	pubKey, err := keyInfo.GetPubKey()
	if err != nil {
		return &c, err
	}
	c.pubKey = pubKey

	c.clientCtx = c.clientCtx.WithFromAddress(accAddress).WithFromName(keyName)
	c.signer = signer.NewSigner(c.clientCtx)
	return c, nil
}

func (c chainIO) SendTransaction(ctx context.Context, opts types.ExecuteOptions) (*coretypes.ResultTx, error) {
	speedups := 0
	var (
		txResp *sdktypes.TxResponse
		err    error
	)

	for attempt := 0; attempt < c.params.MaxRetries; attempt++ {
		txResp, err = c.ExecuteContract(opts)
		if err == nil {
			break
		}
		zap.L().Warn("Failed to send transaction", zap.Int("attempt", attempt+1), zap.Error(err))
		if attempt == c.params.MaxRetries-1 {
			return nil, fmt.Errorf("max retries exceeded: %w", err)
		}
		// adjust GasPrice
		opts.GasPrice = sdktypes.NewDecCoinFromDec(opts.GasPrice.Denom, opts.GasPrice.Amount.Mul(math.LegacyMustNewDecFromStr(c.params.GasPriceAdjustmentRate)))
		time.Sleep(c.params.RetryInterval)
		speedups++
	}

	if txResp == nil {
		return nil, fmt.Errorf("failed to send transaction after %d attempts", c.params.MaxRetries)
	}

	confirmedTxResp, err := c.waitForConfirmation(ctx, txResp.TxHash)
	if err != nil {
		return nil, err
	}

	return confirmedTxResp, nil
}

func (c chainIO) ExecuteContract(opts types.ExecuteOptions) (*sdktypes.TxResponse, error) {
	amount, err := sdktypes.ParseCoinsNormalized(opts.Funds)
	if err != nil {
		return nil, err
	}
	contractMsg := &wasmtypes.MsgExecuteContract{
		Sender:   c.clientCtx.GetFromAddress().String(),
		Contract: opts.ContractAddr,
		Msg:      opts.ExecuteMsg,
		Funds:    amount,
	}
	// Build, sign, and broadcast transactions
	signedTx, err := c.signer.BuildAndSignTx(opts.GasAdjustment, opts.GasPrice, opts.Gas, opts.Memo, opts.Simulate, contractMsg)
	if err != nil {
		return nil, err
	}

	resp, err := c.BroadcastTx(signedTx)
	if err != nil {
		return nil, err
	}
	return resp, nil
}

func (c chainIO) BroadcastTx(signedTx sdktypes.Tx) (*sdktypes.TxResponse, error) {
	// Encode the transaction
	txBytes, err := c.clientCtx.TxConfig.TxEncoder()(signedTx)
	if err != nil {
		return nil, err
	}
	// Broadcast the transaction
	return c.clientCtx.BroadcastTx(txBytes)
}

func (c chainIO) waitForConfirmation(ctx context.Context, txHash string) (*coretypes.ResultTx, error) {
	ticker := time.NewTicker(3 * time.Second)
	defer ticker.Stop()

	timeout := time.After(c.params.ConfirmationTimeout)

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timeout:
			return nil, fmt.Errorf("transaction confirmation timed out")
		case <-ticker.C:
			txResp, err := c.QueryTransaction(txHash)
			if err != nil {
				zap.L().Debug("Failed to query transaction", zap.String("txHash", txHash), zap.Error(err))
				continue
			}

			if txResp.TxResult.Code == 0 {
				return txResp, nil
			} else if txResp.TxResult.Code != 0 {
				return nil, fmt.Errorf("transaction failed with code %d: %s", txResp.TxResult.Code, txResp.TxResult.Log)
			}
		}
	}
}

func (c chainIO) QueryContract(opts types.QueryOptions) (*wasmtypes.QuerySmartContractStateResponse, error) {
	queryClient := wasmtypes.NewQueryClient(c.clientCtx)
	queryMsg := &wasmtypes.QuerySmartContractStateRequest{
		Address:   opts.ContractAddr,
		QueryData: opts.QueryMsg,
	}

	resp, err := queryClient.SmartContractState(context.Background(), queryMsg)
	if err != nil {
		return nil, err
	}
	return resp, nil
}

func (c chainIO) QueryNodeStatus(ctx context.Context) (*coretypes.ResultStatus, error) {
	return c.clientCtx.Client.Status(ctx)
}

func (c chainIO) QueryTransaction(txHash string) (*coretypes.ResultTx, error) {
	hashBytes, err := hex.DecodeString(txHash)
	if err != nil {
		return nil, err
	}
	resp, err := c.clientCtx.Client.Tx(context.Background(), hashBytes, false)
	if err != nil {
		return nil, err
	}
	return resp, nil
}

func (c chainIO) QueryAccount(address string) (client.Account, error) {
	addr, err := sdktypes.AccAddressFromBech32(address)
	if err != nil {
		return nil, err
	}
	account, err := c.clientCtx.AccountRetriever.GetAccount(c.clientCtx, addr)
	if err != nil {
		return nil, err
	}
	return account, nil
}

func (c chainIO) IsCurrentAccountOnChain() bool {
	account, err := c.clientCtx.AccountRetriever.GetAccount(c.clientCtx, c.clientCtx.GetFromAddress())
	if err != nil {
		return false
	}

	if account.GetPubKey() == nil {
		return false
	}

	return true
}

func (c chainIO) GetCurrentAccount() (client.Account, error) {
	return c.clientCtx.AccountRetriever.GetAccount(c.clientCtx, c.clientCtx.GetFromAddress())
}

func (c chainIO) GetClientCtx() client.Context {
	return c.clientCtx
}

func (c chainIO) GetSigner() *signer.Signer {
	return c.signer
}

func (c chainIO) GetCurrentAccountPubKey() cryptotypes.PubKey {
	return c.pubKey
}

func NewChainIO(chainID, rpcURI, homeDir, bech32Prefix string, params types.TxManagerParams) (ChainIO, error) {
	// Set address prefixes
	if err := setAddressPrefixes(bech32Prefix); err != nil {
		return nil, fmt.Errorf("failed to set address prefixes: %w", err)
	}
	// Initialize codec and interface registry
	interfaceRegistry, marshaler, legacyAmino := initCodec()
	// Initialize client context
	clientCtx := initClientContext(chainID, interfaceRegistry, marshaler, legacyAmino)

	// init rpcClient
	rpcClient, err := client.NewClientFromNode(rpcURI)
	if err != nil {
		return nil, err
	}
	clientCtx = clientCtx.WithClient(rpcClient)

	if homeDir != "" {
		clientCtx = clientCtx.WithHomeDir(homeDir).WithKeyringDir(homeDir)
	}
	return chainIO{
		clientCtx: clientCtx,
		signer:    signer.NewSigner(clientCtx),
		params:    params,
	}, nil
}

func setAddressPrefixes(bech32Prefix string) error {
	config := sdktypes.GetConfig()
	config.SetBech32PrefixForAccount(bech32Prefix, bech32Prefix+"pub")
	config.SetBech32PrefixForValidator(bech32Prefix+"valoper", bech32Prefix+"valoperpub")
	config.SetBech32PrefixForConsensusNode(bech32Prefix+"valcons", bech32Prefix+"valconspub")

	config.SetAddressVerifier(func(bytes []byte) error {
		if len(bytes) == 0 {
			return fmt.Errorf("addresses cannot be empty")
		}

		if len(bytes) > address.MaxAddrLen {
			return fmt.Errorf("address max length is %d, got %d, %x", address.MaxAddrLen, len(bytes), bytes)
		}

		if len(bytes) != 20 && len(bytes) != 32 {
			return fmt.Errorf("address length must be 20 or 32 bytes, got %d, %x", len(bytes), bytes)
		}

		return nil
	})

	return nil
}

func initCodec() (codectypes.InterfaceRegistry, codec.Codec, *codec.LegacyAmino) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	authtypes.RegisterInterfaces(interfaceRegistry)
	cryptocodec.RegisterInterfaces(interfaceRegistry)
	std.RegisterInterfaces(interfaceRegistry)

	marshaler := codec.NewProtoCodec(interfaceRegistry)

	legacyAmino := codec.NewLegacyAmino()
	std.RegisterLegacyAminoCodec(legacyAmino)
	module.NewBasicManager(wasm.AppModuleBasic{}).RegisterInterfaces(interfaceRegistry)

	return interfaceRegistry, marshaler, legacyAmino
}

func initClientContext(chainID string, interfaceRegistry codectypes.InterfaceRegistry, marshaler codec.Codec, legacyAmino *codec.LegacyAmino) client.Context {
	txConfig := authtx.NewTxConfig(marshaler, authtx.DefaultSignModes)
	return client.Context{}.
		WithChainID(chainID).
		WithOutputFormat("json").
		WithInterfaceRegistry(interfaceRegistry).
		WithTxConfig(txConfig).
		WithCodec(marshaler).
		WithLegacyAmino(legacyAmino).
		WithAccountRetriever(authtypes.AccountRetriever{}).
		WithBroadcastMode(flags.BroadcastSync)
}

// newKeyringFromBackend gets a Keyring object from a backend
func newKeyringFromBackend(ctx client.Context, backend, keyringServiceName string) (keyring.Keyring, error) {
	if ctx.Simulate {
		backend = keyring.BackendMemory
	}

	if len(keyringServiceName) == 0 {
		keyringServiceName = types.DefaultKeyringServiceName
	}

	return keyring.New(keyringServiceName, backend, ctx.KeyringDir, ctx.Input, ctx.Codec, ctx.KeyringOptions...)
}
