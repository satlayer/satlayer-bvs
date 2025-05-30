package babylond

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"strings"

	"github.com/CosmWasm/wasmd/x/wasm"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/std"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/docker/go-connections/nat"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
)

const (
	apiPort  = "1317"
	grpcPort = "9090"
	rpcPort  = "26657"
	ChainId  = "sat-bbn-localnet"
)

type BabylonContainer struct {
	testcontainers.Container
	ChainId   string
	ApiUri    string
	RpcUri    string
	GrpcUri   string
	ClientCtx client.Context
	TxFactory tx.Factory
}

func getHost(ctx context.Context, container testcontainers.Container, port nat.Port) string {
	// Technically, Container.Host should be Container.Hostname
	host, err := container.Host(ctx)
	if err != nil {
		panic(err)
	}
	port, err = container.MappedPort(ctx, port)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("%s:%s", host, port.Port())
}

func newClientCtx(rpcUri string) client.Context {
	config := sdk.GetConfig()
	config.SetBech32PrefixForAccount("bbn", "bbnpub")

	rpcClient, err := client.NewClientFromNode(rpcUri)
	if err != nil {
		panic(err)
	}
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	authtypes.RegisterInterfaces(interfaceRegistry)
	cryptocodec.RegisterInterfaces(interfaceRegistry)
	std.RegisterInterfaces(interfaceRegistry)
	wasmtypes.RegisterInterfaces(interfaceRegistry)

	legacyAmino := codec.NewLegacyAmino()
	std.RegisterLegacyAminoCodec(legacyAmino)
	wasmtypes.RegisterLegacyAminoCodec(legacyAmino)
	module.NewBasicManager(wasm.AppModuleBasic{}).RegisterInterfaces(interfaceRegistry)

	protoCodec := codec.NewProtoCodec(interfaceRegistry)
	txConfig := authtx.NewTxConfig(protoCodec, authtx.DefaultSignModes)

	memoryKeyring := keyring.NewInMemory(protoCodec)
	err = memoryKeyring.ImportPrivKeyHex("genesis", "230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A", "secp256k1")
	if err != nil {
		panic(err)
	}

	return client.Context{}.
		WithChainID(ChainId).
		WithClient(rpcClient).
		WithKeyring(memoryKeyring).
		WithOutputFormat("json").
		WithInterfaceRegistry(interfaceRegistry).
		WithTxConfig(txConfig).
		WithCodec(protoCodec).
		WithLegacyAmino(legacyAmino).
		WithAccountRetriever(authtypes.AccountRetriever{}).
		WithBroadcastMode(flags.BroadcastSync)
}

func newTxFactory(clientCtx client.Context) tx.Factory {
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
	return txf
}

func Run(ctx context.Context) *BabylonContainer {
	cmd := []string{
		// Setup Testnet
		"babylond",
		"testnet",
		"--v",
		"1",
		"--output-dir",
		".localnet",
		"--keyring-backend",
		"test",
		"--chain-id",
		ChainId,
		"&&",
		// Update Timeout Commit from 5s to 1s
		"sed",
		"-i",
		"'s/timeout_commit = \"5s\"/timeout_commit = \"1s\"/'",
		".localnet/node0/babylond/config/config.toml",
		"&&",
		// Setup Keyring
		"babylond",
		"keys",
		"import-hex",
		"genesis",
		"230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A",
		"--keyring-backend",
		"test",
		"--home",
		".localnet/node0/babylond",
		"&&",
		// Setup Genesis Account
		"babylond",
		"add-genesis-account",
		"bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		"100000000000000000ubbn",
		"--home",
		".localnet/node0/babylond",
		"&&",
		// Start babylond
		"babylond",
		"start",
		"--home",
		".localnet/node0/babylond",
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: testcontainers.ContainerRequest{
			Image: "babylonlabs/babylond:v1.0.0-rc.6",
			Entrypoint: []string{
				"sh",
				"-c",
			},
			Cmd: []string{
				strings.Join(cmd[:], " "),
			},
			ExposedPorts: []string{apiPort, grpcPort, rpcPort},
			WaitingFor: wait.ForHTTP("/status").
				WithPort(rpcPort).WithResponseMatcher(func(body io.Reader) bool {
				var data map[string]interface{}
				if err := json.NewDecoder(body).Decode(&data); err != nil {
					return false
				}
				data = data["result"].(map[string]interface{})
				data = data["sync_info"].(map[string]interface{})
				height := data["latest_block_height"].(string)
				return height != "0"
			}),
		},
		Started: true,
	})

	if err != nil {
		panic(err)
	}

	ApiUri := fmt.Sprintf("http://%s", getHost(ctx, container, apiPort))
	RpcUri := fmt.Sprintf("http://%s", getHost(ctx, container, rpcPort))
	GrpcUri := fmt.Sprintf("%s", getHost(ctx, container, grpcPort))
	ClientCtx := newClientCtx(RpcUri)
	TxFactory := newTxFactory(ClientCtx)

	return &BabylonContainer{
		container,
		ChainId,
		ApiUri,
		RpcUri,
		GrpcUri,
		ClientCtx,
		TxFactory,
	}
}
